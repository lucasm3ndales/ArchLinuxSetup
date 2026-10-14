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
                .map_err(|error| format!("não foi possível ler {}: {error}", path.display())),
            Ok(_) => Ok(LinkState::Occupied),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(LinkState::Missing),
            Err(error) => Err(format!("não foi possível verificar o comando: {error}")),
        }
    }

    fn read_optional(&self, path: &Path) -> Result<Option<Vec<u8>>, String> {
        match fs::read(path) {
            Ok(content) => Ok(Some(content)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(format!("não foi possível ler {}: {error}", path.display())),
        }
    }

    fn executable(&self, path: &Path) -> Result<bool, String> {
        match fs::metadata(path) {
            Ok(metadata) => Ok(metadata.is_file() && metadata.permissions().mode() & 0o111 != 0),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(format!(
                "não foi possível verificar {}: {error}",
                path.display()
            )),
        }
    }
}

impl FileSystem {
    pub fn apply(&self, plan: &Plan) -> Result<(), String> {
        if plan.create_link {
            if !self.executable(&plan.source)? {
                return Err(format!(
                    "binário Rust ausente em {}. Execute cargo build --release.",
                    plan.source.display()
                ));
            }
            fs::create_dir_all(plan.target.parent().expect("target tem diretório pai"))
                .map_err(|error| format!("não foi possível criar ~/.local/bin: {error}"))?;
            if plan.replace_legacy_link {
                if self.link_state(&plan.target)? != LinkState::Symlink(plan.legacy_source.clone())
                {
                    return Err("o link antigo mudou após a prévia; execute novamente".to_owned());
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
                    .map_err(|error| format!("não foi possível preparar o novo link: {error}"))?;
                if let Err(error) = fs::rename(&temporary, &plan.target) {
                    let _ = fs::remove_file(&temporary);
                    return Err(format!("não foi possível atualizar o comando: {error}"));
                }
            } else {
                symlink(&plan.source, &plan.target)
                    .map_err(|error| format!("não foi possível criar o comando: {error}"))?;
            }
        }

        if plan.append_path {
            let rc = plan.rc.as_ref().expect("append_path exige rc");
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
                .map_err(|error| format!("não foi possível abrir {}: {error}", rc.display()))?;
            writeln!(file, "\n{PATH_LINE}")
                .map_err(|error| format!("não foi possível atualizar {}: {error}", rc.display()))?;
            fs::create_dir_all(&plan.state_dir).map_err(|error| {
                format!("não foi possível criar o diretório de estado: {error}")
            })?;
            fs::write(
                plan.state_dir.join("path-rc"),
                format!("{}\n", rc.display()),
            )
            .map_err(|error| format!("não foi possível registrar o PATH: {error}"))?;
        }
        Ok(())
    }
}
