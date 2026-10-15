use crate::adapters::file_backup;
use crate::application::link::{LinkReader, LinkState, PATH_LINE, Plan};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct FileSystem;

impl LinkReader for FileSystem {
    fn link_state(&self, path: &Path) -> Result<LinkState, String> {
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.file_type().is_symlink() => fs::read_link(path)
                .map(LinkState::Symlink)
                .map_err(|error| format!("could not read {}: {error}", path.display())),
            Ok(_) => Ok(LinkState::Occupied),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(LinkState::Missing),
            Err(error) => Err(format!("could not verify the command: {error}")),
        }
    }

    fn read_optional(&self, path: &Path) -> Result<Option<Vec<u8>>, String> {
        match fs::read(path) {
            Ok(content) => Ok(Some(content)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(format!("could not read {}: {error}", path.display())),
        }
    }

    fn executable(&self, path: &Path) -> Result<bool, String> {
        match fs::metadata(path) {
            Ok(metadata) => Ok(metadata.is_file() && metadata.permissions().mode() & 0o111 != 0),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(format!("could not verify {}: {error}", path.display())),
        }
    }
}

impl FileSystem {
    pub fn apply(&self, plan: &Plan) -> Result<(), String> {
        if plan.create_link {
            if !self.executable(&plan.source)? {
                return Err(format!(
                    "Rust binary is missing at {}. Run cargo build --release.",
                    plan.source.display()
                ));
            }
            fs::create_dir_all(plan.target.parent().expect("target has a parent directory"))
                .map_err(|error| format!("could not create ~/.local/bin: {error}"))?;
            if plan.replace_legacy_link {
                if self.link_state(&plan.target)? != LinkState::Symlink(plan.legacy_source.clone())
                {
                    return Err(
                        "the old link changed after the preview; run the command again".to_owned(),
                    );
                }
                let temporary = plan.target.with_file_name(format!(
                    "archsetup.tmp-{}-{}",
                    std::process::id(),
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map_err(|error| error.to_string())?
                        .as_nanos()
                ));
                symlink(&plan.source, &temporary)
                    .map_err(|error| format!("could not prepare the new link: {error}"))?;
                if let Err(error) = fs::rename(&temporary, &plan.target) {
                    let _ = fs::remove_file(&temporary);
                    return Err(format!("could not update the command: {error}"));
                }
            } else {
                symlink(&plan.source, &plan.target)
                    .map_err(|error| format!("could not create the command: {error}"))?;
            }
        }

        if plan.append_path {
            let rc = plan.rc.as_ref().expect("append_path requires rc");
            if plan.backup_rc {
                let suffix = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|error| error.to_string())?
                    .as_nanos();
                let backup = rc.with_file_name(format!(
                    "{}.archsetup.bak-{suffix}",
                    rc.file_name().unwrap_or_default().to_string_lossy()
                ));
                file_backup::copy(rc, &backup)?;
            }
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(rc)
                .map_err(|error| format!("could not open {}: {error}", rc.display()))?;
            writeln!(file, "\n{PATH_LINE}")
                .map_err(|error| format!("could not update {}: {error}", rc.display()))?;
            fs::create_dir_all(&plan.state_dir)
                .map_err(|error| format!("could not create the state directory: {error}"))?;
            fs::write(
                plan.state_dir.join("path-rc"),
                format!("{}\n", rc.display()),
            )
            .map_err(|error| format!("could not record PATH: {error}"))?;
        }
        Ok(())
    }
}
