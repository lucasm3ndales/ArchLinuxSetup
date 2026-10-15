use crate::adapters::linux::LinuxInspector;
use crate::application::doctor;
use crate::application::pkg::{
    Category, NVIDIA_580XX_CONFLICTS, PackageSource, RustProvider, Selection,
};
use crate::system;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct LinuxPkg {
    root: PathBuf,
    home: PathBuf,
    state_home: Option<PathBuf>,
}

impl LinuxPkg {
    pub fn new(root: &Path, home: PathBuf, state_home: Option<PathBuf>) -> Self {
        Self {
            root: root.to_path_buf(),
            home,
            state_home,
        }
    }

    fn state_dir(&self) -> PathBuf {
        self.state_home
            .as_deref()
            .filter(|path| !path.as_os_str().is_empty())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| self.home.join(".local/state"))
            .join("archlinuxsetup")
    }

    pub fn helper(&self) -> Option<&'static str> {
        if system::command_exists("paru") {
            Some("paru")
        } else if system::command_exists("yay") {
            Some("yay")
        } else {
            None
        }
    }

    pub fn preflight(&self, selection: &Selection, profile: Option<&str>) -> Result<(), String> {
        if system::effective_uid()? == 0 {
            return Err("Execute como usuário comum, sem sudo.".to_owned());
        }
        let os_release = fs::read_to_string("/etc/os-release")
            .map_err(|_| "Não foi possível identificar a distribuição.".to_owned())?;
        if !os_release
            .lines()
            .any(|line| line == "ID=arch" || line == "ID=\"arch\"")
        {
            return Err("A instalação de pacotes só é permitida no Arch Linux.".to_owned());
        }
        for name in ["pacman", "sudo"] {
            if !system::command_exists(name) {
                return Err(format!("Comando ausente: {name}"));
            }
        }
        if selection.includes(Category::Development) {
            for name in ["getent", "groupadd", "usermod", "systemctl", "id"] {
                if !system::command_exists(name) {
                    return Err(format!("Comando ausente: {name}"));
                }
            }
        }
        if let Some(profile) = profile {
            doctor::check_profile(&LinuxInspector, &self.root, profile)
                .map_err(|error| format!("Aplicação recusada para este perfil: {error}"))?;
        }
        if selection.aur.iter().any(|name| name == "nvidia-580xx-dkms") {
            let conflicts = NVIDIA_580XX_CONFLICTS
                .iter()
                .copied()
                .filter(|name| self.installed(name))
                .collect::<Vec<_>>();
            if !conflicts.is_empty() {
                return Err(format!(
                    "Driver NVIDIA incompatível com a série 580 já instalado: {}. Revise a migração para nvidia-580xx-dkms antes de executar gaming; o ArchSetup não remove drivers automaticamente.",
                    conflicts.join(", ")
                ));
            }
            if !system::installed("linux")
                && !system::installed("linux-zen")
                && !system::installed("linux-lts")
                && !system::installed("linux-hardened")
            {
                return Err("Nenhum kernel Arch conhecido foi encontrado para instalar os headers exigidos pelo driver NVIDIA DKMS.".to_owned());
            }
        }
        if selection.includes(Category::Gaming) {
            if !system::command_exists("pacman-conf") {
                return Err("pacman-conf não encontrado.".to_owned());
            }
            let repos = capture("pacman-conf", &["--repo-list"])?;
            if !repos.lines().any(|repo| repo == "multilib") {
                return Err(
                    "Ative [multilib] em /etc/pacman.conf antes de instalar gaming.".to_owned(),
                );
            }
        }
        Ok(())
    }

    pub fn apply(&self, selection: &Selection, helper: Option<&str>) -> Result<(), String> {
        println!("\n==> Atualizando o sistema e instalando pacotes oficiais");
        run("sudo", &["-v"])?;
        let mut args = vec!["pacman", "-Syu", "--needed", "--noconfirm"];
        args.extend(selection.official.iter().map(String::as_str));
        run("sudo", &args)?;

        if selection.includes(Category::Development) {
            self.configure_docker()?;
        }
        if !selection.aur.is_empty() {
            let helper = helper.ok_or("auxiliar AUR indisponível após conferir a dependência")?;
            println!("\n==> Instalando pacotes AUR com {helper}");
            let mut args = vec!["-S", "--needed", "--noconfirm"];
            args.extend(selection.aur.iter().map(String::as_str));
            run(helper, &args)?;
        }
        if selection.includes(Category::Apps) {
            self.configure_browser()?;
        }
        if selection.rust_provider == RustProvider::Rustup {
            self.configure_rustup()?;
        }
        Ok(())
    }

    fn configure_docker(&self) -> Result<(), String> {
        println!("\n==> Conferindo acesso ao Docker");
        if success("systemctl", &["is-active", "--quiet", "ufw.service"])? {
            println!("Aviso: portas publicadas pelo Docker podem contornar regras do UFW. Revise a exposição dos containers.");
        }
        let user = capture("id", &["-un"])?.trim().to_owned();
        if user.is_empty() {
            return Err("não foi possível identificar o usuário".to_owned());
        }
        let groups = capture("id", &["-nG", &user])?;
        let in_group = groups.split_whitespace().any(|group| group == "docker");
        let enabled = success("systemctl", &["is-enabled", "--quiet", "docker.service"])?;
        let active = success("systemctl", &["is-active", "--quiet", "docker.service"])?;
        let state = self.state_dir().join("docker-before");
        if !state.exists() {
            fs::create_dir_all(self.state_dir())
                .map_err(|error| format!("não foi possível criar diretório de estado: {error}"))?;
            match OpenOptions::new().write(true).create_new(true).open(&state) {
                Ok(mut file) => writeln!(
                    file,
                    "{user}\n{}\n{}\n{}",
                    yes_no(in_group),
                    yes_no(enabled),
                    yes_no(active)
                )
                .map_err(|error| {
                    format!("não foi possível registrar o estado do Docker: {error}")
                })?,
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => {
                    return Err(format!(
                        "não foi possível criar {}: {error}",
                        state.display()
                    ));
                }
            }
        }
        if !success("getent", &["group", "docker"])? {
            run("sudo", &["groupadd", "--system", "docker"])?;
            println!("Grupo docker criado.");
        }
        if in_group {
            println!("Usuário {user} já pertence ao grupo docker.");
        } else {
            run("sudo", &["usermod", "-aG", "docker", &user])?;
            println!(
                "Usuário {user} adicionado ao grupo docker. Saia e entre novamente para ativar o acesso."
            );
        }
        if enabled {
            println!("Docker já está habilitado para iniciar no boot.");
        } else {
            run("sudo", &["systemctl", "enable", "docker.service"])?;
            println!("Docker habilitado para iniciar no boot.");
        }
        if active {
            println!("Docker já está em execução.");
        } else {
            run("sudo", &["systemctl", "start", "docker.service"])?;
            println!("Docker iniciado.");
        }
        Ok(())
    }

    fn configure_browser(&self) -> Result<(), String> {
        if !Path::new("/usr/share/applications/brave-browser.desktop").is_file() {
            return Ok(());
        }
        println!("\n==> Conferindo o navegador padrão");
        let graphical = std::env::var_os("DISPLAY").is_some_and(|value| !value.is_empty())
            || std::env::var_os("WAYLAND_DISPLAY").is_some_and(|value| !value.is_empty());
        if !system::command_exists("xdg-settings") || !graphical {
            println!(
                "Na sessão gráfica, execute: xdg-settings set default-web-browser brave-browser.desktop"
            );
            return Ok(());
        }
        let current = capture("xdg-settings", &["get", "default-web-browser"]).unwrap_or_default();
        if current.trim() == "brave-browser.desktop" {
            return Ok(());
        }
        let backup = self.state_dir().join("browser-before");
        if !backup.join("recorded").exists() {
            fs::create_dir_all(&backup)
                .map_err(|error| format!("não foi possível criar backup do navegador: {error}"))?;
            fs::write(
                backup.join("default-browser"),
                format!("{}\n", current.trim()),
            )
            .map_err(|error| format!("não foi possível registrar navegador anterior: {error}"))?;
            for label in [
                ".config/mimeapps.list",
                ".local/share/applications/mimeapps.list",
            ] {
                let source = self.home.join(label);
                if source.is_file() {
                    let destination = backup.join(label);
                    fs::create_dir_all(destination.parent().unwrap())
                        .map_err(|error| error.to_string())?;
                    fs::copy(&source, &destination).map_err(|error| {
                        format!("não foi possível copiar {}: {error}", source.display())
                    })?;
                }
            }
            fs::write(backup.join("recorded"), "").map_err(|error| {
                format!("não foi possível concluir backup do navegador: {error}")
            })?;
        }
        if !success(
            "xdg-settings",
            &["set", "default-web-browser", "brave-browser.desktop"],
        )? {
            println!("Defina o Brave como navegador padrão nas preferências do desktop.");
        }
        Ok(())
    }

    fn configure_rustup(&self) -> Result<(), String> {
        println!("\n==> Configurando Rust com rustup");
        let toolchains = capture("rustup", &["toolchain", "list"])?;
        if toolchains.contains("(default)") {
            println!("Toolchain padrão do Rust já configurada.");
        } else {
            println!("Instalando a toolchain estável do Rust.");
            run("rustup", &["default", "stable"])?;
        }
        Ok(())
    }
}

impl PackageSource for LinuxPkg {
    fn read_manifest(&self, path: &Path) -> Result<Option<String>, String> {
        match fs::read_to_string(path) {
            Ok(content) => Ok(Some(content)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(format!("não foi possível ler {}: {error}", path.display())),
        }
    }
    fn pacman_available(&self) -> bool {
        system::command_exists("pacman")
    }
    fn installed(&self, package: &str) -> bool {
        system::installed(package)
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn run(program: &str, args: &[&str]) -> Result<(), String> {
    io::stdout().flush().map_err(|error| error.to_string())?;
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|error| format!("não foi possível executar {program}: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "{program} falhou (código {}).",
            status.code().unwrap_or(1)
        ))
    }
}

fn success(program: &str, args: &[&str]) -> Result<bool, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| format!("não foi possível executar {program}: {error}"))?;
    Ok(output.status.success())
}

fn capture(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| format!("não foi possível executar {program}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "{program} falhou (código {}).",
            output.status.code().unwrap_or(1)
        ));
    }
    String::from_utf8(output.stdout).map_err(|_| format!("saída inválida de {program}"))
}
