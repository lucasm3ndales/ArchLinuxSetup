use super::{LinuxDesktop, capture, run, success};
use crate::application::desktop;
use std::fs::{self, DirBuilder};
use std::io;
use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

impl LinuxDesktop {
    pub(super) fn configure_ly(&self) -> Result<(), String> {
        let config = Path::new("/etc/ly/config.ini");
        let original = fs::read_to_string(config)
            .map_err(|error| format!("could not read {}: {error}", config.display()))?;
        let work = TempWorkDir::new()?;
        self.update_root_file(config, &desktop::render_ly(&original), "archsetup", &work)?;
        println!("Ly is configured to show DOOM fire animation and date/time at the next login.");
        Ok(())
    }

    pub(super) fn configure_touchpad(&self, tapping: bool) -> Result<(), String> {
        let work = TempWorkDir::new()?;
        let natural = Path::new("/etc/X11/xorg.conf.d/90-archlinuxsetup-touchpad.conf");
        if self.update_root_file(natural, desktop::NATURAL_SCROLL, "archlinuxsetup", &work)? {
            println!("Natural scrolling will take effect in the next X11 session.");
        } else {
            println!("Natural scrolling is already configured; no changes needed.");
        }
        if tapping {
            let target = Path::new("/etc/X11/xorg.conf.d/91-archlinuxsetup-acer-tapping.conf");
            if self.update_root_file(target, desktop::TAPPING, "archlinuxsetup", &work)? {
                println!("Tap-to-click will take effect in the next X11 session.");
            } else {
                println!("Tap-to-click is already configured; no changes needed.");
            }
        }
        println!("Complete: touchpad settings applied.");
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
                    "Will not replace a symbolic link: {}",
                    target.display()
                ));
            }
            Ok(metadata) if !metadata.is_file() => {
                return Err(format!("Path is not a regular file: {}", target.display()));
            }
            Ok(metadata) => Some(metadata),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => {
                return Err(format!("could not inspect {}: {error}", target.display()));
            }
        };
        let name = target.file_name().ok_or("invalid configuration filename")?;
        let source = work.path.join(name);
        fs::write(&source, content)
            .map_err(|error| format!("could not prepare {}: {error}", target.display()))?;
        let source_text = source.to_string_lossy();
        let target_text = target.to_string_lossy();
        let unchanged = if metadata.is_some() {
            match fs::read(target) {
                Ok(current) => current == content.as_bytes(),
                Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {
                    success("sudo", &["cmp", "-s", "--", &source_text, &target_text])?
                }
                Err(error) => {
                    return Err(format!("could not read {}: {error}", target.display()));
                }
            }
        } else {
            false
        };
        if unchanged {
            println!("Already configured: {}", target.display());
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
                .ok_or("sha256sum returned no hash")?;
            fs::create_dir_all(&marker_dir)
                .map_err(|error| format!("could not create {}: {error}", marker_dir.display()))?;
            fs::write(&marker, format!("{hash} {}\n", target.display()))
                .map_err(|error| format!("could not record {}: {error}", marker.display()))?;
        }
        println!("Updated: {}", target.display());
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
                    return Err(format!("could not create temporary directory: {error}"));
                }
            }
        }
        Err("could not reserve a temporary directory".to_owned())
    }
}

impl Drop for TempWorkDir {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.path) {
            eprintln!("Warning: could not remove {}: {error}", self.path.display());
        }
    }
}
