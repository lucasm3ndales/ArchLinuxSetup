mod root_files;
mod xfce;

use crate::adapters::linux::LinuxInspector;
use crate::application::{desktop, doctor};
use crate::system;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct LinuxDesktop {
    root: PathBuf,
    home: PathBuf,
    state_dir: PathBuf,
    config_dir: PathBuf,
}

impl LinuxDesktop {
    pub fn new(root: &Path) -> Result<Self, String> {
        let home = env::var_os("HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .ok_or("HOME não está definido")?;
        let state_home = env::var_os("XDG_STATE_HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local/state"));
        let config_dir = env::var_os("XDG_CONFIG_HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".config"));
        Ok(Self {
            root: root.to_path_buf(),
            home,
            state_dir: state_home.join("archlinuxsetup"),
            config_dir,
        })
    }

    pub fn pacman_available(&self) -> bool {
        system::command_exists("pacman")
    }

    pub fn missing_packages(&self) -> Option<Vec<&'static str>> {
        self.pacman_available().then(|| {
            desktop::REQUIRED_PACKAGES
                .iter()
                .copied()
                .filter(|name| !system::installed(name))
                .collect()
        })
    }

    pub fn check_xfce_environment(&self) -> Result<(), String> {
        let session_type = env::var("XDG_SESSION_TYPE").ok();
        let current_desktop = env::var("XDG_CURRENT_DESKTOP").ok();
        desktop::check_xfce(desktop::XfceFacts {
            session_type: session_type.as_deref(),
            current_desktop: current_desktop.as_deref(),
            display_available: env::var_os("DISPLAY").is_some_and(|value| !value.is_empty()),
            xfce_session_installed: system::installed("xfce4-session"),
            xfce_panel_installed: system::installed("xfce4-panel"),
        })
    }

    pub fn uses_lightdm(&self) -> bool {
        fs::canonicalize("/etc/systemd/system/display-manager.service")
            .is_ok_and(|path| path.ends_with("lightdm.service"))
    }

    pub fn uses_ly(&self) -> bool {
        success("systemctl", &["is-enabled", "--quiet", "ly.service"]).unwrap_or(false)
            || (1..=6).any(|tty| {
                ["ly", "ly-kmsconvt"].iter().any(|name| {
                    let service = format!("{name}@tty{tty}.service");
                    success("systemctl", &["is-enabled", "--quiet", &service]).unwrap_or(false)
                })
            })
    }

