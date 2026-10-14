use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

pub fn effective_uid() -> Result<u32, String> {
    let status = fs::read_to_string("/proc/self/status")
        .map_err(|error| format!("não foi possível identificar o usuário: {error}"))?;
    let uid_line = status
        .lines()
        .find(|line| line.starts_with("Uid:"))
        .ok_or_else(|| "UID não encontrado em /proc/self/status".to_owned())?;
    uid_line
        .split_whitespace()
        .nth(2)
        .ok_or_else(|| "UID efetivo indisponível".to_owned())?
        .parse::<u32>()
        .map_err(|error| format!("UID efetivo inválido: {error}"))
}

pub fn command_exists(name: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| {
        let path = dir.join(name);
        path.metadata()
            .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
    })
}

pub fn output(name: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(name).args(args).output().ok()?;
    output.status.success().then(|| {
        String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_owned()
    })
}

pub fn output_any_status(name: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(name).args(args).output().ok()?;
    let value = String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_owned();
    (!value.is_empty()).then_some(value)
}

pub fn installed(package: &str) -> bool {
    Command::new("pacman")
        .args(["-Qq", package])
        .output()
        .is_ok_and(|output| output.status.success())
}

pub fn read_trimmed(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_owned())
}

pub fn run_optional(name: &str, args: &[&str]) {
    if !command_exists(name) {
        println!("Comando indisponível: {name}");
        return;
    }
    let _ = io::stdout().flush();
    match Command::new(name).args(args).status() {
        Ok(status) if status.success() => {}
        _ => println!("Não foi possível executar: {name}"),
    }
}
