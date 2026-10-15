use crate::adapters::unlink::FileSystem;
use crate::application::unlink;
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
    let mut dry_run = false;
    for option in args {
        match option.to_string_lossy().as_ref() {
            "--dry-run" => dry_run = true,
            "-h" | "--help" => return Ok(Mode::Help),
            other => return Err(format!("unknown option for unlink: {other}")),
        }
    }
    Ok(if dry_run { Mode::DryRun } else { Mode::Apply })
}

pub fn help() -> i32 {
    println!("Usage: archsetup unlink [--dry-run]");
    println!("Removes the command link and the PATH entry recorded by ArchSetup.");
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
    match unlink_command(root, mode == Mode::DryRun) {
        Ok(()) => 0,
        Err(message) => {
            eprintln!("Error: {message}");
            1
        }
    }
}

fn unlink_command(root: &Path, dry_run: bool) -> Result<(), String> {
    if system::effective_uid()? == 0 {
        return Err("Run as a regular user, without sudo.".to_owned());
    }
    let home = std::env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set".to_owned())?;
    let state_home = std::env::var_os("XDG_STATE_HOME").map(PathBuf::from);
    let fs = FileSystem;
    let plan = unlink::plan(&fs, root, &home, state_home.as_deref())?;

    println!("==> Removing the archsetup command");
    if plan.remove_link {
        println!("Command: remove {}", plan.target.display());
    } else {
        println!("Command: link is already absent.");
    }
    if let (Some(rc), Some(_)) = (&plan.rc, &plan.updated_rc) {
        println!("PATH: remove the entry added to {}", rc.display());
    }
    if dry_run {
        println!("Preview complete. No changes made.");
        return Ok(());
    }

    fs.apply(&plan)?;
    if let (Some(rc), Some(_)) = (&plan.rc, &plan.updated_rc) {
        println!("PATH entry removed from {}.", rc.display());
    }
    if plan.remove_link {
        println!("Command removed.");
    }
    println!(
        "Complete. Backups preserved at {}.",
        plan.backup_base.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_unlink_flags() {
        assert_eq!(parse_args(&[]), Ok(Mode::Apply));
        assert_eq!(parse_args(&["--dry-run".into()]), Ok(Mode::DryRun));
        assert_eq!(parse_args(&["--help".into()]), Ok(Mode::Help));
        assert!(parse_args(&["--packages".into()]).is_err());
    }
}
