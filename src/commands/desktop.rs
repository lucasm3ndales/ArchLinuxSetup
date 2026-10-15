use super::pkg;
use crate::adapters::desktop::LinuxDesktop;
use crate::adapters::linux::LinuxInspector;
use crate::application::doctor;
use crate::project;
use std::ffi::OsString;
use std::io::{self, IsTerminal, Write};
use std::path::Path;

#[derive(Debug, Default, Eq, PartialEq)]
struct Options {
    profile: Option<String>,
    dry_run: bool,
}

fn parse_args(args: &[OsString]) -> Result<Option<Options>, String> {
    let mut options = Options::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].to_string_lossy().as_ref() {
            "--dry-run" => options.dry_run = true,
            "--profile" => {
                index += 1;
                let profile = args
                    .get(index)
                    .ok_or("Falta o nome após --profile.")?
                    .to_string_lossy();
                if !project::valid_profile_name(&profile) {
                    return Err("Perfil inexistente ou inválido".to_owned());
                }
                options.profile = Some(profile.into_owned());
            }
            "-h" | "--help" => return Ok(None),
            _ => return Err("desktop aceita somente --profile NOME e --dry-run".to_owned()),
        }
        index += 1;
    }
    Ok(Some(options))
}

pub fn help() -> i32 {
    println!("Uso: archsetup desktop [--profile NOME] [--dry-run]\n");
    println!("Configura Xfce e touchpad X11; ajusta o greeter LightDM/Slick ou Ly, se habilitado.");
    println!("Execute como usuário comum em uma sessão X11 do Xfce.");
    println!("O perfil opcional aplica somente ajustes de hardware compatíveis.");
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
    let profile_data = if let Some(profile) = &options.profile {
        if !root.join("profiles").join(profile).is_dir() {
            eprintln!("Erro: Perfil inexistente ou inválido: {profile}");
            return 1;
        }
        match doctor::read_profile(&LinuxInspector, root, profile) {
            Ok(data) => Some(data),
            Err(error) => { eprintln!("Erro: {error}"); return 1; }
        }
    } else { None };
    let linux = match LinuxDesktop::new(root) {
        Ok(linux) => linux,
        Err(error) => {
            eprintln!("Erro: {error}");
            return 1;
        }
    };
    if let Err(error) = linux.check_xfce_environment() {
        eprintln!("Erro: {error}");
        return 1;
    }
    println!(
        "==> {} do desktop",
        if options.dry_run {
            "Prévia"
        } else {
            "Configuração"
        }
    );
    println!("Perfil: {}", options.profile.as_deref().unwrap_or("nenhum"));
    println!(
        "  Xfce: Nordic, ícones Zafiro Dark, cursor Hackneyed, fontes, painel, busca, teclas Lock, espaços de trabalho e tecla Super."
    );
    if linux.uses_lightdm() {
        println!("  Login: Nordic, Zafiro Dark e Hackneyed no LightDM com Slick Greeter.");
    } else if linux.uses_ly() {
        println!("  Login: animação Matrix e data/hora no Ly.");
    } else {
        println!("  Login: gerenciador atual preservado; tema do greeter não será alterado.");
    }
    println!("  Touchpad X11: rolagem natural.");
    if profile_data.as_ref().is_some_and(|data| data.touchpad_tapping) {
        println!("  Touchpad X11: toque para clicar ativado por este perfil.");
    }
    if options.dry_run {
        match linux.missing_packages() {
            Some(missing) if !missing.is_empty() => {
                println!(
                    "Pacotes ausentes: {}\nDependência: archsetup pkg desktop theme.",
                    missing.join(" ")
                );
            }
            Some(_) => {
                println!("Pacotes necessários das categorias desktop e theme já instalados.")
            }
            None => println!("Pacotes necessários não verificados (pacman indisponível)."),
        }
        println!("Prévia concluída. Nenhuma alteração realizada.");
        return 0;
    }
    if let Err(error) = linux.preflight(options.profile.as_deref()) {
        eprintln!("Erro: {error}");
        return 1;
    }
    if let Some(missing) = linux.missing_packages() {
        if !missing.is_empty() {
            println!("Pacotes necessários ausentes: {}", missing.join(" "));
            println!("Dependência: archsetup pkg desktop theme.");
            if !io::stdin().is_terminal() {
                eprintln!(
                    "Erro: Execute archsetup pkg desktop theme em um terminal interativo e tente novamente."
                );
                return 1;
            }
            print!("Executar archsetup pkg desktop theme agora? [s/N] ");
            if let Err(error) = io::stdout().flush() {
                eprintln!("Erro: {error}");
                return 1;
            }
            let mut answer = String::new();
            if io::stdin().read_line(&mut answer).is_err() {
                eprintln!("Erro: Não foi possível ler a resposta.");
                return 1;
            }
            if !matches!(answer.trim(), "s" | "S" | "sim" | "SIM" | "Sim") {
                eprintln!(
                    "Erro: Dependência não executada. Configure os pacotes e tente novamente."
                );
                return 1;
            }
            let status = pkg::run(root, &["desktop".into(), "theme".into()]);
            if status != 0 {
                eprintln!("Erro: instalação de desktop e theme falhou (código {status}).");
                return status;
            }
        }
    }
    if let Err(error) = linux.preflight_assets() {
        eprintln!("Erro: {error}");
        return 1;
    }
    if let Err(error) = linux.apply(options.profile.as_deref()) {
        eprintln!("Erro: {error}");
        return 1;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_desktop_options() {
        assert_eq!(parse_args(&[]), Ok(Some(Options::default())));
        let options = parse_args(&["--profile".into(), "acer".into(), "--dry-run".into()])
            .unwrap()
            .unwrap();
        assert_eq!(options.profile.as_deref(), Some("acer"));
        assert!(options.dry_run);
        assert!(parse_args(&["--profile".into()]).is_err());
        assert!(parse_args(&["theme".into()]).is_err());
    }
}
