use std::fs::{self, OpenOptions};
use std::os::unix::fs::symlink;
use std::path::Path;

/// Copy a user configuration file while preserving a symlink as a symlink.
pub fn copy(source: &Path, backup: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(source)
        .map_err(|error| format!("could not verify {}: {error}", source.display()))?;
    if metadata.file_type().is_symlink() {
        let target = fs::read_link(source)
            .map_err(|error| format!("could not read {}: {error}", source.display()))?;
        symlink(target, backup)
            .map_err(|error| format!("could not save {}: {error}", source.display()))
    } else {
        fs::copy(source, backup)
            .map_err(|error| format!("could not save {}: {error}", source.display()))?;
        let metadata = fs::metadata(source).map_err(|error| {
            format!("could not read metadata for {}: {error}", source.display())
        })?;
        let times = fs::FileTimes::new()
            .set_accessed(metadata.accessed().map_err(|error| error.to_string())?)
            .set_modified(metadata.modified().map_err(|error| error.to_string())?);
        OpenOptions::new()
            .write(true)
            .open(backup)
            .and_then(|file| file.set_times(times))
            .map_err(|error| format!("could not preserve backup timestamps: {error}"))
    }
}
