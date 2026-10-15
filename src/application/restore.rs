use crate::application::package_state::valid_package_name;
use std::collections::BTreeSet;
use std::path::PathBuf;

pub const XFCE_CHANNELS: [&str; 4] = [
    "xsettings",
    "xfwm4",
    "xfce4-panel",
    "xfce4-keyboard-shortcuts",
];

pub const ROOT_FILES: [&str; 3] = [
    "/etc/ly/config.ini",
    "/etc/X11/xorg.conf.d/90-archlinuxsetup-touchpad.conf",
    "/etc/X11/xorg.conf.d/91-archlinuxsetup-acer-tapping.conf",
];

#[derive(Debug, Eq, PartialEq)]
pub enum RootAction {
    RemoveOwned,
    Restore(PathBuf),
    PreserveChanged,
    PreserveUnknown,
    None,
}

pub fn root_action(
    marker_exists: bool,
    marker_matches: bool,
    target_exists: bool,
    backup: Option<PathBuf>,
) -> RootAction {
    if marker_exists {
        if !target_exists {
            RootAction::None
        } else if marker_matches {
            RootAction::RemoveOwned
        } else {
            RootAction::PreserveChanged
        }
    } else if let Some(backup) = backup {
        RootAction::Restore(backup)
    } else if target_exists {
        RootAction::PreserveUnknown
    } else {
        RootAction::None
    }
}

pub fn parse_packages(content: &str) -> Result<Vec<String>, String> {
    let mut seen = BTreeSet::new();
    let mut packages = Vec::new();
    for name in content.lines() {
        if !valid_package_name(name) {
            return Err(format!("Invalid package name in record: {name}"));
        }
        if seen.insert(name) {
            packages.push(name.to_owned());
        }
    }
    Ok(packages)
}

#[derive(Debug, Eq, PartialEq)]
pub struct DockerBefore {
    pub user: String,
    pub in_group: bool,
    pub enabled: bool,
    pub active: bool,
}

pub fn parse_docker_before(content: &str, current_user: &str) -> Result<DockerBefore, String> {
    let values: Vec<_> = content.lines().collect();
    if values.len() != 4 || values[0] != current_user || current_user.is_empty() {
        return Err("Invalid previous Docker state record.".to_owned());
    }
    fn bool_value(value: &str) -> Result<bool, String> {
        match value {
            "yes" => Ok(true),
            "no" => Ok(false),
            _ => Err("Invalid previous Docker state record.".to_owned()),
        }
    }
    Ok(DockerBefore {
        user: current_user.to_owned(),
        in_group: bool_value(values[1])?,
        enabled: bool_value(values[2])?,
        active: bool_value(values[3])?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_files_need_proof_before_removal() {
        let backup = PathBuf::from("/backup");
        assert_eq!(
            root_action(true, true, true, Some(backup.clone())),
            RootAction::RemoveOwned
        );
        assert_eq!(
            root_action(true, false, true, Some(backup.clone())),
            RootAction::PreserveChanged
        );
        assert_eq!(
            root_action(false, false, true, None),
            RootAction::PreserveUnknown
        );
        assert_eq!(
            root_action(false, false, true, Some(backup.clone())),
            RootAction::Restore(backup)
        );
    }

    #[test]
    fn manifest_is_validated_and_duplicates_are_removed() {
        assert_eq!(
            parse_packages("docker\nparu\ndocker\n").unwrap(),
            vec!["docker", "paru"]
        );
        assert!(parse_packages("safe\n\n").is_err());
        assert!(parse_packages("-unsafe\n").is_err());
    }

    #[test]
    fn docker_record_must_match_user_and_booleans() {
        let valid = parse_docker_before("dev\nno\nyes\nno\n", "dev").unwrap();
        assert!(!valid.in_group && valid.enabled && !valid.active);
        assert!(parse_docker_before("other\nno\nyes\nno\n", "dev").is_err());
        assert!(parse_docker_before("dev\nmaybe\nyes\nno\n", "dev").is_err());
    }
}
