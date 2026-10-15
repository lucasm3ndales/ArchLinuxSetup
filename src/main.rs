mod adapters;
mod application;
mod cli;
mod commands;
mod project;
mod system;

fn main() {
    let args = std::env::args_os().skip(1).collect();
    let invocation = match cli::parse(args) {
        Ok(invocation) => invocation,
        Err(message) => {
            eprintln!("Error: {message}");
            eprintln!();
            eprintln!("{}", cli::USAGE);
            std::process::exit(2);
        }
    };
    let root = match project::find_root() {
        Ok(root) => root,
        Err(message) => {
            eprintln!("Error: {message}");
            std::process::exit(1);
        }
    };
    std::process::exit(commands::run(invocation, &root));
}
