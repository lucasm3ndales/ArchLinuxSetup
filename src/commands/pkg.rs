use super::aur;
use crate::adapters::package_state::PackageTracker;
use crate::adapters::pkg::LinuxPkg;
use crate::application::pkg::{
    self, Category, NVIDIA_580XX_CONFLICTS, PackageSource, RustProvider, Selection,
};
use crate::project;
use std::ffi::OsString;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Eq, PartialEq)]
struct Options {
    categories: Vec<Category>,
    profile: Option<String>,
    dry_run: bool,
}

fn parse_args(args: &[OsString]) -> Result<Option<Options>, String> {
    let mut options = Options::default();
    let mut index = 0;
    while index < args.len() {
        let value = args[index].to_string_lossy();
        match value.as_ref() {
            "--dry-run" => options.dry_run = true,
            "--profile" => {
                index += 1;
                let profile = args
                    .get(index)
                    .ok_or("Falta o nome após --profile.")?
                    .to_string_lossy();
                if !project::valid_profile_name(&profile) {
                    return Err("Nome de perfil inválido.".to_owned());
                }
                options.profile = Some(profile.into_owned());
            }
            "-h" | "--help" => return Ok(None),
            name => match Category::parse(name) {
                Some(category) => options.categories.push(category),
                None => return Err(format!("Opção ou categoria desconhecida: {name}")),
            },
        }
        index += 1;
    }
    if options.categories.is_empty() {
        options.categories.extend(Category::DEFAULT);
    }
    Ok(Some(options))
}

pub fn help() -> i32 {
    println!("Uso: archsetup pkg [CATEGORIA...] [--profile NOME] [--dry-run]\n");
    println!("Categorias: base development apps theme desktop gaming emulation");
    println!("Padrão: base development apps theme desktop\n");
    println!("O perfil é opcional. Sem ele, são usados apenas os pacotes compartilhados.");
    println!("--dry-run mostra a seleção sem atualizar ou instalar nada.");
    0
}

pub fn run(root: &Path, args: &[OsString]) -> i32 {
    let options = match parse_args(args) {
        Ok(Some(options)) => options,
        Ok(None) => return help(),
        Err(error) => {
            eprintln!("Erro: {error}");
            return 2;
        }
    };
    if let Some(profile) = &options.profile {
        if !root.join("profiles").join(profile).is_dir() {
            eprintln!("Erro: Perfil inexistente: {profile}");
            return 2;
        }
    }
    let home = match std::env::var_os("HOME").filter(|home| !home.is_empty()) {
        Some(home) => PathBuf::from(home),
        None => {
            eprintln!("Erro: HOME não está definido.");
            return 1;
        }
    };
    let state_home = std::env::var_os("XDG_STATE_HOME").map(PathBuf::from);
    let linux = LinuxPkg::new(root, home.clone(), state_home.clone());
    let selection = match pkg::select(
        &linux,
        root,
        &options.categories,
        options.profile.as_deref(),
    ) {
        Ok(selection) => selection,
        Err(error) => {
            eprintln!("Erro: {error}");
            return 1;
        }
    };
    show_selection(&linux, &selection, &options);
    if options.dry_run {
        if !selection.aur.is_empty() && linux.helper().is_none() {
            println!(
                "\nDependência: archsetup aur instala o auxiliar necessário para os pacotes AUR."
            );
        }
        if selection.includes(Category::Development) {
            println!(
                "\nDocker: adicionar o usuário atual ao grupo docker, se necessário, e habilitar/iniciar docker.service."
            );
            if linux.installed("ufw") {
                println!("Aviso: portas publicadas pelo Docker podem contornar regras do UFW.");
            }
        }
        if selection.includes(Category::Gaming) {
            println!(
                "\nSteam exige [multilib] e bibliotecas gráficas de 32 bits compatíveis com a GPU."
            );
        }
        println!(
            "\nPrévia concluída. Nenhuma alteração realizada. O estado de dependências não foi calculado."
        );
        return 0;
    }
    if let Err(error) = linux.preflight(&selection, options.profile.as_deref()) {
        eprintln!("Erro: {error}");
        return 1;
    }
    let helper = if selection.aur.is_empty() {
        None
    } else {
        match ensure_aur_helper(&linux) {
            Ok(helper) => Some(helper),
            Err(error) => {
                eprintln!("Erro: {error}");
                return 1;
            }
        }
    };
    let tracker = match PackageTracker::begin(&home, state_home.as_deref()) {
        Ok(tracker) => tracker,
        Err(error) => {
            eprintln!("Erro: {error}");
            return 1;
        }
    };
    let result = linux.apply(&selection, helper);
    let tracked = tracker.finish();
    if let Ok(count) = &tracked {
        if *count > 0 {
            println!("Pacotes novos registrados para possível remoção: {count}");
        }
    }
    if let Err(error) = result {
        eprintln!("Erro: {error}");
        if let Err(tracking_error) = tracked {
            eprintln!("Aviso: registro de pacotes incompleto: {tracking_error}");
        }
        return 1;
    }
    if let Err(error) = tracked {
        eprintln!("Erro: instalação concluída, mas registro de pacotes falhou: {error}");
        return 1;
    }
    let categories = selection
        .categories
        .iter()
        .map(|category| category.name())
        .collect::<Vec<_>>()
        .join(" ");
    println!(
        "\nConcluído: categorias {categories} processadas. Pacotes já instalados são ignorados por --needed."
    );
    0
}

