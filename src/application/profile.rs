use crate::application::package_state::valid_package_name;
use crate::application::pkg::Category;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Profile {
    pub name: String,
    pub description: String,
    pub match_vendor: Option<String>,
    pub match_dmi: Option<String>,
    pub touchpad_tapping: bool,
    pub official: HashMap<String, Vec<String>>,
    pub aur: HashMap<String, Vec<String>>,
}

impl Profile {
    pub fn matches(&self, vendor: &str, product: &str) -> bool {
        match (&self.match_vendor, &self.match_dmi) {
            (Some(expected_vendor), Some(expected_product)) => {
                vendor
                    .to_ascii_lowercase()
                    .contains(&expected_vendor.to_ascii_lowercase())
                    && product
                        .to_ascii_lowercase()
                        .contains(&expected_product.to_ascii_lowercase())
            }
            _ => self.name == "generic",
        }
    }

    pub fn packages(&self, kind: &str, category: Category) -> &[String] {
        let packages = if kind == "official" {
            &self.official
        } else {
            &self.aur
        };
        packages
            .get(category.name())
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

/// Parse the small, documented TOML shape used for profiles. Reject unsupported keys
/// so a contributor cannot add a setting that the CLI silently ignores.
pub fn parse(name: &str, content: &str) -> Result<Profile, String> {
    let mut profile = Profile::default();
    let mut section = "";
    let mut seen = HashSet::new();
    let mut sections = HashSet::new();
    for (index, raw) in content.lines().enumerate() {
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = &line[1..line.len() - 1];
            if !matches!(section, "packages.official" | "packages.aur" | "desktop") {
                return Err(format!("Unknown section in profile {name}: {section}"));
            }
            if !sections.insert(section.to_owned()) {
                return Err(format!("Duplicate section in profile {name}: {section}"));
            }
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("Invalid line in profile {name}: {}", index + 1))?;
        let key = key.trim();
        let value = value.trim();
        if !seen.insert((section.to_owned(), key.to_owned())) {
            return Err(format!(
                "Duplicate field in profile {name}: {section}.{key}"
            ));
        }
        match (section, key) {
            ("", "name") => profile.name = quoted(value)?.to_owned(),
            ("", "description") => profile.description = quoted(value)?.to_owned(),
            ("", "match_vendor") => profile.match_vendor = Some(quoted(value)?.to_owned()),
            ("", "match_dmi") => profile.match_dmi = Some(quoted(value)?.to_owned()),
            ("desktop", "touchpad_tapping") => {
                profile.touchpad_tapping = match value {
                    "true" => true,
                    "false" => false,
                    _ => return Err(format!("Invalid boolean in profile {name}: {key}")),
                }
            }
            ("packages.official" | "packages.aur", category)
                if Category::parse(category).is_some() =>
            {
                let target = if section == "packages.official" {
                    &mut profile.official
                } else {
                    &mut profile.aur
                };
                if target
                    .insert(category.to_owned(), package_array(value)?)
                    .is_some()
                {
                    return Err(format!("Duplicate category in profile {name}: {category}"));
                }
            }
            _ => return Err(format!("Unknown field in profile {name}: {section}.{key}")),
        }
    }
    if profile.name != name || profile.description.trim().is_empty() {
        return Err(format!("Invalid name or description in profile {name}"));
    }
    if name == "generic" {
        if profile.match_vendor.is_some() || profile.match_dmi.is_some() || profile.touchpad_tapping
        {
            return Err(
                "generic cannot require hardware or apply device-specific settings".to_owned(),
            );
        }
    } else if profile
        .match_vendor
        .as_deref()
        .is_none_or(|value| value.trim().is_empty())
        || profile
            .match_dmi
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
    {
        return Err(format!(
            "Profile {name} requires match_vendor and match_dmi"
        ));
    }
    Ok(profile)
}

fn strip_comment(line: &str) -> &str {
    let mut quoted = false;
    for (index, character) in line.char_indices() {
        if character == '"' {
            quoted = !quoted;
        } else if character == '#' && !quoted {
            return &line[..index];
        }
    }
    line
}

fn quoted(value: &str) -> Result<&str, String> {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .filter(|value| !value.contains('"') && !value.contains('\\'))
        .ok_or_else(|| format!("Invalid basic TOML string: {value}"))
}

fn package_array(value: &str) -> Result<Vec<String>, String> {
    let inner = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or_else(|| format!("Invalid package list: {value}"))?
        .trim();
    if inner.is_empty() {
        return Ok(Vec::new());
    }
    inner
        .split(',')
        .map(|entry| {
            let name = quoted(entry.trim())?;
            if !valid_package_name(name) {
                return Err(format!("Invalid package in profile: {name}"));
            }
            Ok(name.to_owned())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_profile_and_matches_base_model() {
        let profile = parse("acer", "name = \"acer\"\ndescription = \"Acer\"\nmatch_vendor = \"Acer\"\nmatch_dmi = \"VX5-591G\"\n[packages.official]\ngaming = [\"lib32-mesa\"]\n[desktop]\ntouchpad_tapping = true\n").unwrap();
        assert!(profile.matches("Acer Inc.", "Aspire VX5-591G"));
        assert_eq!(
            profile.packages("official", Category::Gaming),
            &["lib32-mesa".to_owned()]
        );
        assert!(profile.touchpad_tapping);
        assert!(!profile.matches("Other", "VX5-591G"));
    }

    #[test]
    fn rejects_unknown_fields_and_unsafe_packages() {
        assert!(
            parse(
                "generic",
                "name = \"generic\"\ndescription = \"Base\"\n[desktop]\nrun_shell = \"rm -rf /\"\n"
            )
            .is_err()
        );
        assert!(parse("generic", "name = \"generic\"\ndescription = \"Base\"\n[packages.official]\nbase = [\"bad package\"]\n").is_err());
        assert!(
            parse(
                "generic",
                "name = \"generic\"\nname = \"generic\"\ndescription = \"Base\"\n"
            )
            .is_err()
        );
        assert_eq!(
            parse(
                "generic",
                "name = \"generic\"\ndescription = \"Base #1\" # comment\n"
            )
            .unwrap()
            .description,
            "Base #1"
        );
    }
}