    pub fn preflight(&self, profile: Option<&str>) -> Result<(), String> {
        if system::effective_uid()? == 0 {
            return Err("Execute como usuário comum, sem sudo.".to_owned());
        }
        let os_release = fs::read_to_string("/etc/os-release")
            .map_err(|_| "Não foi possível identificar a distribuição.".to_owned())?;
        if !os_release
            .lines()
            .any(|line| line == "ID=arch" || line == "ID=\"arch\"")
        {
            return Err("A configuração do desktop só é permitida no Arch Linux.".to_owned());
        }
        self.check_xfce_environment()?;
        for name in [
            "sudo",
            "pacman",
            "xfconf-query",
            "xfce4-panel",
            "pgrep",
            "sha256sum",
            "date",
            "mv",
            "cp",
            "cmp",
            "install",
        ] {
            if !system::command_exists(name) {
                return Err(format!("Comando necessário ausente: {name}"));
            }
        }
        if let Some(profile) = profile {
            doctor::check_profile(&LinuxInspector, &self.root, profile)
                .map_err(|error| format!("Aplicação recusada para este perfil: {error}"))?;
        }
        if !system::installed("xorg-server") {
            return Err(
                "xorg-server ausente; configure a sessão X11 do Xfce antes de aplicar.".to_owned(),
            );
        }
        if self.uses_lightdm() {
            if !system::command_exists("lightdm") {
                return Err("LightDM habilitado, mas o comando lightdm está ausente.".to_owned());
            }
            if !Path::new("/usr/share/xgreeters/lightdm-slick-greeter.desktop").is_file() {
                return Err(
                    "LightDM habilitado, mas Slick Greeter está ausente.".to_owned(),
                );
            }
        } else if self.uses_ly() {
            if !system::command_exists("ly-dm") {
                return Err("Ly habilitado, mas ly-dm não foi encontrado.".to_owned());
            }
            if !Path::new("/etc/ly/config.ini").is_file() {
                return Err("Ly habilitado, mas /etc/ly/config.ini está ausente.".to_owned());
            }
        }
        for path in [
            "/usr/share/xfce4/panel/plugins/xfce4-verve-plugin.desktop",
            "/usr/share/xfce4/panel/plugins/applicationsmenu.desktop",
            "/usr/share/pixmaps/archlinux-logo.svg",
        ] {
            if !Path::new(path).is_file() {
                return Err(format!("Arquivo necessário ausente: {path}"));
            }
        }
        let alt = maybe_output(
            "xfconf-query",
            &[
                "-c",
                "xfce4-keyboard-shortcuts",
                "-p",
                "/commands/custom/<Alt>F1",
            ],
        );
        if !alt.as_deref().is_none_or(|value| {
            value.is_empty()
                || value == "xfce4-popup-applicationsmenu"
                || value == "xfce4-popup-whiskermenu"
        }) {
            return Err(
                "Alt+F1 já executa outro comando; revise o atalho no Xfce antes de aplicar."
                    .to_owned(),
            );
        }
        let super_key = maybe_output(
            "xfconf-query",
            &[
                "-c",
                "xfce4-keyboard-shortcuts",
                "-p",
                "/commands/custom/Super_L",
            ],
        );
        if super_key.as_deref().is_some_and(|value| !value.is_empty()) {
            return Err(
                "Super_L já tem um atalho próprio no Xfce; remova o conflito antes de aplicar."
                    .to_owned(),
            );
        }
        let mut root_paths = vec![
            "/etc/X11/xorg.conf.d/90-archlinuxsetup-touchpad.conf",
            "/etc/X11/xorg.conf.d/91-archlinuxsetup-acer-tapping.conf",
        ];
        if self.uses_lightdm() {
            root_paths.extend([
                "/etc/lightdm/slick-greeter.conf",
                "/etc/lightdm/lightdm.conf.d/90-archsetup-slick.conf",
            ]);
        } else if self.uses_ly() {
            root_paths.push("/etc/ly/config.ini");
        }
        for path in root_paths {
            let path = Path::new(path);
            match fs::symlink_metadata(path) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err(format!(
                        "Não vou substituir um link simbólico: {}",
                        path.display()
                    ));
                }
                Ok(metadata) if !metadata.is_file() => {
                    return Err(format!(
                        "Caminho não é um arquivo comum: {}",
                        path.display()
                    ));
                }
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(format!(
                        "não foi possível inspecionar {}: {error}",
                        path.display()
                    ));
                }
            }
        }
        for directory in [self.home.join(".icons"), self.home.join(".icons/default")] {
            match fs::symlink_metadata(&directory) {
                Ok(metadata) if !metadata.is_dir() || metadata.file_type().is_symlink() => {
                    return Err(format!(
                        "Diretório de cursores inválido: {}",
                        directory.display()
                    ));
                }
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(format!(
                        "não foi possível inspecionar {}: {error}",
                        directory.display()
                    ));
                }
            }
        }
        for config_file in [
            self.config_dir.join("autostart/ksuperkey.desktop"),
            self.home.join(".Xresources"),
            self.home.join(".icons/default/index.theme"),
        ] {
            match fs::symlink_metadata(&config_file) {
                Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
                    return Err(format!(
                        "Configuração não é um arquivo comum: {}",
                        config_file.display()
                    ));
                }
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(format!(
                        "não foi possível inspecionar {}: {error}",
                        config_file.display()
                    ));
                }
            }
            if config_file.is_file()
                && config_file != self.config_dir.join("autostart/ksuperkey.desktop")
            {
                fs::read_to_string(&config_file).map_err(|error| {
                    format!("não foi possível ler {}: {error}", config_file.display())
                })?;
            }
        }
        if self.uses_lightdm() && Path::new("/etc/lightdm/slick-greeter.conf").is_file() {
            fs::read_to_string("/etc/lightdm/slick-greeter.conf").map_err(|_| {
                "Não é possível ler a configuração existente do Slick Greeter.".to_owned()
            })?;
        } else if self.uses_ly() {
            fs::read_to_string("/etc/ly/config.ini")
                .map_err(|_| "Não é possível ler a configuração existente do Ly.".to_owned())?;
        }
        self.panel_plan()?;
        Ok(())
    }

    pub fn preflight_assets(&self) -> Result<(), String> {
        if !system::command_exists("ksuperkey") {
            return Err("ksuperkey não encontrado; execute archsetup pkg desktop.".to_owned());
        }
        if let Some(missing) = self.missing_packages() {
            if !missing.is_empty() {
                return Err(format!(
                    "Pacotes necessários ausentes: {}",
                    missing.join(" ")
                ));
            }
        }
        for (path, directory) in [
            ("/usr/share/themes/Nordic/gtk-3.0", true),
            ("/usr/share/themes/Nordic/xfwm4", true),
            ("/usr/share/icons/zafiro-dark/index.theme", false),
            ("/usr/share/icons/Hackneyed/cursors", true),
        ] {
            let present = if directory {
                Path::new(path).is_dir()
            } else {
                Path::new(path).is_file()
            };
            if !present {
                return Err(format!(
                    "Tema, ícones ou cursor ausente: {path}. Execute archsetup pkg theme."
                ));
            }
        }
        if !Path::new("/usr/share/xfce4/panel/plugins/kbdleds.desktop").is_file() {
            return Err("Plugin Keyboard LEDs ausente; execute archsetup pkg desktop.".to_owned());
        }
        let plan = self.panel_plan()?;
        if !system::command_exists(plan.menu_command) {
            return Err(format!("Comando do menu ausente: {}", plan.menu_command));
        }
        Ok(())
    }

    pub fn apply(&self, profile: Option<&str>) -> Result<(), String> {
        let tapping = profile
            .map(|name| {
                doctor::read_profile(&LinuxInspector, &self.root, name)
                    .map(|data| data.touchpad_tapping)
            })
            .transpose()?
            .unwrap_or(false);
        run("sudo", &["-v"])?;
        println!("\n==> Configurando Xfce");
        self.configure_xfce()?;
        if self.uses_lightdm() {
            println!("\n==> Configurando tela de login LightDM/Slick");
            self.configure_greeter()?;
        } else if self.uses_ly() {
            println!("\n==> Configurando tela de login Ly");
            self.configure_ly()?;
        } else {
            println!("\n==> Login: gerenciador atual preservado (sem ajustes de greeter).");
        }
        println!("\n==> Configurando touchpad");
        self.configure_touchpad(tapping)?;
        println!("\nConcluído: desktop configurado.");
        Ok(())
    }
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

fn maybe_output(program: &str, args: &[&str]) -> Option<String> {
    capture(program, args)
        .ok()
        .map(|value| value.trim_end().to_owned())
}

fn success(program: &str, args: &[&str]) -> Result<bool, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| format!("não foi possível executar {program}: {error}"))?;
    Ok(output.status.success())
}