fn show_selection(source: &impl PackageSource, selection: &Selection, options: &Options) {
    println!(
        "==> {} de pacotes",
        if options.dry_run {
            "Prévia"
        } else {
            "Instalação"
        }
    );
    println!(
        "Perfil: {}",
        options
            .profile
            .as_deref()
            .unwrap_or("nenhum (somente pacotes compartilhados)")
    );
    println!(
        "Categorias: {}",
        selection
            .categories
            .iter()
            .map(|category| category.name())
            .collect::<Vec<_>>()
            .join(" ")
    );
    let mut installed = 0;
    let mut missing = 0;
    println!("\nPacotes oficiais ({}):", selection.official.len());
    for package in &selection.official {
        show_package(source, package, &mut installed, &mut missing);
    }
    println!("\nPacotes AUR ({}):", selection.aur.len());
    if selection.aur.is_empty() {
        println!("  nenhum");
    } else {
        for package in &selection.aur {
            show_package(source, package, &mut installed, &mut missing);
        }
    }
    if source.pacman_available() {
        println!("\nResumo: {installed} instalados; {missing} ausentes.");
    }
    if selection.rust_provider == RustProvider::Rust {
        println!(
            "\nRust: rust já está instalado; rustup foi retirado da seleção para evitar conflito."
        );
    }
    if selection.aur.iter().any(|name| name == "nvidia-580xx-dkms") {
        let conflicts = NVIDIA_580XX_CONFLICTS
            .iter()
            .copied()
            .filter(|name| source.installed(name))
            .collect::<Vec<_>>();
        if !conflicts.is_empty() {
            println!("\n[AVISO] NVIDIA 580xx conflita com: {}. A aplicação será recusada até revisar esses pacotes.", conflicts.join(", "));
        }
    }
}

fn show_package(
    source: &impl PackageSource,
    package: &str,
    installed: &mut usize,
    missing: &mut usize,
) {
    if !source.pacman_available() {
        println!("  {package:<28} não verificado (pacman indisponível)");
    } else if source.installed(package) {
        *installed += 1;
        println!("  {package:<28} instalado");
    } else {
        *missing += 1;
        println!("  {package:<28} ausente");
    }
}

fn ensure_aur_helper(linux: &LinuxPkg) -> Result<&'static str, String> {
    if let Some(helper) = linux.helper() {
        return Ok(helper);
    }
    println!("\nDependência: os pacotes AUR exigem um auxiliar. archsetup aur instala o paru.");
    if !io::stdin().is_terminal() {
        return Err(
            "Execute archsetup aur em um terminal interativo e tente novamente.".to_owned(),
        );
    }
    print!("Executar archsetup aur agora? [s/N] ");
    io::stdout().flush().map_err(|error| error.to_string())?;
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .map_err(|_| "Não foi possível ler a resposta.".to_owned())?;
    if !matches!(answer.trim(), "s" | "S" | "sim" | "SIM" | "Sim") {
        return Err("Dependência não executada. Instale o auxiliar e tente novamente.".to_owned());
    }
    if aur::run(&[]) != 0 {
        return Err("archsetup aur falhou; confira a saída acima.".to_owned());
    }
    if !crate::system::command_exists("paru") {
        return Err("archsetup aur terminou, mas paru não foi encontrado no PATH.".to_owned());
    }
    Ok("paru")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_new_categories_and_default_order() {
        assert_eq!(
            parse_args(&[]).unwrap().unwrap().categories,
            Category::DEFAULT
        );
        let options = parse_args(&["theme".into(), "apps".into(), "--dry-run".into()])
            .unwrap()
            .unwrap();
        assert_eq!(options.categories, vec![Category::Theme, Category::Apps]);
        assert!(options.dry_run);
        assert!(parse_args(&["visual".into()]).is_err());
        assert!(parse_args(&["--profile".into(), "../wrong".into()]).is_err());
    }
}
