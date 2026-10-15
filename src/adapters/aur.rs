use crate::application::aur::{AurProbe, RustProvider, build_packages};
use crate::system;
use std::env;
use std::fs::{self, DirBuilder};
use std::io::{self, IsTerminal, Write};
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct LinuxAur;

impl AurProbe for LinuxAur {
    fn command_path(&self, name: &str) -> Option<PathBuf> {
        let path = env::var_os("PATH")?;
        env::split_paths(&path)
            .map(|dir| dir.join(name))
            .find(|candidate| {
                candidate.metadata().is_ok_and(|metadata| {
                    metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
                })
            })
    }

    fn package_installed(&self, name: &str) -> bool {
        system::installed(name)
    }
}

impl LinuxAur {
    pub fn preflight(&self) -> Result<(), String> {
        if system::effective_uid()? == 0 {
            return Err("Run as a regular user, without sudo.".to_owned());
        }
        if !io::stdin().is_terminal() {
            return Err(
                "An interactive terminal is required if sudo requests authentication.".to_owned(),
            );
        }
        let os_release = fs::read_to_string("/etc/os-release")
            .map_err(|_| "Could not identify the distribution.".to_owned())?;
        if !os_release
            .lines()
            .any(|line| line == "ID=arch" || line == "ID=\"arch\"")
        {
            return Err("This command can only run on Arch Linux.".to_owned());
        }
        for name in ["sudo", "pacman"] {
            if self.command_path(name).is_none() {
                return Err(format!("Missing command: {name}"));
            }
        }
        Ok(())
    }

    pub fn apply(&self) -> Result<(), String> {
        println!("\n==> Preparing build tools");
        run("sudo", &["-v"], None)?;
        let rust = crate::application::aur::plan(self).rust;
        let mut args = vec!["pacman", "-Syu", "--needed", "--noconfirm"];
        let packages = build_packages(rust);
        match rust {
            RustProvider::RustupInstalled => {
                println!("rustup is installed and will be used to build paru.")
            }
            RustProvider::RustInstalled => {
                println!("rust is installed and will be used to build paru.")
            }
            RustProvider::InstallRustup | RustProvider::Unknown => {
                println!("rustup will be installed before building paru.")
            }
        }
        args.extend(packages);
        run("sudo", &args, None)?;

        for name in ["git", "makepkg"] {
            if self.command_path(name).is_none() {
                return Err(format!("{name} was not found after preparing build tools."));
            }
        }
        if system::installed("rustup") {
            if self.command_path("rustup").is_none() {
                return Err("rustup is installed but its command was not found in PATH.".to_owned());
            }
            let toolchains = output("rustup", &["toolchain", "list"])?;
            if !toolchains.contains("(default)") {
                println!("Configuring the stable Rust toolchain to build paru.");
                run("rustup", &["default", "stable"], None)?;
            }
        }
        if self.command_path("cargo").is_none() {
            return Err("cargo was not found after preparing Rust.".to_owned());
        }

        println!("\n==> Downloading the paru PKGBUILD");
        let temporary = TempBuildDir::new()?;
        let repo = temporary.path.join("paru");
        let repo_text = repo.to_string_lossy();
        run(
            "git",
            &[
                "clone",
                "--",
                "https://aur.archlinux.org/paru.git",
                &repo_text,
            ],
            None,
        )?;
        let pkgbuild = repo.join("PKGBUILD");
        if !pkgbuild.is_file() {
            return Err("The cloned repository does not contain PKGBUILD.".to_owned());
        }
        println!("\nFiles received from AUR:");
        run("git", &["ls-files"], Some(&repo))?;
        println!("\nReceived PKGBUILD:");
        let contents =
            fs::read(&pkgbuild).map_err(|error| format!("could not read PKGBUILD: {error}"))?;
        io::stdout()
            .write_all(&contents)
            .map_err(|error| format!("could not display PKGBUILD: {error}"))?;
        println!("\nOther downloaded files: {}", repo.display());

        println!("\n==> Building and installing paru");
        run("makepkg", &["-si", "--noconfirm"], Some(&repo))?;
        if self.command_path("paru").is_none() {
            return Err("makepkg finished, but paru was not found in PATH.".to_owned());
        }
        Ok(())
    }
}

fn run(name: &str, args: &[&str], cwd: Option<&Path>) -> Result<(), String> {
    io::stdout().flush().map_err(|error| error.to_string())?;
    let mut command = Command::new(name);
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let status = command
        .status()
        .map_err(|error| format!("could not run {name}: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "{name} failed (exit code {}).",
            status.code().unwrap_or(1)
        ))
    }
}

fn output(name: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(name)
        .args(args)
        .output()
        .map_err(|error| format!("could not run {name}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "{name} failed (exit code {}).",
            output.status.code().unwrap_or(1)
        ));
    }
    String::from_utf8(output.stdout).map_err(|_| format!("invalid output from {name}"))
}

struct TempBuildDir {
    path: PathBuf,
}

impl TempBuildDir {
    fn new() -> Result<Self, String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos();
        for attempt in 0..10 {
            let path = env::temp_dir().join(format!(
                "archlinuxsetup-paru-{}-{now}-{attempt}",
                std::process::id()
            ));
            match DirBuilder::new().mode(0o700).create(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(format!("could not create temporary directory: {error}"));
                }
            }
        }
        Err("could not reserve a temporary directory.".to_owned())
    }
}

impl Drop for TempBuildDir {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.path) {
            eprintln!("Warning: could not remove {}: {error}", self.path.display());
        }
    }
}
