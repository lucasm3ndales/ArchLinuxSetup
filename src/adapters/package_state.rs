use crate::application::package_state::{self, Inventory};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct PackageTracker {
    state_dir: PathBuf,
    before_path: PathBuf,
    before: Inventory,
    pacman: PathBuf,
}

impl PackageTracker {
    pub fn begin(home: &Path, state_home: Option<&Path>) -> Result<Self, String> {
        Self::begin_with(home, state_home, Path::new("pacman"))
    }

    fn begin_with(home: &Path, state_home: Option<&Path>, pacman: &Path) -> Result<Self, String> {
        let state_dir = state_home
            .filter(|path| !path.as_os_str().is_empty())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| home.join(".local/state"))
            .join("archlinuxsetup");
        fs::create_dir_all(&state_dir)
            .map_err(|error| format!("não foi possível criar {}: {error}", state_dir.display()))?;
        let before = installed_packages(pacman)?;
        let before_path = unique_path(&state_dir, "packages-before")?;
        write_new(&before_path, &package_state::serialize_inventory(&before))?;
        let baseline = state_dir.join("packages-baseline");
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&baseline)
        {
            Ok(mut file) => file
                .write_all(package_state::serialize_inventory(&before).as_bytes())
                .map_err(|error| {
                    format!("não foi possível registrar o inventário inicial: {error}")
                })?,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(format!(
                    "não foi possível criar {}: {error}",
                    baseline.display()
                ));
            }
        }
        Ok(Self {
            state_dir,
            before_path,
            before,
            pacman: pacman.to_path_buf(),
        })
    }

    pub fn finish(self) -> Result<usize, String> {
        self.finish_inner().map_err(|error| {
            format!(
                "{error}. Inventário anterior preservado em {}",
                self.before_path.display()
            )
        })
    }

    fn finish_inner(&self) -> Result<usize, String> {
        let after = installed_packages(&self.pacman)?;
        let new = package_state::newly_installed(&self.before, &after);
        let manifest = self.state_dir.join("packages-installed");
        let existing = match fs::read_to_string(&manifest) {
            Ok(content) => package_state::parse_inventory(&content)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Inventory::new(),
            Err(error) => {
                return Err(format!(
                    "não foi possível ler {}: {error}",
                    manifest.display()
                ));
            }
        };
        let merged = package_state::merge_manifest(&existing, &new);
        let temporary = unique_path(&self.state_dir, "packages-merged")?;
        write_new(&temporary, &package_state::serialize_inventory(&merged))?;
        fs::rename(&temporary, &manifest).map_err(|error| {
            format!("não foi possível atualizar {}: {error}", manifest.display())
        })?;
        fs::remove_file(&self.before_path)
            .map_err(|error| format!("não foi possível remover inventário temporário: {error}"))?;
        Ok(new.len())
    }
}

fn installed_packages(pacman: &Path) -> Result<Inventory, String> {
    let output = Command::new(pacman)
        .arg("-Qq")
        .output()
        .map_err(|error| format!("não foi possível consultar pacotes: {error}"))?;
    if !output.status.success() {
        return Err("não foi possível registrar os pacotes existentes".to_owned());
    }
    let content = String::from_utf8(output.stdout)
        .map_err(|_| "pacman retornou nomes de pacotes inválidos".to_owned())?;
    package_state::parse_inventory(&content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn records_only_new_packages_across_runs_and_preserves_failed_snapshot() {
        let root = std::env::temp_dir().join(format!(
            "archsetup-package-state-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let inventory = root.join("inventory");
        let fake_pacman = root.join("pacman");
        fs::write(
            &fake_pacman,
            format!("#!/bin/sh\ncat '{}'\n", inventory.display()),
        )
        .unwrap();
        fs::set_permissions(&fake_pacman, fs::Permissions::from_mode(0o700)).unwrap();
        fs::write(&inventory, "base\n").unwrap();
        let state_home = root.join("state");
        let state_dir = state_home.join("archlinuxsetup");
        let first = PackageTracker::begin_with(&root, Some(&state_home), &fake_pacman).unwrap();
        fs::write(state_dir.join("packages-installed"), "docker\n").unwrap();
        fs::write(&inventory, "base\ngit\nparu\n").unwrap();
        assert_eq!(first.finish().unwrap(), 2);
        assert_eq!(
            fs::read_to_string(state_dir.join("packages-baseline")).unwrap(),
            "base\n"
        );
        assert_eq!(
            fs::read_to_string(state_dir.join("packages-installed")).unwrap(),
            "docker\ngit\nparu\n"
        );

        let repeated = PackageTracker::begin_with(&root, Some(&state_home), &fake_pacman).unwrap();
        assert_eq!(repeated.finish().unwrap(), 0);
        let failed = PackageTracker::begin_with(&root, Some(&state_home), &fake_pacman).unwrap();
        let before_path = failed.before_path.clone();
        fs::remove_file(&inventory).unwrap();
        assert!(failed.finish().is_err());
        assert_eq!(
            fs::read_to_string(before_path).unwrap(),
            "base\ngit\nparu\n"
        );
        fs::remove_dir_all(root).unwrap();
    }
}

fn unique_path(dir: &Path, prefix: &str) -> Result<PathBuf, String> {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    Ok(dir.join(format!("{prefix}-{}-{suffix}", std::process::id())))
}

fn write_new(path: &Path, content: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("não foi possível criar {}: {error}", path.display()))?;
    file.write_all(content.as_bytes())
        .map_err(|error| format!("não foi possível escrever {}: {error}", path.display()))
}
