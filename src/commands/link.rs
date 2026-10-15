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
            "unknown option for link: {}",
            option.to_string_lossy()
        )),
        _ => Err("link does not accept extra arguments".to_owned()),
    }
}

pub fn help() -> i32 {
    println!("Usage: archsetup link [--dry-run]");
    println!("Links the Rust binary in ~/.local/bin and configures PATH for Bash or Zsh.");
    println!("Build first with cargo build --release.");
    0
}

pub fn run(root: &Path, args: &[OsString]) -> i32 {
    let mode = match parse_args(args) {
        Ok(mode) => mode,
        Err(message) => {
            eprintln!("Error: {message}");
            return 2;
        }
    };
    if mode == Mode::Help {
        return help();
    }
    match link_command(root, mode == Mode::DryRun) {
        Ok(()) => 0,
        Err(message) => {
            eprintln!("Error: {message}");
            1
        }
    }
}

fn link_command(root: &Path, dry_run: bool) -> Result<(), String> {
    if system::effective_uid()? == 0 {
        return Err("run as a regular user, without sudo".to_owned());
    }
    let home = std::env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set".to_owned())?;
    let shell = std::env::var_os("SHELL").unwrap_or_else(|| OsString::from("/bin/bash"));
    let state_home = std::env::var_os("XDG_STATE_HOME").map(PathBuf::from);
    let fs = FileSystem;
    let plan = link::plan(&fs, root, &home, &shell, state_home.as_deref())?;

    println!("==> Creating the archsetup command");
    println!("Source: {}", plan.source.display());
    println!("Destination: {}", plan.target.display());
    if plan.append_path {
        println!(
            "Persistent PATH entry: {}",
            plan.rc.as_ref().expect("rc is set").display()
        );
    }
    if dry_run {
        println!("Preview complete. No changes made.");
        return Ok(());
    }
    fs.apply(&plan)?;
    if plan.create_link {
        if plan.replace_legacy_link {
            println!("Old link updated to point to the Rust binary.");
        } else {
            println!("Command created.");
        }
    } else {
        println!("The command already points to this project.");
    }
    if let Some(rc) = &plan.rc {
        if plan.append_path {
            println!(
                "PATH configured in {}. Open a new terminal to use archsetup.",
                rc.display()
            );
        } else {
            println!("PATH is already configured in {}.", rc.display());
        }
    } else {
        println!("Add ~/.local/bin to your shell PATH to use archsetup as a command.");
    }
    println!("Complete: archsetup command available from this repository.");
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
