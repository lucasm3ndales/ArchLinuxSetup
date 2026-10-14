mod all;
mod aur;
mod desktop;
mod doctor;
mod link;
mod pkg;
mod profiles;
mod restore;
mod unlink;

use crate::cli::{Command, Invocation, USAGE};
use std::path::Path;

pub fn run(invocation: Invocation, root: &Path) -> i32 {
    match invocation {
        Invocation::Usage => {
            println!("{USAGE}");
            0
        }
        Invocation::Help(Command::Link) => link::help(),
        Invocation::Help(Command::Unlink) => unlink::help(),
        Invocation::Help(Command::Doctor) => doctor::help(),
        Invocation::Help(Command::Profiles) => profiles::help(),
        Invocation::Help(Command::All) => all::help(),
        Invocation::Help(Command::Aur) => aur::help(),
        Invocation::Help(Command::Pkg) => pkg::help(),
        Invocation::Help(Command::Desktop) => desktop::help(),
        Invocation::Help(Command::Restore) => restore::help(),
        Invocation::Run(Command::Link, args) => link::run(root, &args),
        Invocation::Run(Command::Unlink, args) => unlink::run(root, &args),
        Invocation::Run(Command::Doctor, args) => doctor::run(root, &args),
        Invocation::Run(Command::Profiles, args) => profiles::run(root, &args),
        Invocation::Run(Command::All, args) => all::run(root, &args),
        Invocation::Run(Command::Aur, args) => aur::run(&args),
        Invocation::Run(Command::Pkg, args) => pkg::run(root, &args),
        Invocation::Run(Command::Desktop, args) => desktop::run(root, &args),
        Invocation::Run(Command::Restore, args) => restore::run(&args),
    }
}
