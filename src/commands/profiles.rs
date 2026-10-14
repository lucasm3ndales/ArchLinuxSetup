use crate::project;
use std::ffi::OsString;
use std::fs;
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
    let entries = match fs::read_dir(root.join("profiles")) {
        Ok(entries) => entries,
        Err(error) => {
            eprintln!("Erro: não foi possível listar os perfis: {error}");
            return 1;
        }
    };
    let mut names = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                eprintln!("Erro: não foi possível ler um perfil: {error}");
                return 1;
            }
        };
        let name = entry.file_name().to_string_lossy().into_owned();
        if project::valid_profile_name(&name) && entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            names.push(name);
        }
    }
    names.sort();
    println!("Perfis disponíveis:");
    if names.is_empty() {
        println!("  nenhum");
    } else {
        for name in names {
            println!("  {name}");
        }
    }
    0
}
