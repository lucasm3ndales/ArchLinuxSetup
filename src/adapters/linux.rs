use crate::application::doctor::SystemInspector;
use crate::project;
use crate::system;
use std::fs;
use std::path::Path;

/// Read-only adapter for the current Linux host.
pub struct LinuxInspector;

impl SystemInspector for LinuxInspector {
    fn effective_uid(&self) -> Result<u32, String> {
        system::effective_uid()
    }

    fn read_text(&self, path: &Path) -> Option<String> {
        fs::read_to_string(path).ok()
    }

    fn command_exists(&self, name: &str) -> bool {
        system::command_exists(name)
    }

    fn command_output(&self, name: &str, args: &[&str]) -> Option<String> {
        system::output(name, args)
    }

    fn package_installed(&self, name: &str) -> bool {
        system::installed(name)
    }

    fn env_var(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }

    fn profile_names(&self, root: &Path) -> Result<Vec<String>, String> {
        let entries = fs::read_dir(root.join("profiles"))
            .map_err(|error| format!("não foi possível listar perfis: {error}"))?;
        let mut names = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|error| error.to_string())?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if project::valid_profile_name(&name)
                && entry.path().join("profile.toml").is_file() {
                names.push(name);
            }
        }
        names.sort();
        Ok(names)
    }
}
