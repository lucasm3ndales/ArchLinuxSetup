use std::collections::BTreeSet;

pub type Inventory = BTreeSet<String>;

pub fn parse_inventory(content: &str) -> Result<Inventory, String> {
    let mut packages = Inventory::new();
    for name in content.lines().filter(|name| !name.is_empty()) {
        if !valid_package_name(name) {
            return Err(format!("nome de pacote inválido no inventário: {name}"));
        }
        packages.insert(name.to_owned());
    }
    Ok(packages)
}

pub fn newly_installed(before: &Inventory, after: &Inventory) -> Inventory {
    after.difference(before).cloned().collect()
}

pub fn merge_manifest(existing: &Inventory, new: &Inventory) -> Inventory {
    existing.union(new).cloned().collect()
}

pub fn serialize_inventory(packages: &Inventory) -> String {
    packages.iter().map(|name| format!("{name}\n")).collect()
}

pub fn valid_package_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    !bytes.is_empty()
        && (bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit())
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'@' | b'.' | b'_' | b'+' | b'-')
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_only_new_packages_and_preserves_existing_manifest() {
        let before = parse_inventory("base\nrust\n").unwrap();
        let after = parse_inventory("base\nparu\nrust\ngit\n").unwrap();
        let existing = parse_inventory("docker\n").unwrap();
        let new = newly_installed(&before, &after);
        assert_eq!(serialize_inventory(&new), "git\nparu\n");
        assert_eq!(
            serialize_inventory(&merge_manifest(&existing, &new)),
            "docker\ngit\nparu\n"
        );
    }

    #[test]
    fn rejects_unsafe_inventory_names() {
        assert!(parse_inventory("safe\n$(malicious)\n").is_err());
        assert!(parse_inventory("-remove\n").is_err());
        assert_eq!(parse_inventory("7zip\n").unwrap().len(), 1);
    }
}
