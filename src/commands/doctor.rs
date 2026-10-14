use crate::adapters::linux::LinuxInspector;
use crate::application::doctor::{self as application_doctor, Message};
use crate::project;
use crate::system;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Default, Eq, PartialEq)]
struct Options {
    profile: Option<String>,
    verbose: bool,
}

fn parse_args(args: &[OsString]) -> Result<Option<Options>, String> {
    let mut options = Options::default();
    let mut index = 0;
    while index < args.len() {
        let argument = args[index].to_string_lossy();
        match argument.as_ref() {
            "--profile" => {
                index += 1;
                let name = args.get(index).ok_or("falta o nome após --profile")?;
                options.profile = Some(name.to_string_lossy().into_owned());
            }
            "--verbose" => options.verbose = true,
            "-h" | "--help" => return Ok(None),
            _ => return Err(format!("opção desconhecida em doctor: {argument}")),
        }
        index += 1;
    }
    Ok(Some(options))
}

pub fn help() -> i32 {
    println!("Uso: archsetup doctor [--profile NOME] [--verbose]");
    println!();
    println!("Confere o sistema sem alterar arquivos. --verbose também mostra o diagnóstico");
    println!("detalhado de hardware, discos, sessão e serviços.");
    println!("Informa se o Xfce está instalado e se a sessão X11 do Xfce está ativa.");
    0
}

pub fn run(root: &Path, args: &[OsString]) -> i32 {
    let options = match parse_args(args) {
        Ok(Some(options)) => options,
        Ok(None) => return help(),
        Err(message) => {
            eprintln!("Erro: {message}");
            return 2;
        }
    };
    if let Some(profile) = &options.profile {
        if !project::valid_profile_name(profile) || !root.join("profiles").join(profile).is_dir() {
            eprintln!("Erro: perfil inexistente ou inválido: {profile}.");
            return 2;
        }
    }

    let (failed, ordinary_user) = summary(root, &options);
    if options.verbose && ordinary_user {
        println!();
        details();
    }
    i32::from(failed)
}

fn summary(root: &Path, options: &Options) -> (bool, bool) {
    let inspector = LinuxInspector;
    let report = application_doctor::inspect(&inspector, root, options.profile.as_deref());
    for message in report.messages {
        match message {
            Message::Info(line) => println!("{line}"),
            Message::Error(line) => eprintln!("{line}"),
        }
    }
    (report.failed, report.ordinary_user)
}

fn mount_value(mountpoint: &str, field: &str) -> Option<String> {
    system::output("findmnt", &["-rnM", mountpoint, "-o", field]).filter(|value| !value.is_empty())
}

fn section(name: &str) {
    println!("\n== {name} ==");
}

