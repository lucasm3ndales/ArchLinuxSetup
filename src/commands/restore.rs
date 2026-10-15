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
            _ => return Err(format!("Unknown option: {}", arg.to_string_lossy())),
        }
    }
    Ok(options)
}

pub fn help() -> i32 {
    println!("Usage: archsetup restore [--packages] [--dry-run]");
    println!();
    println!("Restores previously backed-up settings without removing the archsetup command.");
    println!("--packages removes only packages tracked as new by ArchSetup.");
    println!("--dry-run previews actions without changing the system.");
    println!("Run outside the Xfce graphical session to restore its files.");
    0
}

pub fn run(args: &[OsString]) -> i32 {
    let options = match parse_args(args) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("Error: {message}");
            return 2;
        }
    };
    if options.help {
        return help();
    }
    match restore_command(options) {
        Ok(()) => 0,
        Err(message) => {
            eprintln!("Error: {message}");
            1
        }
    }
}

fn restore_command(options: Options) -> Result<(), String> {
    if system::effective_uid()? == 0 {
        return Err("Run as a regular user, without sudo.".to_owned());
    }
    let home = std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or("HOME is not set")?;
    let config_home = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from);
    let state_home = std::env::var_os("XDG_STATE_HOME").map(PathBuf::from);
    let adapter = LinuxRestore::new(home, config_home, state_home);
    let xfce_backup = adapter.oldest_xfce_backup()?;
    println!("==> ArchSetup restore");
    if let Some(backup) = &xfce_backup {
        println!(
            "Xfce: restore previous configuration from {}",
            backup.display()
        );
        println!("Cursor: restore .Xresources and .icons/default/index.theme if a backup exists.");
    } else {
        println!("Xfce: no previous backup found; nothing will change.");
    }
    println!("Ly and touchpad: restore previous backups when available.");
    println!("Docker and browser: restore the previous state when tracked.");
    let packages = if options.packages {
        let packages = adapter.installed_packages()?;
        println!("Tracked packages still installed: {}", packages.len());
        println!(
            "The list includes newly installed dependencies and may include packages you used later."
        );
        println!("Record: {}", adapter.manifest().display());
        for package in &packages {
            println!("  {package}");
        }
        Some(packages)
    } else {
        None
    };
    if options.dry_run {
        println!("Preview complete. No changes made.");
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
