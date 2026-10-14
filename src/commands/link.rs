use crate::adapters::link::FileSystem;
use crate::application::link;
use crate::system;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

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
        [option] => Err(format!(
            "opção desconhecida em link: {}",
            option.to_string_lossy()
        )),
        _ => Err("link não aceita argumentos extras".to_owned()),
    }
}

pub fn help() -> i32 {
    println!("Uso: archsetup link [--dry-run]");
    println!("Cria o link para o binário Rust em ~/.local/bin e configura o PATH do Bash ou Zsh.");
    println!("Compile antes com cargo build --release.");
    0
}

pub fn run(root: &Path, args: &[OsString]) -> i32 {
    let mode = match parse_args(args) {
        Ok(mode) => mode,
        Err(message) => {
            eprintln!("Erro: {message}");
            return 2;
        }
    };
    if mode == Mode::Help {
        return help();
    }
    match link_command(root, mode == Mode::DryRun) {
        Ok(()) => 0,
        Err(message) => {
            eprintln!("Erro: {message}");
            1
        }
    }
}

fn link_command(root: &Path, dry_run: bool) -> Result<(), String> {
    if system::effective_uid()? == 0 {
        return Err("execute como usuário comum, sem sudo".to_owned());
    }
    let home = std::env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| "HOME não está definido".to_owned())?;
    let shell = std::env::var_os("SHELL").unwrap_or_else(|| OsString::from("/bin/bash"));
    let state_home = std::env::var_os("XDG_STATE_HOME").map(PathBuf::from);
    let fs = FileSystem;
    let plan = link::plan(&fs, root, &home, &shell, state_home.as_deref())?;

    println!("==> Criando o comando archsetup");
    println!("Origem: {}", plan.source.display());
    println!("Destino: {}", plan.target.display());
    if plan.append_path {
        println!(
            "PATH persistente: {}",
            plan.rc.as_ref().expect("rc definido").display()
        );
    }
    if dry_run {
        println!("Prévia concluída. Nenhuma alteração realizada.");
        return Ok(());
    }
    fs.apply(&plan)?;
    if plan.create_link {
        if plan.replace_legacy_link {
            println!("Link antigo atualizado para o binário Rust.");
        } else {
            println!("Comando criado.");
        }
    } else {
        println!("O comando já aponta para este projeto.");
    }
    if let Some(rc) = &plan.rc {
        if plan.append_path {
            println!(
                "PATH configurado em {}. Abra um novo terminal para usar archsetup.",
                rc.display()
            );
        } else {
            println!("PATH já configurado em {}.", rc.display());
        }
    } else {
        println!("Inclua ~/.local/bin no PATH do seu shell para usar archsetup como comando.");
    }
    println!("Concluído: comando archsetup disponível a partir deste repositório.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_link_flags() {
        assert_eq!(parse_args(&[]), Ok(Mode::Apply));
        assert_eq!(parse_args(&["--dry-run".into()]), Ok(Mode::DryRun));
        assert_eq!(parse_args(&["--help".into()]), Ok(Mode::Help));
        assert!(parse_args(&["--dry-run".into(), "extra".into()]).is_err());
    }
}
