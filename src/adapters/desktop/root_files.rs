use super::{LinuxDesktop, capture, run, success};
use crate::application::desktop;
use std::fs::{self, DirBuilder};
use std::io;
use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

impl LinuxDesktop {
    pub(super) fn configure_greeter(&self) -> Result<(), String> {
        let config = Path::new("/etc/lightdm/slick-greeter.conf");
        let seat = Path::new("/etc/lightdm/lightdm.conf.d/90-archsetup-slick.conf");
        let original = match fs::read_to_string(config) {
            Ok(content) => content,
            Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
            Err(error) => {
                return Err(format!(
                    "não foi possível ler {}: {error}",
                    config.display()
                ));
            }
        };
        let work = TempWorkDir::new()?;
        run(
            "sudo",
            &[
                "install",
                "-d",
                "-m",
                "755",
                "--",
                "/etc/lightdm/lightdm.conf.d",
            ],
        )?;
        self.update_root_file(
            config,
            &desktop::render_greeter(&original),
            "archsetup",
            &work,
        )?;
        self.update_root_file(seat, desktop::SEAT_CONFIG, "archsetup", &work)?;
        println!(
            "Concluído. A nova aparência aparecerá no próximo login; a sessão atual continua aberta."
        );
        Ok(())
    }

    pub(super) fn configure_touchpad(&self, tapping: bool) -> Result<(), String> {
        let work = TempWorkDir::new()?;
        let natural = Path::new("/etc/X11/xorg.conf.d/90-archlinuxsetup-touchpad.conf");
        if self.update_root_file(natural, desktop::NATURAL_SCROLL, "archlinuxsetup", &work)? {
            println!("Rolagem natural configurada para a próxima sessão X11.");
        } else {
            println!("Rolagem natural já configurada; nenhuma alteração necessária.");
        }
        if tapping {
            let target = Path::new("/etc/X11/xorg.conf.d/91-archlinuxsetup-acer-tapping.conf");
            if self.update_root_file(target, desktop::TAPPING, "archlinuxsetup", &work)? {
                println!("Toque para clicar configurado para a próxima sessão X11.");
            } else {
                println!("Toque para clicar já configurado; nenhuma alteração necessária.");
            }
        }
        println!("Concluído: configurações do touchpad processadas.");
        Ok(())
    }

    fn update_root_file(
        &self,
        target: &Path,
        content: &str,
        backup_prefix: &str,
        work: &TempWorkDir,
    ) -> Result<bool, String> {
        let metadata = match fs::symlink_metadata(target) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!(
                    "Não vou substituir um link simbólico: {}",
                    target.display()
                ));
            }
            Ok(metadata) if !metadata.is_file() => {
                return Err(format!(
                    "Caminho não é um arquivo comum: {}",
                    target.display()
                ));
            }
            Ok(metadata) => Some(metadata),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => {
                return Err(format!(
                    "não foi possível inspecionar {}: {error}",
                    target.display()
                ));
            }
        };
        let name = target
            .file_name()
            .ok_or("nome de arquivo de configuração inválido")?;
        let source = work.path.join(name);
        fs::write(&source, content)
            .map_err(|error| format!("não foi possível preparar {}: {error}", target.display()))?;
        let source_text = source.to_string_lossy();
        let target_text = target.to_string_lossy();
        if metadata.is_some() && success("sudo", &["cmp", "-s", "--", &source_text, &target_text])?
        {
            println!("Já configurado: {}", target.display());
            return Ok(false);
        }
        if metadata.is_some() {
            let date = capture("date", &["+%Y%m%d%H%M%S%N"])?.trim().to_owned();
            let backup = format!("{}.{}.bak-{date}", target.display(), backup_prefix);
            run("sudo", &["cp", "-p", "--", &target_text, &backup])?;
            println!("Backup: {backup}");
        }
        run(
            "sudo",
            &[
                "install",
                "-D",
                "-m",
                "644",
                "--",
                &source_text,
                &target_text,
            ],
        )?;
        let marker_dir = self.state_dir.join("root-created");
        let marker = marker_dir.join(name);
        if metadata.is_none() || marker.is_file() {
            let digest = capture("sha256sum", &["--", &source_text])?;
            let hash = digest
                .split_whitespace()
                .next()
                .ok_or("sha256sum não retornou hash")?;
            fs::create_dir_all(&marker_dir).map_err(|error| {
                format!("não foi possível criar {}: {error}", marker_dir.display())
            })?;
            fs::write(&marker, format!("{hash} {}\n", target.display())).map_err(|error| {
                format!("não foi possível registrar {}: {error}", marker.display())
            })?;
        }
        println!("Atualizado: {}", target.display());
        Ok(true)
    }
}

struct TempWorkDir {
    path: PathBuf,
}

impl TempWorkDir {
    fn new() -> Result<Self, String> {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos();
        for attempt in 0..10 {
            let path = std::env::temp_dir().join(format!(
                "archsetup-desktop-{}-{stamp}-{attempt}",
                std::process::id()
            ));
            match DirBuilder::new().mode(0o700).create(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(format!(
                        "não foi possível criar diretório temporário: {error}"
                    ));
                }
            }
        }
        Err("não foi possível reservar um diretório temporário".to_owned())
    }
}

impl Drop for TempWorkDir {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.path) {
            eprintln!(
                "Aviso: não foi possível remover {}: {error}",
                self.path.display()
            );
        }
    }
}
