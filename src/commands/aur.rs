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
        _ => Err("aur accepts only --dry-run or --help".to_owned()),
    }
}

pub fn help() -> i32 {
    println!("Usage: archsetup aur [--dry-run]");
    println!("Installs the paru helper for AUR packages.");
    0
}

pub fn run(args: &[OsString]) -> i32 {
    let mode = match parse_args(args) {
        Ok(mode) => mode,
        Err(error) => {
            eprintln!("Error: {error}");
            return 2;
        }
    };
    if mode == Mode::Help {
        return help();
    }
    let linux = LinuxAur;
    let plan = aur::plan(&linux);
    println!(
        "==> {}",
        if mode == Mode::DryRun {
            "paru installation preview"
        } else {
            "Installing paru"
        }
    );
    if let Some(path) = &plan.paru {
        println!("paru is already available: {}", path.display());
        println!("Complete: no installation needed.");
        return 0;
    }
    match plan.rust {
        RustProvider::RustupInstalled => println!("Rust: rustup is already installed."),
        RustProvider::RustInstalled => {
            println!("Rust: rust is already installed; rustup will not be installed.")
        }
        RustProvider::InstallRustup => {
            println!("Rust: rustup will be installed and configured before building.")
        }
        RustProvider::Unknown => println!("Rust: state not checked (pacman unavailable)."),
    }
    println!("Steps:");
    println!("  1. Update Arch and install git, base-devel and a Rust tool if needed.");
    println!("  2. Clone https://aur.archlinux.org/paru.git into a temporary directory.");
    println!("  3. Show the PKGBUILD and list other files.");
    println!("  4. Build and install with makepkg -si --noconfirm as a regular user.");
    if mode == Mode::DryRun {
        println!("Preview complete. No changes made.");
        return 0;
    }
    if let Err(error) = linux.preflight() {
        eprintln!("Error: {error}");
        return 1;
    }
    let home = match std::env::var_os("HOME").filter(|home| !home.is_empty()) {
        Some(home) => PathBuf::from(home),
        None => {
            eprintln!("Error: HOME is not set.");
            return 1;
        }
    };
    let state_home = std::env::var_os("XDG_STATE_HOME").map(PathBuf::from);
    let tracker = match PackageTracker::begin(&home, state_home.as_deref()) {
        Ok(tracker) => tracker,
        Err(error) => {
            eprintln!("Error: {error}");
            return 1;
        }
    };
    let result = linux.apply();
    let tracked = tracker.finish();
    if let Ok(count) = &tracked {
        if *count > 0 {
            println!("New packages tracked for possible removal: {count}");
        }
    }
    if let Err(error) = result {
        eprintln!("Error: {error}");
        if let Err(tracking_error) = tracked {
            eprintln!("Warning: package tracking is incomplete: {tracking_error}");
        }
        return 1;
    }
    if let Err(error) = tracked {
        eprintln!("Error: installation finished, but package tracking failed: {error}");
        return 1;
    }
    println!("\nComplete: paru installed. Check packages with archsetup pkg --dry-run.");
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
