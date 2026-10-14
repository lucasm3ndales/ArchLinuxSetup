use std::path::PathBuf;

pub trait AurProbe {
    fn command_path(&self, name: &str) -> Option<PathBuf>;
    fn package_installed(&self, name: &str) -> bool;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RustProvider {
    RustupInstalled,
    RustInstalled,
    InstallRustup,
    Unknown,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Plan {
    pub paru: Option<PathBuf>,
    pub rust: RustProvider,
}

pub fn plan(probe: &impl AurProbe) -> Plan {
    let paru = probe.command_path("paru");
    let rust = if probe.command_path("pacman").is_none() {
        RustProvider::Unknown
    } else if probe.package_installed("rustup") {
        RustProvider::RustupInstalled
    } else if probe.package_installed("rust") {
        RustProvider::RustInstalled
    } else {
        RustProvider::InstallRustup
    };
    Plan { paru, rust }
}

pub fn build_packages(rust: RustProvider) -> Vec<&'static str> {
    let mut packages = vec!["git", "base-devel"];
    if matches!(rust, RustProvider::InstallRustup | RustProvider::Unknown) {
        packages.push("rustup");
    }
    packages
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeProbe {
        paru: bool,
        pacman: bool,
        rust: bool,
        rustup: bool,
    }

    impl AurProbe for FakeProbe {
        fn command_path(&self, name: &str) -> Option<PathBuf> {
            match name {
                "paru" if self.paru => Some(PathBuf::from("/usr/bin/paru")),
                "pacman" if self.pacman => Some(PathBuf::from("/usr/bin/pacman")),
                _ => None,
            }
        }

        fn package_installed(&self, name: &str) -> bool {
            match name {
                "rust" => self.rust,
                "rustup" => self.rustup,
                _ => false,
            }
        }
    }

    #[test]
    fn selects_one_rust_provider_without_conflicts() {
        let mut probe = FakeProbe {
            paru: false,
            pacman: true,
            rust: true,
            rustup: false,
        };
        assert_eq!(plan(&probe).rust, RustProvider::RustInstalled);
        assert_eq!(build_packages(plan(&probe).rust), vec!["git", "base-devel"]);
        probe.rust = false;
        assert_eq!(plan(&probe).rust, RustProvider::InstallRustup);
        assert_eq!(
            build_packages(plan(&probe).rust),
            vec!["git", "base-devel", "rustup"]
        );
        probe.rustup = true;
        assert_eq!(plan(&probe).rust, RustProvider::RustupInstalled);
        assert_eq!(build_packages(plan(&probe).rust), vec!["git", "base-devel"]);
    }

    #[test]
    fn already_available_paru_skips_installation() {
        let probe = FakeProbe {
            paru: true,
            pacman: false,
            rust: false,
            rustup: false,
        };
        assert_eq!(plan(&probe).paru, Some(PathBuf::from("/usr/bin/paru")));
        assert_eq!(plan(&probe).rust, RustProvider::Unknown);
    }
}
