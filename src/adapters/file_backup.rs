use std::fs::{self, OpenOptions};
use std::os::unix::fs::symlink;
use std::path::Path;

/// Copy a user configuration file while preserving a symlink as a symlink.
pub fn copy(source: &Path, backup: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(source)
        .map_err(|error| format!("não foi possível verificar {}: {error}", source.display()))?;
    if metadata.file_type().is_symlink() {
        let target = fs::read_link(source)
            .map_err(|error| format!("não foi possível ler {}: {error}", source.display()))?;
        symlink(target, backup)
            .map_err(|error| format!("não foi possível salvar {}: {error}", source.display()))
    } else {
        fs::copy(source, backup)
            .map_err(|error| format!("não foi possível salvar {}: {error}", source.display()))?;
        let metadata = fs::metadata(source).map_err(|error| {
            format!(
                "não foi possível ler metadados de {}: {error}",
                source.display()
            )
        })?;
        let times = fs::FileTimes::new()
            .set_accessed(metadata.accessed().map_err(|error| error.to_string())?)
            .set_modified(metadata.modified().map_err(|error| error.to_string())?);
        OpenOptions::new()
            .write(true)
            .open(backup)
            .and_then(|file| file.set_times(times))
            .map_err(|error| format!("não foi possível preservar datas do backup: {error}"))
    }
}
