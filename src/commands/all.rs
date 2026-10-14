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
                let name = args.get(index).ok_or("falta o nome após --profile")?;
                let name = name.to_string_lossy();
                if !project::valid_profile_name(&name) {
                    return Err(format!("nome de perfil inválido: {name}"));
                }
                options.profile = Some(name.into_owned());
            }
            "-h" | "--help" => return Ok(None),
            other => return Err(format!("opção desconhecida em all: {other}")),
        }
        index += 1;
    }
    Ok(Some(options))
}

pub fn help() -> i32 {
    println!("Uso: archsetup all [--profile NOME] [--dry-run]");
    println!("Executa aur, pkg e desktop (atualmente só Xfce) nessa ordem.");
    println!("Requer Xfce instalado e uma sessão X11 do Xfce antes de começar.");
    println!("Para no primeiro erro.");
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
            eprintln!("Erro: perfil inexistente: {profile}");
            return 2;
        }
    }
    let desktop_system = match LinuxDesktop::new(root) {
        Ok(system) => system,
        Err(error) => {
            eprintln!("Erro: {error}");
            return 1;
        }
    };
    if let Err(error) = desktop_system.check_xfce_environment() {
        eprintln!("Erro: {error}");
        return 1;
    }
    if !options.dry_run {
        let report = doctor::inspect(&LinuxInspector, root, options.profile.as_deref());
        if report.failed {
            eprintln!(
                "Erro: checagem inicial falhou. Execute archsetup doctor para ver os detalhes."
            );
            return 1;
        }
    }

    println!(
        "==> {} completa do ArchSetup",
        if options.dry_run {
            "Prévia"
        } else {
            "Configuração"
        }
    );
    for (index, command) in [Command::Aur, Command::Pkg, Command::Desktop]
        .into_iter()
        .enumerate()
    {
        println!("\n==> Etapa {}/3: {}", index + 1, command_name(command));
        let args = stage_args(command, &options);
        let status = match command {
            Command::Aur => aur::run(&args),
            Command::Pkg => pkg::run(root, &args),
            Command::Desktop => desktop::run(root, &args),
            _ => unreachable!("somente etapas do all"),
        };
        if status != 0 {
            eprintln!(
                "Erro: etapa {} falhou (código {status}).",
                command_name(command)
            );
            return status;
        }
    }
    println!("\nConcluído: aur, pkg e desktop processados.");
    0
}

fn command_name(command: Command) -> &'static str {
    match command {
        Command::Aur => "aur",
        Command::Pkg => "pkg",
        Command::Desktop => "desktop",
        _ => unreachable!("somente etapas do all"),
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
