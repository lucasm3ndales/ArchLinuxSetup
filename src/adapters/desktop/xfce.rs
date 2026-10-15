use super::{LinuxDesktop, capture, maybe_output, run, success};
use crate::application::desktop::{self, Panel, PanelPlan};
use crate::system;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

impl LinuxDesktop {
    pub(super) fn panel_plan(&self) -> Result<PanelPlan, String> {
        let values =
            capture("xfconf-query", &["-c", "xfce4-panel", "-p", "/panels"]).map_err(|_| {
                "O painel do Xfce ainda não possui configuração. Abra uma sessão completa primeiro."
                    .to_owned()
            })?;
        let ids = parse_ids(&values);
        if ids.is_empty() {
            return Err("Nenhum painel existente foi encontrado.".to_owned());
        }
        let mut panels = Vec::new();
        for id in ids {
            let position_path = format!("/panels/panel-{id}/position");
            let plugins_path = format!("/panels/panel-{id}/plugin-ids");
            let position =
                maybe_output("xfconf-query", &["-c", "xfce4-panel", "-p", &position_path])
                    .unwrap_or_default();
            let plugins = maybe_output("xfconf-query", &["-c", "xfce4-panel", "-p", &plugins_path])
                .map(|value| parse_ids(&value))
                .unwrap_or_default();
            panels.push(Panel {
                id,
                position,
                plugins,
            });
        }
        let properties = capture("xfconf-query", &["-c", "xfce4-panel", "-l"])?;
        let mut max_id = 0;
        for line in properties.lines() {
            if let Some(rest) = line.strip_prefix("/plugins/plugin-") {
                let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
                if let Ok(id) = rest[..digits].parse::<u32>() {
                    max_id = max_id.max(id);
                }
            }
        }
        let top = panels
            .iter()
            .find(|panel| panel.position.starts_with("p=6;"))
            .or_else(|| {
                panels
                    .iter()
                    .find(|panel| !panel.position.starts_with("p=10;"))
            })
            .or_else(|| panels.first())
            .unwrap();
        max_id = top.plugins.iter().copied().fold(max_id, u32::max);
        let mut plugin_types = HashMap::new();
        for id in &top.plugins {
            let property = format!("/plugins/plugin-{id}");
            let kind = maybe_output("xfconf-query", &["-c", "xfce4-panel", "-p", &property])
                .unwrap_or_default();
            plugin_types.insert(*id, kind);
        }
        let legacy_path = format!("/panels/panel-{}/archlinuxsetup-keylocks-id", top.id);
        let kbdleds_path = format!("/panels/panel-{}/archlinuxsetup-kbdleds-id", top.id);
        let legacy = read_id(&legacy_path);
        let recorded = read_id(&kbdleds_path);
        desktop::plan_panels(&panels, &plugin_types, max_id, legacy, recorded)
    }

