use crate::adapters::file_backup;
use crate::adapters::link::FileSystem as LinkFileSystem;
use crate::application::link::{LinkReader, LinkState};
use crate::application::unlink::{Plan, UnlinkReader};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct FileSystem;

impl UnlinkReader for FileSystem {
    fn link_state(&self, path: &Path) -> Result<LinkState, String> {
        LinkReader::link_state(&LinkFileSystem, path)
    }

    fn read_optional(&self, path: &Path) -> Result<Option<Vec<u8>>, String> {
        LinkReader::read_optional(&LinkFileSystem, path)
    }

    fn historical_backups(&self, rc: &Path) -> Result<Vec<Vec<u8>>, String> {
        let parent = rc.parent().ok_or("shell file has no parent directory")?;
        let name = rc
            .file_name()
            .ok_or("shell file has no name")?
            .to_string_lossy();
        let prefix = format!("{name}.archsetup.bak-");
        let entries = fs::read_dir(parent)
            .map_err(|error| format!("could not list {}: {error}", parent.display()))?;
        let mut paths = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|error| error.to_string())?;
            if entry.file_name().to_string_lossy().starts_with(&prefix) && entry.path().is_file() {
                paths.push(entry.path());
            }
        }
        paths.sort();
        paths
            .into_iter()
            .map(|path| {
                fs::read(&path)
                    .map_err(|error| format!("could not read {}: {error}", path.display()))
            })
            .collect()
    }
}

impl FileSystem {
    pub fn apply(&self, plan: &Plan) -> Result<(), String> {
        // Reject changes made since the plan was prepared before touching either file.
        if plan.remove_link {
            let state = self.link_state(&plan.target)?;
            if state != LinkState::Symlink(plan.source.clone())
                && state != LinkState::Symlink(plan.legacy_source.clone())
            {
                return Err(
                    "the command link changed after the preview; run the command again".to_owned(),
                );
            }
        }
        if let (Some(rc), Some(original)) = (&plan.rc, &plan.original_rc) {
            if plan.updated_rc.is_some() && self.read_optional(rc)?.as_deref() != Some(original) {
                return Err(format!(
                    "{} changed after the preview; run the command again",
                    rc.display()
                ));
            }
        }

        if let (Some(rc), Some(updated)) = (&plan.rc, &plan.updated_rc) {
            let backup = backup_path(&plan.backup_base, rc)?;
            fs::create_dir_all(backup.parent().expect("backup has a parent directory"))
                .map_err(|error| format!("could not create backup directory: {error}"))?;
            file_backup::copy(rc, &backup)?;
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(rc)
                .map_err(|error| format!("could not open {}: {error}", rc.display()))?;
            file.write_all(updated)
                .map_err(|error| format!("could not update {}: {error}", rc.display()))?;
        }
        if plan.remove_link {
            fs::remove_file(&plan.target)
                .map_err(|error| format!("could not remove {}: {error}", plan.target.display()))?;
        }
        Ok(())
    }
}

fn backup_path(base: &Path, rc: &Path) -> Result<PathBuf, String> {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    let name = rc.file_name().ok_or("shell file has no name")?;
    Ok(base
        .join(format!("unlink-{suffix}-{}", std::process::id()))
        .join(name))
}
