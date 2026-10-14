use std::ffi::OsString;

pub const USAGE: &str = "Uso: archsetup <comando> [opções]

Comandos:
  link                  Cria o comando archsetup em ~/.local/bin
  unlink                Remove apenas o comando e sua linha PATH
  restore               Recupera configurações; --packages inclui pacotes registrados
  doctor                Confere o sistema; --verbose mostra o diagnóstico completo
  aur                   Instala o auxiliar paru para a AUR
  pkg [categorias]      Instala pacotes oficiais e AUR
  desktop               Configura o desktop (atualmente só Xfce)
  profiles              Lista os perfis disponíveis
  all                   Executa aur, pkg e desktop (requer Xfce)

Use --dry-run em link, unlink, restore, aur, pkg, desktop ou all para ver a ação sem executá-la.
Use archsetup help <comando> para ver opções e exemplos.";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Command {
    Link,
    Unlink,
    Restore,
    Doctor,
    Aur,
    Pkg,
    Desktop,
    Profiles,
    All,
}

impl Command {
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "link" => Self::Link,
            "unlink" => Self::Unlink,
            "restore" => Self::Restore,
            "doctor" => Self::Doctor,
            "aur" => Self::Aur,
            "pkg" => Self::Pkg,
            "desktop" => Self::Desktop,
            "profiles" => Self::Profiles,
            "all" => Self::All,
            _ => return None,
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Invocation {
    Usage,
    Help(Command),
    Run(Command, Vec<OsString>),
}

pub fn parse(mut args: Vec<OsString>) -> Result<Invocation, String> {
    if args.is_empty() {
        return Ok(Invocation::Usage);
    }
    let first = args.remove(0);
    let first = first
        .to_str()
        .ok_or_else(|| "nome de comando inválido".to_owned())?;
    if matches!(first, "help" | "-h" | "--help") {
        return match args.len() {
            0 => Ok(Invocation::Usage),
            1 if first == "help" => {
                let name = args[0]
                    .to_str()
                    .ok_or_else(|| "nome de comando inválido".to_owned())?;
                Command::from_name(name)
                    .map(Invocation::Help)
                    .ok_or_else(|| format!("comando desconhecido: {name}"))
            }
            _ => Err("help aceita somente um comando".to_owned()),
        };
    }
    let command =
        Command::from_name(first).ok_or_else(|| format!("comando desconhecido: {first}"))?;
    Ok(Invocation::Run(command, args))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn parses_help_and_commands() {
        assert_eq!(parse(args(&[])), Ok(Invocation::Usage));
        assert_eq!(
            parse(args(&["help", "doctor"])),
            Ok(Invocation::Help(Command::Doctor))
        );
        assert_eq!(
            parse(args(&["pkg", "gaming", "--dry-run"])),
            Ok(Invocation::Run(
                Command::Pkg,
                args(&["gaming", "--dry-run"])
            ))
        );
    }

    #[test]
    fn rejects_unknown_or_extra_commands() {
        assert!(parse(args(&["check"])).is_err());
        assert!(parse(args(&["install"])).is_err());
        assert!(parse(args(&["uninstall"])).is_err());
        assert!(parse(args(&["packages"])).is_err());
        assert!(parse(args(&["help", "doctor", "extra"])).is_err());
        assert!(parse(args(&["--help", "doctor"])).is_err());
    }
}
