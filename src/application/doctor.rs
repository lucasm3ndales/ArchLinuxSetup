use crate::application::desktop::{self, XfceFacts};
use crate::application::profile::{self, Profile};
use std::path::Path;

/// Only the observations needed by the compatibility check. Other commands can
/// define their own ports instead of growing this into a general system API.
pub trait SystemInspector {
    fn effective_uid(&self) -> Result<u32, String>;
    fn read_text(&self, path: &Path) -> Option<String>;
    fn command_exists(&self, name: &str) -> bool;
    fn command_output(&self, name: &str, args: &[&str]) -> Option<String>;
    fn package_installed(&self, name: &str) -> bool;
    fn env_var(&self, name: &str) -> Option<String>;
    fn profile_names(&self, root: &Path) -> Result<Vec<String>, String>;
}

#[derive(Debug, Eq, PartialEq)]
pub enum Message {
    Info(String),
    Error(String),
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Report {
    pub messages: Vec<Message>,
    pub failed: bool,
    pub ordinary_user: bool,
}

impl Report {
    fn info(&mut self, message: impl Into<String>) {
        self.messages.push(Message::Info(message.into()));
    }

    fn error(&mut self, message: impl Into<String>) {
        self.messages.push(Message::Error(message.into()));
    }

    fn failure(&mut self, message: impl Into<String>) {
        self.info(message);
        self.failed = true;
    }
}

pub fn inspect(system: &impl SystemInspector, root: &Path, profile: Option<&str>) -> Report {
    let mut report = Report::default();
    report.info("==> System check");
    report.info(format!("Read-only. Profile: {}", profile.unwrap_or("none")));

    match system.effective_uid() {
        Ok(0) => report.failure("[FAIL] Run as a regular user, without sudo."),
        Ok(_) => {
            report.info("[OK] Regular user");
            report.ordinary_user = true;
        }
        Err(error) => report.failure(format!("[FAIL] Could not identify the user: {error}")),
    }

    let os_id = system
        .read_text(Path::new("/etc/os-release"))
        .and_then(|content| parse_os_id(&content));
    if os_id.as_deref() == Some("arch") {
        report.info("[OK] Arch Linux");
    } else {
        report.failure("[FAIL] /etc/os-release does not identify this machine as Arch Linux.");
    }

    if let Some(profile) = profile {
        match check_profile(system, root, profile) {
            Ok(()) => report.info(format!("[OK] Hardware matches {profile}")),
            Err(error) => {
                report.error(format!("Error: {error}"));
                report.failure("[FAIL] Profile does not match this machine.");
            }
        }
    } else if let (Some(vendor), Some(product)) = (
        system.read_text(Path::new("/sys/class/dmi/id/sys_vendor")),
        system.read_text(Path::new("/sys/class/dmi/id/product_name")),
    ) {
        match system.profile_names(root) {
            Ok(names) => {
                let mut found = false;
                for name in names.into_iter().filter(|name| name != "generic") {
                    if let Ok(candidate) = read_profile(system, root, &name) {
                        if candidate.matches(vendor.trim(), product.trim()) {
                            report.info(format!(
                                "[INFO] Matching profile: {name}. Use --profile {name}."
                            ));
                            found = true;
                        }
                    }
                }
                if !found {
                    report.info("[INFO] No device-specific profile found. Use --profile generic to select the generic profile.");
                }
            }
            Err(error) => report.info(format!("[WARN] Could not check profiles: {error}")),
        }
    } else {
        report.info("[INFO] DMI is unavailable; use --profile generic if no device-specific profile applies.");
    }

    if system.command_exists("sudo") {
        report.info("[OK] sudo available");
    } else {
        report.info("[WARN] sudo not found; package installation requires sudo.");
    }
    if system.command_exists("pacman-conf") {
        let multilib = system
            .command_output("pacman-conf", &["--repo-list"])
            .is_some_and(|output| output.lines().any(|line| line == "multilib"));
        if multilib {
            report.info("[OK] [multilib] enabled for gaming");
        } else {
            report.info("[WARN] [multilib] disabled; gaming packages cannot be installed.");
        }
    } else {
        report.info("[WARN] pacman-conf unavailable; [multilib] not checked.");
    }

    if system.command_exists("findmnt") {
        let root_fs = mount_value(system, "/", "FSTYPE");
        let root_options = mount_value(system, "/", "OPTIONS");
        if root_fs.as_deref() == Some("btrfs") {
            report.info("[OK] Root uses Btrfs");
            if root_options
                .as_deref()
                .is_some_and(|options| has_subvolume(options, "@"))
            {
                report.info(
                    "[INFO] Root subvolume: @; verify the restore procedure for this layout.",
                );
            }
        } else {
            report.info(
                "[WARN] Root was not identified as Btrfs; Btrfs snapshots were not verified.",
            );
        }
        let home_fs = mount_value(system, "/home", "FSTYPE");
        let home_options = mount_value(system, "/home", "OPTIONS");
        if home_fs.as_deref() == Some("btrfs") {
            if home_options
                .as_deref()
                .is_some_and(|options| has_subvolume(options, "@home"))
            {
                report.info("[INFO] /home uses the @home Btrfs subvolume, outside root snapshots.");
            } else {
                report.info(
                    "[INFO] /home is a separate Btrfs mount; check its Snapper configuration.",
                );
            }
        }
        let boot_fs = mount_value(system, "/boot", "FSTYPE");
        let efi_fs = mount_value(system, "/boot/efi", "FSTYPE")
            .or_else(|| mount_value(system, "/efi", "FSTYPE"));
        if let Some(boot_fs) = boot_fs {
            report.info(format!("[WARN] /boot is a separate partition ({boot_fs}); root Btrfs snapshots do not include kernels or UKIs stored there."));
        } else if let Some(efi_fs) = efi_fs {
            report.info(format!(
                "[WARN] Separate EFI partition ({efi_fs}); root snapshots do not include UKIs stored there."
            ));
        } else {
            report.info("[WARN] EFI partition not found at /boot/efi or /efi.");
        }
    } else {
        report.info("[WARN] findmnt unavailable; Btrfs layout not checked.");
    }

    if system.command_exists("pacman") {
        if system.package_installed("snapper") {
            report.info(
                "[OK] Snapper installed; verify its configurations and create a test snapshot.",
            );
        } else {
            report.info("[WARN] Snapper not installed; root snapshots were not confirmed.");
        }
        if system.read_text(Path::new("/boot/limine.conf")).is_some()
            || system
                .read_text(Path::new("/boot/EFI/BOOT/limine.conf"))
                .is_some()
        {
            report.info("[INFO] Limine configuration found. Verify that snapshot entries and boot files are synchronized before relying on rollback.");
        } else if system.package_installed("limine") {
            report.info("[WARN] Limine is installed, but its configuration could not be read from the usual /boot locations. Check the active path and permissions before relying on rollback.");
        } else {
            report.info("[WARN] Bootloader configuration could not be verified. Check the active bootloader and recovery path.");
        }
        if system.package_installed("snapper")
            && system
                .read_text(Path::new("/proc/cmdline"))
                .is_some_and(|cmdline| {
                    cmdline
                        .split_whitespace()
                        .any(|arg| arg.starts_with("rootflags=subvol="))
                })
        {
            report.info("[WARN] Boot pins the root subvolume with rootflags=subvol=; snapper rollback alone may not change the booted root.");
        }
    }

    let session_type = system.env_var("XDG_SESSION_TYPE");
    let current_desktop = system.env_var("XDG_CURRENT_DESKTOP");
    let display = system.env_var("DISPLAY");
    match desktop::check_xfce(XfceFacts {
        session_type: session_type.as_deref(),
        current_desktop: current_desktop.as_deref(),
        display_available: display.is_some_and(|value| !value.is_empty()),
        xfce_session_installed: system.package_installed("xfce4-session"),
        xfce_panel_installed: system.package_installed("xfce4-panel"),
    }) {
        Ok(()) => report.info("[OK] Xfce installed and Xfce X11 session active"),
        Err(error) => report.info(format!("[WARN] Desktop: {error}")),
    }

    if report.failed {
        report.info("Check completed with compatibility failures. No changes made.");
    } else {
        report.info("Check completed. Review warnings before applying changes. No changes made.");
    }
    report
}

fn mount_value(system: &impl SystemInspector, mountpoint: &str, field: &str) -> Option<String> {
    system
        .command_output("findmnt", &["-rnM", mountpoint, "-o", field])
        .filter(|value| !value.is_empty())
}

pub fn check_profile(system: &impl SystemInspector, root: &Path, name: &str) -> Result<(), String> {
    let profile = read_profile(system, root, name)?;
    if name == "generic" {
        return Ok(());
    }
    let vendor = system
        .read_text(Path::new("/sys/class/dmi/id/sys_vendor"))
        .map(|value| value.trim().to_owned())
        .ok_or_else(|| "could not read this machine's vendor".to_owned())?;
    let product = system
        .read_text(Path::new("/sys/class/dmi/id/product_name"))
        .map(|value| value.trim().to_owned())
        .ok_or_else(|| "could not read this machine's model".to_owned())?;
    if profile.matches(&vendor, &product) {
        Ok(())
    } else {
        Err(format!(
            "profile {name} requires {} / {}; this machine reports {vendor} / {product}",
            profile.match_vendor.as_deref().unwrap_or("?"),
            profile.match_dmi.as_deref().unwrap_or("?")
        ))
    }
}

pub fn read_profile(
    system: &impl SystemInspector,
    root: &Path,
    name: &str,
) -> Result<Profile, String> {
    let file = root.join("profiles").join(name).join("profile.toml");
    let content = system
        .read_text(&file)
        .ok_or_else(|| format!("profile {name} has no profile.toml"))?;
    profile::parse(name, &content)
}

fn parse_os_id(content: &str) -> Option<String> {
    content.lines().find_map(|line| {
        line.strip_prefix("ID=")
            .map(|value| value.trim_matches('"').to_owned())
    })
}

fn has_subvolume(options: &str, name: &str) -> bool {
    options
        .split(',')
        .any(|option| option == format!("subvol=/{name}") || option == format!("subvol={name}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    #[derive(Default)]
    struct FakeInspector {
        uid: u32,
        files: HashMap<String, String>,
        commands: HashSet<String>,
        outputs: HashMap<String, String>,
        packages: HashSet<String>,
        environment: HashMap<String, String>,
        profiles: Vec<String>,
    }

    impl SystemInspector for FakeInspector {
        fn effective_uid(&self) -> Result<u32, String> {
            Ok(self.uid)
        }
        fn read_text(&self, path: &Path) -> Option<String> {
            self.files.get(&path.display().to_string()).cloned()
        }
        fn command_exists(&self, name: &str) -> bool {
            self.commands.contains(name)
        }
        fn command_output(&self, name: &str, args: &[&str]) -> Option<String> {
            self.outputs
                .get(&format!("{name} {}", args.join(" ")))
                .cloned()
        }
        fn package_installed(&self, name: &str) -> bool {
            self.packages.contains(name)
        }
        fn env_var(&self, name: &str) -> Option<String> {
            self.environment.get(name).cloned()
        }
        fn profile_names(&self, _: &Path) -> Result<Vec<String>, String> {
            Ok(self.profiles.clone())
        }
    }

    #[test]
    fn reports_compatible_arch_layout() {
        let mut fake = FakeInspector::default();
        fake.uid = 1000;
        fake.files
            .insert("/etc/os-release".into(), "ID=arch\n".into());
        fake.commands
            .extend(["sudo", "pacman-conf", "findmnt", "pacman"].map(str::to_owned));
        fake.outputs
            .insert("pacman-conf --repo-list".into(), "core\nmultilib".into());
        for (mount, field, value) in [
            ("/", "FSTYPE", "btrfs"),
            ("/", "OPTIONS", "rw,subvol=/@"),
            ("/home", "FSTYPE", "btrfs"),
            ("/home", "OPTIONS", "rw,subvol=/@home"),
            ("/boot/efi", "FSTYPE", "vfat"),
        ] {
            fake.outputs
                .insert(format!("findmnt -rnM {mount} -o {field}"), value.into());
        }
        fake.packages.insert("snapper".into());
        fake.files.insert(
            "/boot/EFI/BOOT/limine.conf".into(),
            "// Arch Linux\n".into(),
        );
        fake.files.insert(
            "/proc/cmdline".into(),
            "root=UUID=example rootflags=subvol=@ rw".into(),
        );
        let report = inspect(&fake, Path::new("/project"), None);
        assert!(!report.failed);
        assert!(report.ordinary_user);
        assert!(
            report
                .messages
                .contains(&Message::Info("[OK] Root uses Btrfs".into()))
        );
        assert!(
            report
                .messages
                .contains(&Message::Info("[OK] [multilib] enabled for gaming".into()))
        );
        assert!(report.messages.iter().any(|message| matches!(message, Message::Info(text) if text.contains("Limine configuration found"))));
        assert!(report.messages.iter().any(|message| matches!(message, Message::Info(text) if text.contains("snapper rollback alone"))));
    }

    #[test]
    fn reports_xfce_readiness_without_failing_general_diagnosis() {
        let mut fake = FakeInspector::default();
        fake.uid = 1000;
        fake.files
            .insert("/etc/os-release".into(), "ID=arch\n".into());
        let absent = inspect(&fake, Path::new("/project"), None);
        assert!(!absent.failed);
        assert!(absent.messages.iter().any(|message| matches!(message, Message::Info(text) if text.contains("[WARN] Desktop: Xfce is not installed"))));
        fake.packages
            .extend(["xfce4-session", "xfce4-panel"].map(str::to_owned));
        fake.environment
            .insert("XDG_SESSION_TYPE".into(), "x11".into());
        fake.environment
            .insert("XDG_CURRENT_DESKTOP".into(), "XFCE".into());
        fake.environment.insert("DISPLAY".into(), ":0".into());
        let ready = inspect(&fake, Path::new("/project"), None);
        assert!(ready.messages.contains(&Message::Info(
            "[OK] Xfce installed and Xfce X11 session active".into()
        )));
    }

    #[test]
    fn does_not_claim_limine_config_is_missing_when_boot_is_unreadable() {
        let mut fake = FakeInspector::default();
        fake.uid = 1000;
        fake.files
            .insert("/etc/os-release".into(), "ID=arch\n".into());
        fake.commands.insert("pacman".into());
        fake.packages.insert("limine".into());
        let report = inspect(&fake, Path::new("/project"), None);
        assert!(report.messages.iter().any(|message| matches!(message, Message::Info(text) if text.contains("configuration could not be read"))));
    }

    #[test]
    fn rejects_incompatible_profile() {
        let mut fake = FakeInspector::default();
        fake.uid = 1000;
        fake.files
            .insert("/etc/os-release".into(), "ID=arch\n".into());
        fake.files.insert(
            "/project/profiles/acer/profile.toml".into(),
            "name = \"acer\"\ndescription = \"Acer\"\nmatch_vendor = \"acer\"\nmatch_dmi = \"vx5-591g\"\n".into(),
        );
        fake.files
            .insert("/sys/class/dmi/id/sys_vendor".into(), "Other\n".into());
        fake.files
            .insert("/sys/class/dmi/id/product_name".into(), "Computer\n".into());
        let report = inspect(&fake, Path::new("/project"), Some("acer"));
        assert!(report.failed);
        assert!(report.messages.iter().any(
            |message| matches!(message, Message::Error(text) if text.contains("profile acer requires"))
        ));
    }

    #[test]
    fn suggests_matching_profile_without_selecting_it() {
        let mut fake = FakeInspector::default();
        fake.uid = 1000;
        fake.files
            .insert("/etc/os-release".into(), "ID=arch\n".into());
        fake.files
            .insert("/sys/class/dmi/id/sys_vendor".into(), "Acer Inc.\n".into());
        fake.files.insert(
            "/sys/class/dmi/id/product_name".into(),
            "Aspire VX5-591G\n".into(),
        );
        fake.files.insert(
            "/project/profiles/acer/profile.toml".into(),
            "name = \"acer\"\ndescription = \"Acer\"\nmatch_vendor = \"Acer\"\nmatch_dmi = \"VX5-591G\"\n".into(),
        );
        fake.profiles = vec!["acer".into(), "generic".into()];
        let report = inspect(&fake, Path::new("/project"), None);
        assert!(!report.failed);
        assert!(report.messages.iter().any(
            |message| matches!(message, Message::Info(text) if text.contains("Use --profile acer"))
        ));
    }

    #[test]
    fn accepts_generic_profile_without_dmi() {
        let mut fake = FakeInspector::default();
        fake.files.insert(
            "/project/profiles/generic/profile.toml".into(),
            "name = \"generic\"\ndescription = \"No extras\"\n".into(),
        );
        assert!(check_profile(&fake, Path::new("/project"), "generic").is_ok());
    }

    #[test]
    fn parses_system_facts() {
        assert_eq!(
            parse_os_id("NAME=Arch Linux\nID=arch\n"),
            Some("arch".to_owned())
        );
        assert!(has_subvolume("rw,ssd,subvol=/@,compress=zstd:3", "@"));
        assert!(!has_subvolume("rw,ssd,subvol=/@home", "@"));
    }
}
