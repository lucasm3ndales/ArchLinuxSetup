use crate::adapters::desktop::LinuxDesktop;
use crate::adapters::linux::LinuxInspector;
use crate::application::doctor;
use crate::cli::Command;
use crate::project;
use std::ffi::OsString;
use std::path::Path;

use super::{aur, desktop, pkg};

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
                let name = args.get(index).ok_or("missing name after --profile")?;
                let name = name.to_string_lossy();
                if !project::valid_profile_name(&name) {
                    return Err(format!("invalid profile name: {name}"));
                }
                options.profile = Some(name.into_owned());
            }
            "-h" | "--help" => return Ok(None),
            other => return Err(format!("unknown option for all: {other}")),
        }
        index += 1;
    }
    Ok(Some(options))
}

pub fn help() -> i32 {
    println!("Usage: archsetup all [--profile NAME] [--dry-run]");
    println!("Runs aur, pkg and desktop (currently Xfce only), in that order.");
    println!("Requires Xfce and an active Xfce X11 session before starting.");
    println!("Stops at the first error.");
    0
}

pub fn run(root: &Path, args: &[OsString]) -> i32 {
    let options = match parse_args(args) {
        Ok(Some(options)) => options,
        Ok(None) => return help(),
        Err(error) => {
            eprintln!("Error: {error}");
            return 2;
        }
    };
    if let Some(profile) = &options.profile {
        if !root.join("profiles").join(profile).is_dir() {
            eprintln!("Error: unknown profile: {profile}");
            return 2;
        }
    }
    let desktop_system = match LinuxDesktop::new(root) {
        Ok(system) => system,
        Err(error) => {
            eprintln!("Error: {error}");
            return 1;
        }
    };
    if let Err(error) = desktop_system.check_xfce_environment() {
        eprintln!("Error: {error}");
        return 1;
    }
    if !options.dry_run {
        let report = doctor::inspect(&LinuxInspector, root, options.profile.as_deref());
        if report.failed {
            eprintln!("Error: initial check failed. Run archsetup doctor for details.");
            return 1;
        }
    }

    println!(
        "==> Full ArchSetup {}",
        if options.dry_run {
            "Preview"
        } else {
            "Configuration"
        }
    );
    for (index, command) in [Command::Aur, Command::Pkg, Command::Desktop]
        .into_iter()
        .enumerate()
    {
        println!("\n==> Step {}/3: {}", index + 1, command_name(command));
        let args = stage_args(command, &options);
        let status = match command {
            Command::Aur => aur::run(&args),
            Command::Pkg => pkg::run(root, &args),
            Command::Desktop => desktop::run(root, &args),
            _ => unreachable!("all steps only"),
        };
        if status != 0 {
            eprintln!(
                "Error: step {} failed (exit code {status}).",
                command_name(command)
            );
            return status;
        }
    }
    println!("\nComplete: aur, pkg and desktop completed.");
    0
}

fn command_name(command: Command) -> &'static str {
    match command {
        Command::Aur => "aur",
        Command::Pkg => "pkg",
        Command::Desktop => "desktop",
        _ => unreachable!("all steps only"),
    }
}

fn stage_args(command: Command, options: &Options) -> Vec<OsString> {
    let mut args = Vec::new();
    if options.dry_run {
        args.push("--dry-run".into());
    }
    if command != Command::Aur {
        if let Some(profile) = &options.profile {
            args.push("--profile".into());
            args.push(profile.into());
        }
    }
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forwards_options_only_to_relevant_stages() {
        let options = parse_args(&["--profile".into(), "acer".into(), "--dry-run".into()])
            .unwrap()
            .unwrap();
        assert_eq!(
            stage_args(Command::Aur, &options),
            vec![OsString::from("--dry-run")]
        );
        assert_eq!(
            stage_args(Command::Pkg, &options),
            vec![
                OsString::from("--dry-run"),
                OsString::from("--profile"),
                OsString::from("acer")
            ]
        );
        assert_eq!(
            stage_args(Command::Desktop, &options),
            stage_args(Command::Pkg, &options)
        );
        assert!(parse_args(&["--profile".into(), "../bad".into()]).is_err());
    }
}
