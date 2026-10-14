use crate::adapters::aur::LinuxAur;
use crate::adapters::package_state::PackageTracker;
use crate::application::aur::{self, RustProvider};
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, Eq, PartialEq)]
enum Mode {
    Apply,
    DryRun,
    Help,
}

fn parse_args(args: &[OsString]) -> Result<Mode, String> {
    match args {
        [] => Ok(Mode::Apply),
        [option] if option == "--dry-run" => Ok(Mode::DryRun),
        [option] if option == "-h" || option == "--help" => Ok(Mode::Help),
        _ => Err("aur aceita apenas --dry-run ou --help".to_owned()),
    }
}

pub fn help() -> i32 {
    println!("Uso: archsetup aur [--dry-run]");
    println!("Instala o auxiliar paru para pacotes da AUR.");
    0
}

pub fn run(args: &[OsString]) -> i32 {
    let mode = match parse_args(args) {
        Ok(mode) => mode,
        Err(error) => {
            eprintln!("Erro: {error}");
            return 2;
        }
    };
    if mode == Mode::Help {
        return help();
    }
    let linux = LinuxAur;
    let plan = aur::plan(&linux);
    println!(
        "==> {} do paru",
        if mode == Mode::DryRun {
            "Prévia da instalação"
        } else {
            "Instalação"
        }
    );
    if let Some(path) = &plan.paru {
        println!("paru já está disponível: {}", path.display());
        println!("Concluído: nenhuma instalação necessária.");
        return 0;
    }
    match plan.rust {
        RustProvider::RustupInstalled => println!("Rust: rustup já instalado."),
        RustProvider::RustInstalled => {
            println!("Rust: rust já instalado; rustup não será instalado.")
        }
        RustProvider::InstallRustup => {
            println!("Rust: rustup será instalado e configurado antes da compilação.")
        }
        RustProvider::Unknown => println!("Rust: estado não verificado (pacman indisponível)."),
    }
    println!("Etapas:");
    println!(
        "  1. Atualizar o Arch e instalar git, base-devel e uma ferramenta Rust, se necessário."
    );
    println!("  2. Clonar https://aur.archlinux.org/paru.git em diretório temporário.");
    println!("  3. Mostrar o PKGBUILD e listar os demais arquivos.");
    println!("  4. Compilar e instalar com makepkg -si --noconfirm como usuário comum.");
    if mode == Mode::DryRun {
        println!("Prévia concluída. Nenhuma alteração realizada.");
        return 0;
    }
    if let Err(error) = linux.preflight() {
        eprintln!("Erro: {error}");
        return 1;
    }
    let home = match std::env::var_os("HOME").filter(|home| !home.is_empty()) {
        Some(home) => PathBuf::from(home),
        None => {
            eprintln!("Erro: HOME não está definido.");
            return 1;
        }
    };
    let state_home = std::env::var_os("XDG_STATE_HOME").map(PathBuf::from);
    let tracker = match PackageTracker::begin(&home, state_home.as_deref()) {
        Ok(tracker) => tracker,
        Err(error) => {
            eprintln!("Erro: {error}");
            return 1;
        }
    };
    let result = linux.apply();
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
    println!("\nConcluído: paru instalado. Confira os pacotes com archsetup pkg --dry-run.");
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_aur_flags() {
        assert_eq!(parse_args(&[]), Ok(Mode::Apply));
        assert_eq!(parse_args(&["--dry-run".into()]), Ok(Mode::DryRun));
        assert_eq!(parse_args(&["--help".into()]), Ok(Mode::Help));
        assert!(parse_args(&["--dry-run".into(), "extra".into()]).is_err());
    }
}
