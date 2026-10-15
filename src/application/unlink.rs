use crate::application::link::{LinkState, PATH_LINE};
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};

/// Observations needed to decide which files ArchSetup owns.
pub trait UnlinkReader {
    fn link_state(&self, path: &Path) -> Result<LinkState, String>;
    fn read_optional(&self, path: &Path) -> Result<Option<Vec<u8>>, String>;
    fn historical_backups(&self, rc: &Path) -> Result<Vec<Vec<u8>>, String>;
}

#[derive(Debug, Eq, PartialEq)]
pub struct Plan {
    pub source: PathBuf,
    pub legacy_source: PathBuf,
    pub target: PathBuf,
    pub remove_link: bool,
    pub rc: Option<PathBuf>,
    pub updated_rc: Option<Vec<u8>>,
    pub original_rc: Option<Vec<u8>>,
    pub backup_base: PathBuf,
}

pub fn plan(
    reader: &impl UnlinkReader,
    root: &Path,
    home: &Path,
    state_home: Option<&Path>,
) -> Result<Plan, String> {
    let state_dir = state_home
        .filter(|path| !path.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| home.join(".local/state"))
        .join("archlinuxsetup");
    let target = home.join(".local/bin/archsetup");
    let source = root.join("target/release/archsetup");
    let legacy_source = root.join("archsetup");
    let remove_link = match reader.link_state(&target)? {
        LinkState::Missing => false,
        LinkState::Symlink(path) if path == source || path == legacy_source => true,
        LinkState::Symlink(_) | LinkState::Occupied => {
            return Err(format!(
                "{} exists but does not belong to this repository.",
                target.display()
            ));
        }
    };

    let rc = if let Some(record) = reader.read_optional(&state_dir.join("path-rc"))? {
        let first_line = record
            .split(|byte| *byte == b'\n')
            .next()
            .unwrap_or_default();
        let recorded = PathBuf::from(OsString::from_vec(first_line.to_vec()));
        if recorded != home.join(".bashrc") && recorded != home.join(".zshrc") {
            return Err("Invalid PATH record.".to_owned());
        }
        Some(recorded)
    } else {
        find_owned_rc(reader, home)?
    };

    let original_rc = if let Some(rc) = &rc {
        reader.read_optional(rc)?
    } else {
        None
    };
    let updated_rc = original_rc
        .as_ref()
        .filter(|content| contains_line(content, PATH_LINE.as_bytes()))
        .map(|content| without_line(content, PATH_LINE.as_bytes()));

    Ok(Plan {
        source,
        legacy_source,
        target,
        remove_link,
        rc,
        updated_rc,
        original_rc,
        backup_base: state_dir.join("backups"),
    })
}

fn find_owned_rc(reader: &impl UnlinkReader, home: &Path) -> Result<Option<PathBuf>, String> {
    for rc in [home.join(".bashrc"), home.join(".zshrc")] {
        let Some(content) = reader.read_optional(&rc)? else {
            continue;
        };
        if !contains_line(&content, PATH_LINE.as_bytes()) {
            continue;
        }
        if reader
            .historical_backups(&rc)?
            .iter()
            .any(|backup| !contains_line(backup, PATH_LINE.as_bytes()))
        {
            return Ok(Some(rc));
        }
    }
    Ok(None)
}

fn contains_line(content: &[u8], line: &[u8]) -> bool {
    content
        .split(|byte| *byte == b'\n')
        .any(|part| part == line)
}

fn without_line(content: &[u8], line: &[u8]) -> Vec<u8> {
    content
        .split_inclusive(|byte| *byte == b'\n')
        .filter(|part| part.strip_suffix(b"\n").unwrap_or(part) != line)
        .flat_map(|part| part.iter().copied())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Default)]
    struct FakeReader {
        link: Option<PathBuf>,
        occupied: bool,
        files: HashMap<PathBuf, Vec<u8>>,
        backups: HashMap<PathBuf, Vec<Vec<u8>>>,
    }

    impl UnlinkReader for FakeReader {
        fn link_state(&self, _: &Path) -> Result<LinkState, String> {
            if self.occupied {
                Ok(LinkState::Occupied)
            } else {
                Ok(self
                    .link
                    .clone()
                    .map(LinkState::Symlink)
                    .unwrap_or(LinkState::Missing))
            }
        }
        fn read_optional(&self, path: &Path) -> Result<Option<Vec<u8>>, String> {
            Ok(self.files.get(path).cloned())
        }
        fn historical_backups(&self, rc: &Path) -> Result<Vec<Vec<u8>>, String> {
            Ok(self.backups.get(rc).cloned().unwrap_or_default())
        }
    }

    #[test]
    fn removes_only_owned_link_and_recorded_path_line() {
        let root = Path::new("/project");
        let home = Path::new("/home/dev");
        let mut fake = FakeReader {
            link: Some(root.join("target/release/archsetup")),
            ..Default::default()
        };
        fake.files.insert(
            home.join(".local/state/archlinuxsetup/path-rc"),
            b"/home/dev/.bashrc\n".to_vec(),
        );
        let original = format!("# settings\n{PATH_LINE}\nalias ll='ls -l'\n").into_bytes();
        fake.files.insert(home.join(".bashrc"), original.clone());
        let plan = plan(&fake, root, home, None).unwrap();
        assert!(plan.remove_link);
        assert_eq!(plan.original_rc, Some(original));
        assert_eq!(
            plan.updated_rc,
            Some(b"# settings\nalias ll='ls -l'\n".to_vec())
        );
    }

    #[test]
    fn historical_backup_proves_path_ownership_without_record() {
        let root = Path::new("/project");
        let home = Path::new("/home/dev");
        let mut fake = FakeReader::default();
        let rc = home.join(".zshrc");
        fake.files
            .insert(rc.clone(), format!("{PATH_LINE}\n").into_bytes());
        fake.backups
            .insert(rc.clone(), vec![b"# original\n".to_vec()]);
        let plan = plan(&fake, root, home, None).unwrap();
        assert_eq!(plan.rc, Some(rc));
        assert_eq!(plan.updated_rc, Some(Vec::new()));
    }

    #[test]
    fn legacy_launcher_link_can_still_be_removed() {
        let root = Path::new("/project");
        let home = Path::new("/home/dev");
        let fake = FakeReader {
            link: Some(root.join("archsetup")),
            ..Default::default()
        };
        assert!(plan(&fake, root, home, None).unwrap().remove_link);
    }

    #[test]
    fn refuses_foreign_link_and_invalid_record() {
        let root = Path::new("/project");
        let home = Path::new("/home/dev");
        let fake = FakeReader {
            link: Some(PathBuf::from("/other/archsetup")),
            ..Default::default()
        };
        assert!(plan(&fake, root, home, None).is_err());
        let mut fake = FakeReader::default();
        fake.files.insert(
            home.join(".local/state/archlinuxsetup/path-rc"),
            b"/tmp/foreign\n".to_vec(),
        );
        assert!(plan(&fake, root, home, None).is_err());
    }
}