    pub(super) fn configure_xfce(&self) -> Result<(), String> {
        let plan = self.panel_plan()?;
        println!("\n==> Salvando configuração atual");
        let backup = self.backup_xfce()?;
        println!("Configuração anterior salva em {}", backup.display());
        println!("\n==> Configurando tema e painéis");
        let old_keylocks = self.home.join(".local/bin/archlinuxsetup-keylocks");
        if fs::symlink_metadata(&old_keylocks).is_ok() {
            let target = backup.join("archlinuxsetup-keylocks");
            run(
                "mv",
                &[
                    "--",
                    &old_keylocks.to_string_lossy(),
                    &target.to_string_lossy(),
                ],
            )?;
            println!("Indicador antigo preservado em {}", target.display());
        }
        set_property("xsettings", "/Net/ThemeName", "string", "Nordic")?;
        set_property("xsettings", "/Net/IconThemeName", "string", "zafiro-dark")?;
        set_property(
            "xsettings",
            "/Gtk/CursorThemeName",
            "string",
            "Hackneyed",
        )?;
        set_property("xsettings", "/Gtk/CursorThemeSize", "int", "24")?;
        set_property("xsettings", "/Gtk/FontName", "string", "Inter 10")?;
        set_property("xfwm4", "/general/theme", "string", "Nordic")?;
        self.configure_cursor_files(&backup)?;
        set_property(
            "xfwm4",
            "/general/title_font",
            "string",
            "Inter SemiBold 10",
        )?;
        set_property(
            "xfce4-keyboard-shortcuts",
            "/commands/custom/<Alt>F1",
            "string",
            plan.menu_command,
        )?;

        for id in &plan.remove_panels {
            let path = format!("/panels/panel-{id}");
            run(
                "xfconf-query",
                &["-c", "xfce4-panel", "-p", &path, "--recursive", "--reset"],
            )?;
        }
        if !plan.remove_panels.is_empty() {
            set_array("xfce4-panel", "/panels", "int", &[plan.top_id.to_string()])?;
        }
        let panel = format!("/panels/panel-{}", plan.top_id);
        set_property(
            "xfce4-panel",
            &format!("{panel}/position"),
            "string",
            "p=6;x=0;y=0",
        )?;
        set_property("xfce4-panel", &format!("{panel}/length"), "uint", "100")?;
        set_property("xfce4-panel", &format!("{panel}/size"), "uint", "48")?;
        set_property(
            "xfce4-panel",
            &format!("{panel}/autohide-behavior"),
            "uint",
            "0",
        )?;
        set_property(
            "xfce4-panel",
            &format!("{panel}/position-locked"),
            "bool",
            "true",
        )?;
        let menu = format!("/plugins/plugin-{}", plan.menu_id);
        set_property("xfce4-panel", &menu, "string", plan.menu_type)?;
        set_property(
            "xfce4-panel",
            &format!("{menu}/button-icon"),
            "string",
            "/usr/share/pixmaps/archlinux-logo.svg",
        )?;
        set_property(
            "xfce4-panel",
            &format!("{menu}/show-button-title"),
            "bool",
            "false",
        )?;
        set_property(
            "xfce4-panel",
            &format!("/plugins/plugin-{}", plan.search_id),
            "string",
            "verve",
        )?;
        let tasklist = format!("/plugins/plugin-{}", plan.tasklist_id);
        set_property("xfce4-panel", &tasklist, "string", "tasklist")?;
        set_property(
            "xfce4-panel",
            &format!("{tasklist}/grouping"),
            "bool",
            "true",
        )?;
        set_property(
            "xfce4-panel",
            &format!("{tasklist}/include-all-workspaces"),
            "bool",
            "false",
        )?;
        let pager = format!("/plugins/plugin-{}", plan.pager_id);
        set_property("xfce4-panel", &pager, "string", "pager")?;
        set_property(
            "xfce4-panel",
            &format!("{pager}/miniature-view"),
            "bool",
            "false",
        )?;
        set_property("xfce4-panel", &format!("{pager}/rows"), "uint", "1")?;
        set_property(
            "xfce4-panel",
            &format!("{pager}/numbering"),
            "bool",
            "false",
        )?;
        if let Some(count) = maybe_output(
            "xfconf-query",
            &["-c", "xfwm4", "-p", "/general/workspace_count"],
        )
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|count| (1..=20).contains(count))
        {
            let names = (1..=count)
                .map(|index| index.to_string())
                .collect::<Vec<_>>();
            set_array("xfwm4", "/general/workspace_names", "string", &names)?;
        }
        if let Some(id) = plan.legacy_genmon_id {
            let path = format!("/plugins/plugin-{id}");
            run(
                "xfconf-query",
                &["-c", "xfce4-panel", "-p", &path, "--recursive", "--reset"],
            )?;
        }
        if plan.remove_legacy_marker {
            let path = format!("{panel}/archlinuxsetup-keylocks-id");
            run(
                "xfconf-query",
                &["-c", "xfce4-panel", "-p", &path, "--reset"],
            )?;
        }
        set_property(
            "xfce4-panel",
            &format!("/plugins/plugin-{}", plan.keylock_id),
            "string",
            "kbdleds",
        )?;
        set_property(
            "xfce4-panel",
            &format!("{panel}/archlinuxsetup-kbdleds-id"),
            "int",
            &plan.keylock_id.to_string(),
        )?;
        let clock = format!("/plugins/plugin-{}", plan.clock_id);
        set_property("xfce4-panel", &clock, "string", "clock")?;
        set_property("xfce4-panel", &format!("{clock}/mode"), "uint", "2")?;
        set_property(
            "xfce4-panel",
            &format!("{clock}/digital-format"),
            "string",
            "%d/%m %H:%M",
        )?;
        let ids = plan
            .plugin_order
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>();
        set_array("xfce4-panel", &format!("{panel}/plugin-ids"), "int", &ids)?;
        run("xfce4-panel", &["--restart"])?;
        println!("\n==> Ativando a tecla Super no Xfce");
        self.configure_ksuperkey(&backup)?;
        println!(
            "\nConcluído: painel único, busca, tarefas do espaço atual, relógio com data e indicadores configurados."
        );
        Ok(())
    }

    fn backup_xfce(&self) -> Result<PathBuf, String> {
        let base = self.state_dir.join("backups");
        fs::create_dir_all(&base)
            .map_err(|error| format!("não foi possível criar {}: {error}", base.display()))?;
        let date = capture("date", &["+%Y%m%d-%H%M%S"])?.trim().to_owned();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .subsec_nanos();
        let backup = base.join(format!("xfce-{date}-{}-{nanos:09}", std::process::id()));
        fs::create_dir(&backup).map_err(|error| {
            format!(
                "não foi possível criar backup {}: {error}",
                backup.display()
            )
        })?;
        for channel in [
            "xsettings",
            "xfwm4",
            "xfce4-panel",
            "xfce4-keyboard-shortcuts",
        ] {
            let listing = capture("xfconf-query", &["-c", channel, "-lv"]).unwrap_or_default();
            fs::write(backup.join(format!("{channel}.txt")), listing)
                .map_err(|error| format!("não foi possível registrar {channel}: {error}"))?;
            let source = self
                .config_dir
                .join("xfce4/xfconf/xfce-perchannel-xml")
                .join(format!("{channel}.xml"));
            if source.is_file() {
                fs::copy(&source, backup.join(format!("{channel}.xml"))).map_err(|error| {
                    format!("não foi possível salvar {}: {error}", source.display())
                })?;
            }
        }
        for (source, backup_name, rendered) in [
            (
                self.home.join(".Xresources"),
                "Xresources.original",
                desktop::render_xresources as fn(&str) -> String,
            ),
            (
                self.home.join(".icons/default/index.theme"),
                "cursor-index.original",
                desktop::render_cursor_index as fn(&str) -> String,
            ),
        ] {
            if source.is_file() {
                let current = fs::read_to_string(&source)
                    .map_err(|error| format!("não foi possível ler {}: {error}", source.display()))?;
                if rendered(&current) != current {
                    fs::copy(&source, backup.join(backup_name)).map_err(|error| {
                        format!("não foi possível salvar {}: {error}", source.display())
                    })?;
                }
            }
        }
        Ok(backup)
    }

    fn configure_cursor_files(&self, backup: &Path) -> Result<(), String> {
        for (target, rendered, created_name) in [
            (
                self.home.join(".Xresources"),
                desktop::render_xresources as fn(&str) -> String,
                "Xresources.created",
            ),
            (
                self.home.join(".icons/default/index.theme"),
                desktop::render_cursor_index as fn(&str) -> String,
                "cursor-index.created",
            ),
        ] {
            let existed = target.is_file();
            let current = if existed {
                fs::read_to_string(&target).map_err(|error| {
                    format!("não foi possível ler {}: {error}", target.display())
                })?
            } else {
                String::new()
            };
            let updated = rendered(&current);
            if updated != current {
                fs::create_dir_all(target.parent().ok_or("caminho de cursor inválido")?)
                    .map_err(|error| format!("não foi possível criar diretório: {error}"))?;
                if !existed {
                    fs::write(backup.join(created_name), &updated).map_err(|error| {
                        format!("não foi possível registrar {}: {error}", target.display())
                    })?;
                }
                fs::write(&target, &updated).map_err(|error| {
                    format!("não foi possível configurar {}: {error}", target.display())
                })?;
            }
        }
        Ok(())
    }

    fn configure_ksuperkey(&self, backup: &Path) -> Result<(), String> {
        let target = self.config_dir.join("autostart/ksuperkey.desktop");
        if fs::symlink_metadata(&target).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
            return Err(format!("Link simbólico inesperado: {}", target.display()));
        }
        let current = fs::read_to_string(&target).ok();
        if current.as_deref() != Some(desktop::AUTOSTART) {
            fs::create_dir_all(target.parent().unwrap()).map_err(|error| error.to_string())?;
            if target.is_file() {
                fs::copy(&target, backup.join("ksuperkey.desktop"))
                    .map_err(|error| format!("não foi possível salvar autostart: {error}"))?;
            }
            fs::write(&target, desktop::AUTOSTART)
                .map_err(|error| format!("não foi possível configurar autostart: {error}"))?;
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&target, fs::Permissions::from_mode(0o644)).map_err(|error| {
                format!("não foi possível ajustar permissões de autostart: {error}")
            })?;
        }
        let uid = system::effective_uid()?.to_string();
        if !success("pgrep", &["-u", &uid, "-x", "ksuperkey"])? {
            Command::new("ksuperkey")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|error| format!("não foi possível iniciar ksuperkey: {error}"))?;
        }
        Ok(())
    }
}

