use crate::adapters::linux::LinuxInspector;
use crate::application::doctor::{self, SystemInspector};
use std::ffi::OsString;
use std::path::Path;

pub fn help() -> i32 {
    println!("Usage: archsetup profiles");
    println!("Lists profiles available in this repository.");
    0
}

pub fn run(root: &Path, args: &[OsString]) -> i32 {
    match args {
        [] => {}
        [option] if option == "-h" || option == "--help" => return help(),
        _ => {
            eprintln!("Error: profiles does not accept arguments.");
            return 2;
        }
    }
    let inspector = LinuxInspector;
    let names = match inspector.profile_names(root) {
        Ok(names) => names,
        Err(error) => {
            eprintln!("Error: could not list profiles: {error}");
            return 1;
        }
    };
    println!("Available profiles:");
    if names.is_empty() {
        println!("  none");
    } else {
        for name in names {
            match doctor::read_profile(&inspector, root, &name) {
                Ok(profile) => println!("  {name:<34} {}", profile.description),
                Err(error) => {
                    eprintln!("Error: {error}");
                    return 1;
                }
            }
        }
    }
    0
}
