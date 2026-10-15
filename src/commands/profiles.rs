use crate::adapters::linux::LinuxInspector;
use crate::application::doctor::{self, SystemInspector};
use std::ffi::OsString;
use std::path::Path;

pub fn help() -> i32 {
    println!("Uso: archsetup profiles");
    println!("Lista os perfis disponíveis neste repositório.");
    0
}

pub fn run(root: &Path, args: &[OsString]) -> i32 {
    match args {
        [] => {}
        [option] if option == "-h" || option == "--help" => return help(),
        _ => {
            eprintln!("Erro: profiles não aceita argumentos.");
            return 2;
        }
    }
    let inspector = LinuxInspector;
    let names = match inspector.profile_names(root) {
        Ok(names) => names,
        Err(error) => {
            eprintln!("Erro: não foi possível listar os perfis: {error}");
            return 1;
        }
    };
    println!("Perfis disponíveis:");
    if names.is_empty() {
        println!("  nenhum");
    } else {
        for name in names {
            match doctor::read_profile(&inspector, root, &name) {
                Ok(profile) => println!("  {name:<34} {}", profile.description),
                Err(error) => { eprintln!("Erro: {error}"); return 1; }
            }
        }
    }
    0
}
