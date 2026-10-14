use std::path::PathBuf;

pub fn find_root() -> Result<PathBuf, String> {
    if let Some(root) = std::env::var_os("ARCHSETUP_PROJECT_DIR") {
        let root = PathBuf::from(root);
        if valid_root(&root) {
            return Ok(root);
        }
        return Err("ARCHSETUP_PROJECT_DIR não aponta para um repositório ArchSetup".to_owned());
    }
    let executable = std::env::current_exe()
        .map_err(|error| format!("não foi possível localizar o executável: {error}"))?;
    for parent in executable.ancestors() {
        if valid_root(parent) {
            return Ok(parent.to_path_buf());
        }
    }
    Err("não foi possível localizar o repositório ArchSetup".to_owned())
}

fn valid_root(root: &std::path::Path) -> bool {
    root.join("Cargo.toml").is_file() && root.join("src/main.rs").is_file()
}

pub fn valid_profile_name(name: &str) -> bool {
    !name.is_empty()
        && name.as_bytes()[0].is_ascii_lowercase()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}
