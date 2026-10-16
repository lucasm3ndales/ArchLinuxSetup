use crate::adapters::linux::LinuxInspector;
use crate::application::doctor;
use crate::application::pkg::{
    self, Category, NVIDIA_580XX_CONFLICTS, PackageSource, RustProvider, Selection,
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
            return Err("Run as a regular user, without sudo.".to_owned());
        }
        let os_release = fs::read_to_string("/etc/os-release")
            .map_err(|_| "Could not identify the distribution.".to_owned())?;
        if !os_release
            .lines()
            .any(|line| line == "ID=arch" || line == "ID=\"arch\"")
        {
            return Err("Packages can only be installed on Arch Linux.".to_owned());
        }
        for name in ["pacman", "sudo"] {
            if !system::command_exists(name) {
                return Err(format!("Missing command: {name}"));
            }
        }
        if selection.includes(Category::Development) {
            for name in ["getent", "groupadd", "usermod", "systemctl", "id"] {
                if !system::command_exists(name) {
                    return Err(format!("Missing command: {name}"));
                }
            }
        }
        if let Some(profile) = profile {
            doctor::check_profile(&LinuxInspector, &self.root, profile)
                .map_err(|error| format!("Cannot apply this profile: {error}"))?;
        }
        if selection.aur.iter().any(|name| name == "nvidia-580xx-dkms") {
            let conflicts = NVIDIA_580XX_CONFLICTS
                .iter()
                .copied()
                .filter(|name| self.installed(name))
                .collect::<Vec<_>>();
            if !conflicts.is_empty() {
                return Err(format!(
                    "Installed NVIDIA driver incompatible with the 580 series: {}. Review the migration to nvidia-580xx-dkms before installing gaming; ArchSetup does not remove drivers automatically.",
                    conflicts.join(", ")
                ));
            }
            if !system::installed("linux")
                && !system::installed("linux-zen")
                && !system::installed("linux-lts")
                && !system::installed("linux-hardened")
            {
                return Err("No known Arch kernel was found for installing the headers required by the NVIDIA DKMS driver.".to_owned());
            }
        }
        if selection.includes(Category::Gaming) {
            if !system::command_exists("pacman-conf") {
                return Err("pacman-conf not found.".to_owned());
            }
            let repos = capture("pacman-conf", &["--repo-list"])?;
            if !repos.lines().any(|repo| repo == "multilib") {
                return Err(
                    "Enable [multilib] in /etc/pacman.conf before installing gaming.".to_owned(),
                );
            }
        }
        Ok(())
    }

    pub fn apply(&self, selection: &Selection, helper: Option<&str>) -> Result<(), String> {
        let missing_official =
            pkg::missing_packages(&selection.official, |name| self.installed(name));
        if missing_official.is_empty() {
            println!("\n==> Official packages are already installed; skipping system update.");
        } else {
            println!("\n==> Updating the system and installing missing official packages");
            run("sudo", &["-v"])?;
            let mut args = vec!["pacman", "-Syu", "--needed", "--noconfirm"];
            args.extend(missing_official);
            run("sudo", &args)?;
        }

        if selection.includes(Category::Development) {
            self.configure_docker()?;
        }
        let missing_aur = pkg::missing_packages(&selection.aur, |name| self.installed(name));
        if !missing_aur.is_empty() {
            let helper = helper.ok_or("AUR helper unavailable after checking the dependency")?;
            println!("\n==> Installing missing AUR packages with {helper}");
            let mut args = vec!["-S", "--needed", "--noconfirm"];
            args.extend(missing_aur);
            run(helper, &args)?;
        } else {
            println!("\n==> AUR packages are already installed.");
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
        println!("\n==> Checking Docker access");
        if success("systemctl", &["is-active", "--quiet", "ufw.service"])? {
            println!(
                "Warning: Docker-published ports may bypass UFW rules. Review container exposure."
            );
        }
        let user = capture("id", &["-un"])?.trim().to_owned();
        if user.is_empty() {
            return Err("could not identify the user".to_owned());
        }
        let groups = capture("id", &["-nG", &user])?;
        let in_group = groups.split_whitespace().any(|group| group == "docker");
        let enabled = success("systemctl", &["is-enabled", "--quiet", "docker.service"])?;
        let active = success("systemctl", &["is-active", "--quiet", "docker.service"])?;
        let state = self.state_dir().join("docker-before");
        if !state.exists() {
            fs::create_dir_all(self.state_dir())
                .map_err(|error| format!("could not create state directory: {error}"))?;
            match OpenOptions::new().write(true).create_new(true).open(&state) {
                Ok(mut file) => writeln!(
                    file,
                    "{user}\n{}\n{}\n{}",
                    yes_no(in_group),
                    yes_no(enabled),
                    yes_no(active)
                )
                .map_err(|error| format!("could not record Docker state: {error}"))?,
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => {
                    return Err(format!("could not create {}: {error}", state.display()));
                }
            }
        }
        if !success("getent", &["group", "docker"])? {
            run("sudo", &["groupadd", "--system", "docker"])?;
            println!("docker group created.");
        }
        if in_group {
            println!("User {user} is already in the docker group.");
        } else {
            run("sudo", &["usermod", "-aG", "docker", &user])?;
            println!(
                "User {user} added to the docker group. Log out and back in to activate access."
            );
        }
        if enabled {
            println!("Docker is already enabled at boot.");
        } else {
            run("sudo", &["systemctl", "enable", "docker.service"])?;
            println!("Docker enabled at boot.");
        }
        if active {
            println!("Docker is already running.");
        } else {
            run("sudo", &["systemctl", "start", "docker.service"])?;
            println!("Docker started.");
        }
        Ok(())
    }

    fn configure_browser(&self) -> Result<(), String> {
        if !Path::new("/usr/share/applications/brave-browser.desktop").is_file() {
            return Ok(());
        }
        println!("\n==> Checking the default browser");
        let graphical = std::env::var_os("DISPLAY").is_some_and(|value| !value.is_empty())
            || std::env::var_os("WAYLAND_DISPLAY").is_some_and(|value| !value.is_empty());
        if !system::command_exists("xdg-settings") || !graphical {
            println!(
                "In the graphical session, run: xdg-settings set default-web-browser brave-browser.desktop"
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
                .map_err(|error| format!("could not back up browser settings: {error}"))?;
            fs::write(
                backup.join("default-browser"),
                format!("{}\n", current.trim()),
            )
            .map_err(|error| format!("could not record the previous browser: {error}"))?;
            for label in [
                ".config/mimeapps.list",
                ".local/share/applications/mimeapps.list",
            ] {
                let source = self.home.join(label);
                if source.is_file() {
                    let destination = backup.join(label);
                    fs::create_dir_all(destination.parent().unwrap())
                        .map_err(|error| error.to_string())?;
                    fs::copy(&source, &destination)
                        .map_err(|error| format!("could not copy {}: {error}", source.display()))?;
                }
            }
            fs::write(backup.join("recorded"), "")
                .map_err(|error| format!("could not finish browser backup: {error}"))?;
        }
        if !success(
            "xdg-settings",
            &["set", "default-web-browser", "brave-browser.desktop"],
        )? {
            println!("Set Brave as the default browser in desktop preferences.");
        }
        Ok(())
    }

    fn configure_rustup(&self) -> Result<(), String> {
        println!("\n==> Configuring Rust with rustup");
        let toolchains = capture("rustup", &["toolchain", "list"])?;
        if toolchains.contains("(default)") {
            println!("Default Rust toolchain is already configured.");
        } else {
            println!("Installing the stable Rust toolchain.");
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
            Err(error) => Err(format!("could not read {}: {error}", path.display())),
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
        .map_err(|error| format!("could not run {program}: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "{program} failed (exit code {}).",
            status.code().unwrap_or(1)
        ))
    }
}

fn success(program: &str, args: &[&str]) -> Result<bool, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| format!("could not run {program}: {error}"))?;
    Ok(output.status.success())
}

fn capture(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| format!("could not run {program}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "{program} failed (exit code {}).",
            output.status.code().unwrap_or(1)
        ));
    }
    String::from_utf8(output.stdout).map_err(|_| format!("invalid output from {program}"))
}