fn parse_ids(output: &str) -> Vec<u32> {
    output
        .lines()
        .filter_map(|line| line.parse::<u32>().ok())
        .collect()
}

fn read_id(path: &str) -> Option<u32> {
    maybe_output("xfconf-query", &["-c", "xfce4-panel", "-p", path])?
        .parse()
        .ok()
}

fn set_property(channel: &str, property: &str, kind: &str, value: &str) -> Result<(), String> {
    run(
        "xfconf-query",
        &[
            "-c", channel, "-p", property, "--create", "--type", kind, "--set", value,
        ],
    )
}

fn set_array(channel: &str, property: &str, kind: &str, values: &[String]) -> Result<(), String> {
    let mut args = vec![
        "-c".to_owned(),
        channel.to_owned(),
        "-p".to_owned(),
        property.to_owned(),
        "--create".to_owned(),
        "--force-array".to_owned(),
    ];
    for value in values {
        args.extend([
            "--type".to_owned(),
            kind.to_owned(),
            "--set".to_owned(),
            value.clone(),
        ]);
    }
    let status = Command::new("xfconf-query")
        .args(&args)
        .status()
        .map_err(|error| format!("não foi possível executar xfconf-query: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("xfconf-query falhou ao configurar array".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_only_numeric_panel_ids() {
        assert_eq!(
            parse_ids("Value is an array with 2 items:\n0\n1\n"),
            vec![0, 1]
        );
    }
}