fn details() {
    println!("==> Diagnóstico do Arch e do hardware");
    println!("Somente leitura. Revise UUIDs e dados do equipamento antes de compartilhar a saída.");

    section("Sistema e sessão");
    println!(
        "Kernel: {}",
        system::output("uname", &["-r"]).unwrap_or_default()
    );
    println!(
        "Hostname: {}",
        system::output("hostname", &[]).unwrap_or_default()
    );
    println!(
        "Sessão: {}",
        std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "desconhecida".to_owned())
    );
    println!(
        "Desktop: {}",
        std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "desconhecido".to_owned())
    );
    if system::command_exists("timedatectl") {
        let timezone = system::output("timedatectl", &["show", "-p", "Timezone", "--value"]);
        println!(
            "Fuso horário: {}",
            timezone
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "indisponível".to_owned())
        );
    }
    if let Some(locale) = system::read_trimmed(Path::new("/etc/locale.conf")) {
        if let Some(language) = locale.lines().find_map(|line| line.strip_prefix("LANG=")) {
            println!("Locale: {language}");
        }
    }
    if let Some(console) = system::read_trimmed(Path::new("/etc/vconsole.conf")) {
        println!("Console: {}", console.replace('\n', " "));
    }
    for field in [
        "sys_vendor",
        "product_name",
        "product_version",
        "bios_version",
    ] {
        if let Some(value) = system::read_trimmed(&Path::new("/sys/class/dmi/id").join(field)) {
            println!("{field}: {value}");
        }
    }

    section("Processador");
    if let Some(cpu) = system::output("lscpu", &[]) {
        for line in cpu.lines() {
            if line.starts_with("Architecture:") || line.starts_with("Model name:") {
                println!("{line}");
            }
        }
    } else {
        println!("Comando indisponível: lscpu");
    }

    section("Placas de vídeo e drivers em uso");
    system::run_optional("lspci", &["-nnk", "-d", "::03xx"]);
    section("Rede, áudio e armazenamento PCI");
    for class in ["0280", "0200", "0403", "0108"] {
        system::run_optional("lspci", &["-nnk", "-d", &format!("::{class}")]);
    }
    section("Dispositivos USB e bloqueios de rádio");
    system::run_optional("lsusb", &[]);
    system::run_optional("rfkill", &["list"]);

    section(
        "Discos e sistema de arquivos (confira modelo, UUID e partições antes de qualquer alteração)",
    );
    system::run_optional(
        "lsblk",
        &["-o", "NAME,MODEL,SIZE,TYPE,FSTYPE,UUID,MOUNTPOINTS"],
    );
    system::run_optional("findmnt", &["-no", "SOURCE,FSTYPE,OPTIONS", "/"]);
    system::run_optional("findmnt", &["-no", "SOURCE,FSTYPE,OPTIONS", "/home"]);
    system::run_optional("swapon", &["--show"]);
    system::run_optional("zramctl", &[]);

    section("Entrada e touchpad");
    if let Ok(devices) = fs::read_to_string("/proc/bus/input/devices") {
        for line in devices.lines().filter(|line| line.starts_with("N: Name=")) {
            println!("{line}");
        }
    }
    if std::env::var("XDG_SESSION_TYPE").as_deref() == Ok("x11")
        && std::env::var_os("DISPLAY").is_some()
    {
        system::run_optional("xinput", &["list"]);
        if system::command_exists("xfconf-query") {
            let _ = Command::new("xfconf-query")
                .args(["-c", "pointers", "-lv"])
                .status();
        }
    } else {
        println!("Abra uma sessão X11 para consultar xinput e as opções do Xfce.");
    }

    section("Módulos e iluminação");
    if let Some(modules) = system::output("lsmod", &[]) {
        const MODULES: &[&str] = &[
            "acer_wmi",
            "i2c_hid",
            "hid_multitouch",
            "iwlwifi",
            "ath10k_pci",
            "snd_hda_intel",
            "nvidia",
            "nouveau",
            "i915",
        ];
        for (index, line) in modules.lines().enumerate() {
            if index == 0 || MODULES.contains(&line.split_whitespace().next().unwrap_or("")) {
                println!("{line}");
            }
        }
    }
    if let Ok(entries) = fs::read_dir("/sys/class/leds") {
        for entry in entries.flatten() {
            println!("{}", entry.file_name().to_string_lossy());
        }
    }

    section("Pacotes relevantes instalados");
    if system::command_exists("pacman") {
        const PACKAGES: &[&str] = &[
            "linux",
            "linux-headers",
            "linux-zen",
            "linux-zen-headers",
            "dkms",
            "grub",
            "grub-btrfs",
            "mesa",
            "vulkan-intel",
            "intel-media-driver",
            "xf86-video-intel",
            "xf86-input-libinput",
            "xf86-input-synaptics",
            "nvidia",
            "nvidia-open",
            "nvidia-dkms",
            "nvidia-580xx-dkms",
            "nvidia-utils",
            "nvidia-prime",
            "btrfs-progs",
            "timeshift",
            "cronie",
            "zram-generator",
            "ly",
            "tlp",
            "power-profiles-daemon",
        ];
        for package in PACKAGES {
            if system::installed(package) {
                system::run_optional("pacman", &["-Q", package]);
            }
        }
    } else {
        println!("Comando indisponível: pacman");
    }

    section("Boot e snapshots");
    if system::command_exists("findmnt") {
        for mountpoint in ["/boot", "/boot/efi", "/efi"] {
            if mount_value(mountpoint, "FSTYPE").is_some() {
                print!("{mountpoint}: ");
                system::run_optional(
                    "findmnt",
                    &["-rnM", mountpoint, "-o", "SOURCE,FSTYPE,OPTIONS"],
                );
            }
        }
    } else {
        println!("Comando indisponível: findmnt");
    }

    section("Serviços de login e rede");
    if system::command_exists("systemctl") {
        for service in [
            "ly@tty1.service",
            "getty@tty1.service",
            "lightdm.service",
            "NetworkManager.service",
        ] {
            let state = system::output_any_status("systemctl", &["is-enabled", service])
                .unwrap_or_else(|| "indisponível".to_owned());
            println!("{service}: {state}");
        }
    }
    section("Serviços de energia");
    if system::command_exists("systemctl") {
        for service in ["tlp.service", "power-profiles-daemon.service"] {
            let state = system::output_any_status("systemctl", &["is-active", service])
                .unwrap_or_else(|| "indisponível".to_owned());
            println!("{service}: {state}");
        }
    } else {
        println!("Comando indisponível: systemctl");
    }
    println!("\nConcluído: diagnóstico exibido. Nenhuma alteração realizada.");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn parses_options_and_rejects_unknown() {
        assert_eq!(
            parse_args(&args(&["--verbose", "--profile", "acer"])),
            Ok(Some(Options {
                profile: Some("acer".to_owned()),
                verbose: true,
            }))
        );
        assert_eq!(parse_args(&args(&["--help"])), Ok(None));
        assert!(parse_args(&args(&["--profile"])).is_err());
        assert!(parse_args(&args(&["--bad"])).is_err());
    }
}
