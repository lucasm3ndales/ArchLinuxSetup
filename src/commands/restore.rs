use crate::adapters::restore::LinuxRestore;
use crate::system;
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, Eq, PartialEq)]
struct Options {
    packages: bool,
    dry_run: bool,
    help: bool,
}

fn parse_args(args: &[OsString]) -> Result<Options, String> {
    let mut options = Options {
        packages: false,
        dry_run: false,
        help: false,
    };
    for arg in args {
        match arg.to_str() {
            Some("--packages") => options.packages = true,
            Some("--dry-run") => options.dry_run = true,
            Some("-h" | "--help") => options.help = true,
            _ => return Err(format!("Opção desconhecida: {}", arg.to_string_lossy())),
        }
    }
    Ok(options)
}

pub fn help() -> i32 {
    println!("Uso: archsetup restore [--packages] [--dry-run]");
    println!();
    println!("Recupera configurações com backup anterior sem remover o comando archsetup.");
    println!("--packages remove somente pacotes registrados como novos pelo ArchSetup.");
    println!("--dry-run mostra as ações sem alterar o sistema.");
    println!("Execute fora da sessão gráfica do Xfce para restaurar seus arquivos.");
    0
}

pub fn run(args: &[OsString]) -> i32 {
    let options = match parse_args(args) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("Erro: {message}");
            return 2;
        }
    };
    if options.help {
        return help();
    }
    match restore_command(options) {
        Ok(()) => 0,
        Err(message) => {
            eprintln!("Erro: {message}");
            1
        }
    }
}

fn restore_command(options: Options) -> Result<(), String> {
    if system::effective_uid()? == 0 {
        return Err("Execute como usuário comum, sem sudo.".to_owned());
    }
    let home = std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or("HOME não está definido")?;
    let config_home = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from);
    let state_home = std::env::var_os("XDG_STATE_HOME").map(PathBuf::from);
    let adapter = LinuxRestore::new(home, config_home, state_home);
    let xfce_backup = adapter.oldest_xfce_backup()?;
    println!("==> Restauração do ArchSetup");
    if let Some(backup) = &xfce_backup {
        println!(
            "Xfce: restaurar configuração anterior de {}",
            backup.display()
        );
        println!("Cursor: restaurar .Xresources e .icons/default/index.theme se houver backup.");
    } else {
        println!("Xfce: nenhum backup anterior encontrado; nada será alterado.");
    }
    println!("LightDM e touchpad: restaurar backups anteriores quando disponíveis.");
    println!("Docker e navegador: restaurar estado anterior quando registrado.");
    let packages = if options.packages {
        let packages = adapter.installed_packages()?;
        println!("Pacotes registrados e ainda instalados: {}", packages.len());
        println!(
            "A lista inclui dependências novas da instalação e pode incluir pacotes usados depois por você."
        );
        println!("Registro: {}", adapter.manifest().display());
        for package in &packages {
            println!("  {package}");
        }
        Some(packages)
    } else {
        None
    };
    if options.dry_run {
        println!("Prévia concluída. Nenhuma alteração realizada.");
        return Ok(());
    }
    adapter.apply(xfce_backup.as_deref(), packages.as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_restore_flags() {
        assert_eq!(
            parse_args(&[]).unwrap(),
            Options {
                packages: false,
                dry_run: false,
                help: false
            }
        );
        assert_eq!(
            parse_args(&["--packages".into(), "--dry-run".into()]).unwrap(),
            Options {
                packages: true,
                dry_run: true,
                help: false
            }
        );
        assert!(parse_args(&["--remove-everything".into()]).is_err());
    }
}
