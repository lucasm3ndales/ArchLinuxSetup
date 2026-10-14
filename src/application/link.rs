use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub const PATH_LINE: &str = "export PATH=\"$HOME/.local/bin:$PATH\"";

#[derive(Debug, Eq, PartialEq)]
pub enum LinkState {
    Missing,
    Symlink(PathBuf),
    Occupied,
}

/// Read operations needed to plan the command link. Writes belong to the adapter.
pub trait LinkReader {
    fn link_state(&self, path: &Path) -> Result<LinkState, String>;
    fn read_optional(&self, path: &Path) -> Result<Option<Vec<u8>>, String>;
    fn executable(&self, path: &Path) -> Result<bool, String>;
}

#[derive(Debug, Eq, PartialEq)]
pub struct Plan {
    pub source: PathBuf,
    pub legacy_source: PathBuf,
    pub target: PathBuf,
    pub create_link: bool,
    pub replace_legacy_link: bool,
    pub rc: Option<PathBuf>,
    pub append_path: bool,
    pub backup_rc: bool,
    pub state_dir: PathBuf,
}

pub fn plan(
    reader: &impl LinkReader,
    root: &Path,
    home: &Path,
    shell: &OsStr,
    state_home: Option<&Path>,
) -> Result<Plan, String> {
    let source = root.join("target/release/archsetup");
    let legacy_source = root.join("archsetup");
    if !reader.executable(&source)? {
        return Err(format!(
            "binário Rust ausente em {}. Execute cargo build --release.",
            source.display()
        ));
    }
    let target = home.join(".local/bin/archsetup");
    let (create_link, replace_legacy_link) = match reader.link_state(&target)? {
        LinkState::Missing => (true, false),
        LinkState::Symlink(path) if path == source => (false, false),
        LinkState::Symlink(path) if path == legacy_source => (true, true),
        LinkState::Symlink(_) | LinkState::Occupied => {
            return Err(format!(
                "{} já existe e não aponta para este projeto",
                target.display()
            ));
        }
    };

    let rc = match Path::new(shell).file_name().and_then(|name| name.to_str()) {
        Some("bash") => Some(home.join(".bashrc")),
        Some("zsh") => Some(home.join(".zshrc")),
        _ => None,
    };
    let (append_path, rc_exists) = if let Some(rc) = &rc {
        match reader.read_optional(rc)? {
            Some(contents) => (!contains_line(&contents, PATH_LINE.as_bytes()), true),
            None => (true, false),
        }
    } else {
        (false, false)
    };
    let backup_rc = append_path && rc_exists;
    let state_dir = state_home
        .filter(|path| !path.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| home.join(".local/state"))
        .join("archlinuxsetup");
    Ok(Plan {
        source,
        legacy_source,
        target,
        create_link,
        replace_legacy_link,
        rc,
        append_path,
        backup_rc,
        state_dir,
    })
}

fn contains_line(content: &[u8], line: &[u8]) -> bool {
    content
        .split(|byte| *byte == b'\n')
        .any(|part| part == line)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeReader {
        link: LinkState,
        rc: Option<Vec<u8>>,
    }

    impl LinkReader for FakeReader {
        fn link_state(&self, _: &Path) -> Result<LinkState, String> {
            Ok(match &self.link {
                LinkState::Missing => LinkState::Missing,
                LinkState::Symlink(path) => LinkState::Symlink(path.clone()),
                LinkState::Occupied => LinkState::Occupied,
            })
        }

        fn read_optional(&self, _: &Path) -> Result<Option<Vec<u8>>, String> {
            Ok(self.rc.clone())
        }

        fn executable(&self, _: &Path) -> Result<bool, String> {
            Ok(true)
        }
    }

    #[test]
    fn plans_first_install_and_repeat_without_duplicate_path() {
        let root = Path::new("/project");
        let home = Path::new("/home/dev");
        let first = plan(
            &FakeReader {
                link: LinkState::Missing,
                rc: Some(b"# shell\n".to_vec()),
            },
            root,
            home,
            OsStr::new("/bin/bash"),
            None,
        )
        .unwrap();
        assert!(first.create_link && first.append_path && first.backup_rc);
        assert_eq!(first.rc, Some(home.join(".bashrc")));

        let repeat = plan(
            &FakeReader {
                link: LinkState::Symlink(root.join("target/release/archsetup")),
                rc: Some(format!("# shell\n{PATH_LINE}\n").into_bytes()),
            },
            root,
            home,
            OsStr::new("/bin/bash"),
            None,
        )
        .unwrap();
        assert!(!repeat.create_link && !repeat.append_path);
    }

    #[test]
    fn upgrades_legacy_launcher_link() {
        let root = Path::new("/project");
        let plan = plan(
            &FakeReader {
                link: LinkState::Symlink(root.join("archsetup")),
                rc: None,
            },
            root,
            Path::new("/home/dev"),
            OsStr::new("bash"),
            None,
        )
        .unwrap();
        assert!(plan.create_link && plan.replace_legacy_link);
        assert_eq!(plan.source, root.join("target/release/archsetup"));
    }

    #[test]
    fn refuses_occupied_target_before_any_write() {
        let result = plan(
            &FakeReader {
                link: LinkState::Occupied,
                rc: None,
            },
            Path::new("/project"),
            Path::new("/home/dev"),
            OsStr::new("bash"),
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn ignores_comment_that_only_contains_path_line() {
        let planned = plan(
            &FakeReader {
                link: LinkState::Missing,
                rc: Some(format!("#{PATH_LINE}\n").into_bytes()),
            },
            Path::new("/project"),
            Path::new("/home/dev"),
            OsStr::new("zsh"),
            None,
        )
        .unwrap();
        assert!(planned.append_path);
        assert_eq!(planned.rc, Some(PathBuf::from("/home/dev/.zshrc")));
    }
}
