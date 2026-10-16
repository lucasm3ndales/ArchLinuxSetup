use super::aur;
use crate::adapters::package_state::PackageTracker;
use crate::adapters::pkg::LinuxPkg;
use crate::application::pkg::{
    self, Category, NVIDIA_580XX_CONFLICTS, PackageSource, RustProvider, Selection,
};
use crate::project;
use std::ffi::OsString;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Eq, PartialEq)]
struct Options {
    categories: Vec<Category>,
    profile: Option<String>,
    dry_run: bool,
}

fn parse_args(args: &[OsString]) -> Result<Option<Options>, String> {
    let mut options = Options::default();
    let mut index = 0;
    while index < args.len() {
        let value = args[index].to_string_lossy();
        match value.as_ref() {
            "--dry-run" => options.dry_run = true,
            "--profile" => {
                index += 1;
                let profile = args
                    .get(index)
                    .ok_or("Missing name after --profile.")?
                    .to_string_lossy();
                if !project::valid_profile_name(&profile) {
                    return Err("Invalid profile name.".to_owned());
                }
                options.profile = Some(profile.into_owned());
            }
            "-h" | "--help" => return Ok(None),
            name => match Category::parse(name) {
                Some(category) => options.categories.push(category),
                None => return Err(format!("Unknown option or category: {name}")),
            },
        }
        index += 1;
    }
    if options.categories.is_empty() {
        options.categories.extend(Category::DEFAULT);
    }
    Ok(Some(options))
}

pub fn help() -> i32 {
    println!("Usage: archsetup pkg [CATEGORY...] [--profile NAME] [--dry-run]\n");
    println!("Categories: base development apps theme desktop gaming emulation");
    println!("Default: base development apps theme desktop\n");
    println!("The profile is optional. Without it, only shared packages are selected.");
    println!("--dry-run shows the selection without updating or installing anything.");
    0
}

pub fn run(root: &Path, args: &[OsString]) -> i32 {
    let options = match parse_args(args) {
        Ok(Some(options)) => options,
        Ok(None) => return help(),
        Err(error) => {
            eprintln!("Error: {error}");
            return 2;
        }
    };
    if let Some(profile) = &options.profile {
        if !root.join("profiles").join(profile).is_dir() {
            eprintln!("Error: Unknown profile: {profile}");
            return 2;
        }
    }
    let home = match std::env::var_os("HOME").filter(|home| !home.is_empty()) {
        Some(home) => PathBuf::from(home),
        None => {
            eprintln!("Error: HOME is not set.");
            return 1;
        }
    };
    let state_home = std::env::var_os("XDG_STATE_HOME").map(PathBuf::from);
    let linux = LinuxPkg::new(root, home.clone(), state_home.clone());
    let selection = match pkg::select(
        &linux,
        root,
        &options.categories,
        options.profile.as_deref(),
    ) {
        Ok(selection) => selection,
        Err(error) => {
            eprintln!("Error: {error}");
            return 1;
        }
    };
    show_selection(&linux, &selection, &options);
    if options.dry_run {
        if !selection.aur.is_empty() && linux.helper().is_none() {
            println!("\nDependency: archsetup aur installs the helper required for AUR packages.");
        }
        if selection.includes(Category::Development) {
            println!(
                "\nDocker: add the current user to the docker group if needed, then enable/start docker.service."
            );
            if linux.installed("ufw") {
                println!("Warning: Docker-published ports may bypass UFW rules.");
            }
        }
        if selection.includes(Category::Gaming) {
            println!(
                "\nSteam requires [multilib] and 32-bit graphics libraries compatible with the GPU."
            );
        }
        println!("\nPreview complete. No changes made. Dependency state was not calculated.");
        return 0;
    }
    if let Err(error) = linux.preflight(&selection, options.profile.as_deref()) {
        eprintln!("Error: {error}");
        return 1;
    }
    let helper = if selection.aur.iter().all(|package| linux.installed(package)) {
        None
    } else {
        match ensure_aur_helper(&linux) {
            Ok(helper) => Some(helper),
            Err(error) => {
                eprintln!("Error: {error}");
                return 1;
            }
        }
    };
    let packages_missing = selection
        .official
        .iter()
        .chain(selection.aur.iter())
        .any(|name| !linux.installed(name));
    let tracker = if packages_missing {
        match PackageTracker::begin(&home, state_home.as_deref()) {
            Ok(tracker) => Some(tracker),
            Err(error) => {
                eprintln!("Error: {error}");
                return 1;
            }
        }
    } else {
        None
    };
    let result = linux.apply(&selection, helper);
    let tracked = tracker.map(PackageTracker::finish).transpose();
    if let Ok(Some(count)) = &tracked {
        if *count > 0 {
            println!("New packages tracked for possible removal: {count}");
        }
    }
    if let Err(error) = result {
        eprintln!("Error: {error}");
        if let Err(tracking_error) = tracked {
            eprintln!("Warning: package tracking is incomplete: {tracking_error}");
        }
        return 1;
    }
    if let Err(error) = tracked {
        eprintln!("Error: installation finished, but package tracking failed: {error}");
        return 1;
    }
    let categories = selection
        .categories
        .iter()
        .map(|category| category.name())
        .collect::<Vec<_>>()
        .join(" ");
    println!(
        "\nComplete: categories {categories} processed. Already installed packages were skipped."
    );
    0
}

fn show_selection(source: &impl PackageSource, selection: &Selection, options: &Options) {
    println!(
        "==> {}",
        if options.dry_run {
            "Package preview"
        } else {
            "Package installation"
        }
    );
    println!(
        "Profile: {}",
        options
            .profile
            .as_deref()
            .unwrap_or("none (shared packages only)")
    );
    println!(
        "Categories: {}",
        selection
            .categories
            .iter()
            .map(|category| category.name())
            .collect::<Vec<_>>()
            .join(" ")
    );
    let mut installed = 0;
    let mut missing = 0;
    println!("\nOfficial packages ({}):", selection.official.len());
    for package in &selection.official {
        show_package(source, package, &mut installed, &mut missing);
    }
    println!("\nAUR packages ({}):", selection.aur.len());
    if selection.aur.is_empty() {
        println!("  none");
    } else {
        for package in &selection.aur {
            show_package(source, package, &mut installed, &mut missing);
        }
    }
    if source.pacman_available() {
        println!("\nSummary: {installed} installed; {missing} missing.");
    }
    if selection.rust_provider == RustProvider::Rust {
        println!(
            "\nRust: rust is already installed; rustup was removed from the selection to avoid a conflict."
        );
    }
    if selection.aur.iter().any(|name| name == "nvidia-580xx-dkms") {
        let conflicts = NVIDIA_580XX_CONFLICTS
            .iter()
            .copied()
            .filter(|name| source.installed(name))
            .collect::<Vec<_>>();
        if !conflicts.is_empty() {
            println!(
                "\n[WARN] NVIDIA 580xx conflicts with: {}. Installation will be blocked until those packages are reviewed.",
                conflicts.join(", ")
            );
        }
    }
}

fn show_package(
    source: &impl PackageSource,
    package: &str,
    installed: &mut usize,
    missing: &mut usize,
) {
    if !source.pacman_available() {
        println!("  {package:<28} not checked (pacman unavailable)");
    } else if source.installed(package) {
        *installed += 1;
        println!("  {package:<28} installed");
    } else {
        *missing += 1;
        println!("  {package:<28} missing");
    }
}

fn ensure_aur_helper(linux: &LinuxPkg) -> Result<&'static str, String> {
    if let Some(helper) = linux.helper() {
        return Ok(helper);
    }
    println!("\nDependency: AUR packages require a helper. archsetup aur installs paru.");
    if !io::stdin().is_terminal() {
        return Err("Run archsetup aur in an interactive terminal and try again.".to_owned());
    }
    print!("Run archsetup aur now? [y/N] ");
    io::stdout().flush().map_err(|error| error.to_string())?;
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .map_err(|_| "Could not read the response.".to_owned())?;
    if !matches!(answer.trim(), "y" | "Y" | "yes" | "YES" | "Yes") {
        return Err("Dependency not installed. Install the helper and try again.".to_owned());
    }
    if aur::run(&[]) != 0 {
        return Err("archsetup aur failed; check the output above.".to_owned());
    }
    if !crate::system::command_exists("paru") {
        return Err("archsetup aur finished, but paru was not found in PATH.".to_owned());
    }
    Ok("paru")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_new_categories_and_default_order() {
        assert_eq!(
            parse_args(&[]).unwrap().unwrap().categories,
            Category::DEFAULT
        );
        let options = parse_args(&["theme".into(), "apps".into(), "--dry-run".into()])
            .unwrap()
            .unwrap();
        assert_eq!(options.categories, vec![Category::Theme, Category::Apps]);
        assert!(options.dry_run);
        assert!(parse_args(&["visual".into()]).is_err());
        assert!(parse_args(&["--profile".into(), "../wrong".into()]).is_err());
    }
}
