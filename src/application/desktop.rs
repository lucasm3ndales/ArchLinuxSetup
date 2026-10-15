use std::collections::HashMap;

pub const REQUIRED_PACKAGES: &[&str] = &[
    "xf86-input-libinput",
    "inter-font",
    "ttf-fira-code",
    "ksuperkey",
    "nordic-theme",
    "zafiro-icon-theme",
    "xcursor-hackneyed-light",
    "xfce4-kbdleds-plugin",
];

#[derive(Clone, Copy)]
pub struct XfceFacts<'a> {
    pub session_type: Option<&'a str>,
    pub current_desktop: Option<&'a str>,
    pub display_available: bool,
    pub xfce_session_installed: bool,
    pub xfce_panel_installed: bool,
}

pub fn check_xfce(facts: XfceFacts<'_>) -> Result<(), String> {
    let mut missing = Vec::new();
    if !facts.xfce_session_installed {
        missing.push("xfce4-session");
    }
    if !facts.xfce_panel_installed {
        missing.push("xfce4-panel");
    }
    if !missing.is_empty() {
        return Err(format!(
            "Xfce não está instalado (faltam {}). Instale-o antes de executar desktop ou all.",
            missing.join(" e ")
        ));
    }
    if facts.session_type != Some("x11") || !facts.display_available {
        return Err("Abra uma sessão X11 do Xfce antes de executar desktop ou all.".to_owned());
    }
    if !facts.current_desktop.is_some_and(|desktop| {
        desktop
            .split(':')
            .any(|name| name.eq_ignore_ascii_case("xfce"))
    }) {
        return Err(
            "A sessão atual não é Xfce. Entre no Xfce antes de executar desktop ou all.".to_owned(),
        );
    }
    Ok(())
}

pub const AUTOSTART: &str = "[Desktop Entry]\nType=Application\nName=ArchLinuxSetup Super key\nComment=Abre o menu de aplicativos ao tocar a tecla Super\nExec=ksuperkey\nOnlyShowIn=XFCE;\nTerminal=false\n";
pub const SEAT_CONFIG: &str = "[Seat:*]\ngreeter-session=lightdm-slick-greeter\n";
pub const NATURAL_SCROLL: &str = "Section \"InputClass\"\n    Identifier \"ArchLinuxSetup touchpad natural scroll\"\n    MatchIsTouchpad \"on\"\n    MatchDevicePath \"/dev/input/event*\"\n    Driver \"libinput\"\n    Option \"NaturalScrolling\" \"true\"\nEndSection\n";
pub const TAPPING: &str = "Section \"InputClass\"\n    Identifier \"ArchLinuxSetup Acer touchpad tap-to-click\"\n    MatchIsTouchpad \"on\"\n    MatchDevicePath \"/dev/input/event*\"\n    Driver \"libinput\"\n    Option \"Tapping\" \"true\"\nEndSection\n";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Panel {
    pub id: u32,
    pub position: String,
    pub plugins: Vec<u32>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct PanelPlan {
    pub top_id: u32,
    pub remove_panels: Vec<u32>,
    pub menu_id: u32,
    pub menu_type: &'static str,
    pub menu_command: &'static str,
    pub search_id: u32,
    pub tasklist_id: u32,
    pub pager_id: u32,
    pub keylock_id: u32,
    pub clock_id: u32,
    pub legacy_genmon_id: Option<u32>,
    pub remove_legacy_marker: bool,
    pub plugin_order: Vec<u32>,
}

pub fn plan_panels(
    panels: &[Panel],
    plugin_types: &HashMap<u32, String>,
    max_plugin_id: u32,
    legacy_keylock: Option<u32>,
    recorded_kbdleds: Option<u32>,
) -> Result<PanelPlan, String> {
    let top = panels
        .iter()
        .find(|panel| panel.position.starts_with("p=6;"))
        .or_else(|| {
            panels
                .iter()
                .find(|panel| !panel.position.starts_with("p=10;"))
        })
        .or_else(|| panels.first())
        .ok_or("Nenhum painel existente foi encontrado.")?;
    let remove_panels = panels
        .iter()
        .filter(|panel| panel.id != top.id)
        .map(|panel| panel.id)
        .collect();
    let mut next_id = max_plugin_id;
    let kind = |id: &u32| plugin_types.get(id).map(String::as_str).unwrap_or("");
    let mut allocate = |name: &str| -> Result<u32, String> {
        if let Some(id) = top.plugins.iter().find(|id| kind(id) == name) {
            return Ok(*id);
        }
        next_id = next_id
            .checked_add(1)
            .ok_or("Limite de plugins do Xfce excedido")?;
        Ok(next_id)
    };
    let existing_whisker = top
        .plugins
        .iter()
        .find(|id| kind(id) == "whiskermenu")
        .copied();
    let (menu_id, menu_type, menu_command) = if let Some(id) = existing_whisker {
        (id, "whiskermenu", "xfce4-popup-whiskermenu")
    } else {
        (
            allocate("applicationsmenu")?,
            "applicationsmenu",
            "xfce4-popup-applicationsmenu",
        )
    };
    let search_id = allocate("verve")?;
    let tasklist_id = allocate("tasklist")?;
    let pager_id = allocate("pager")?;
    let clock_id = allocate("clock")?;
    let legacy_genmon_id =
        legacy_keylock.filter(|id| top.plugins.contains(id) && kind(id) == "genmon");
    let existing_keylock = recorded_kbdleds
        .filter(|id| top.plugins.contains(id) && kind(id) == "kbdleds")
        .or_else(|| top.plugins.iter().find(|id| kind(id) == "kbdleds").copied())
        .or(legacy_genmon_id);
    let keylock_id = if let Some(id) = existing_keylock {
        id
    } else {
        next_id = next_id
            .checked_add(1)
            .ok_or("Limite de plugins do Xfce excedido")?;
        next_id
    };
    let mut plugin_order = vec![menu_id, search_id, tasklist_id];
    let mut right = Vec::new();
    for id in &top.plugins {
        if [
            menu_id,
            search_id,
            tasklist_id,
            pager_id,
            keylock_id,
            clock_id,
        ]
        .contains(id)
            || Some(*id) == legacy_genmon_id
        {
            continue;
        }
        match kind(id) {
            "applicationsmenu" | "whiskermenu" | "docklike" | "clock" => {}
            "systray" | "statusnotifier" | "actions" | "pulseaudio" | "notification-plugin" => {
                right.push(*id)
            }
            _ => plugin_order.push(*id),
        }
    }
    plugin_order.extend([pager_id, keylock_id]);
    plugin_order.extend(right);
    plugin_order.push(clock_id);
    Ok(PanelPlan {
        top_id: top.id,
        remove_panels,
        menu_id,
        menu_type,
        menu_command,
        search_id,
        tasklist_id,
        pager_id,
        keylock_id,
        clock_id,
        legacy_genmon_id,
        remove_legacy_marker: legacy_keylock.is_some(),
        plugin_order,
    })
}

const GREETER_SETTINGS: &[(&str, &str)] = &[
    ("theme-name", "Nordic"),
    ("icon-theme-name", "zafiro-dark"),
    ("cursor-theme-name", "Hackneyed"),
    ("cursor-theme-size", "24"),
    ("font-name", "Inter 10"),
    ("background", "#2e3440"),
    ("background-color", "#2e3440"),
    ("draw-user-backgrounds", "false"),
    ("logo", "/usr/share/pixmaps/archlinux-logo.svg"),
];

const LY_SETTINGS: &[(&str, &str)] = &[
    ("animation", "matrix"),
    ("clock", "%d/%m/%Y %H:%M"),
];

pub fn render_ly(existing: &str) -> String {
    let mut output = String::new();
    let mut seen = Vec::new();
    for line in existing.lines() {
        let setting = line
            .trim_start()
            .split_once('=')
            .map(|(key, _)| key.trim());
        if let Some((key, value)) = LY_SETTINGS.iter().find(|(key, _)| Some(*key) == setting) {
            if !seen.contains(key) {
                output.push_str(&format!("{key} = {value}\n"));
                seen.push(*key);
            }
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }
    for (key, value) in LY_SETTINGS {
        if !seen.contains(key) {
            output.push_str(&format!("{key} = {value}\n"));
        }
    }
    output
}

pub fn render_greeter(existing: &str) -> String {
    let mut output = String::new();
    let mut in_greeter = false;
    let mut found = false;
    for line in existing.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if in_greeter {
                emit_settings(&mut output);
            }
            in_greeter = trimmed == "[Greeter]";
            found |= in_greeter;
            output.push_str(line);
            output.push('\n');
        } else if in_greeter
            && GREETER_SETTINGS.iter().any(|(name, _)| {
                trimmed
                    .split_once('=')
                    .is_some_and(|(key, _)| key.trim() == *name)
            })
        {
            continue;
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }
    if in_greeter {
        emit_settings(&mut output);
    }
    if !found {
        if !output.is_empty() {
            output.push('\n');
        }
        output.push_str("[Greeter]\n");
        emit_settings(&mut output);
    }
    output
}

pub fn render_xresources(existing: &str) -> String {
    let mut output = String::new();
    for line in existing.lines() {
        if line
            .trim_start()
            .split_once(':')
            .is_some_and(|(key, _)| key.trim() == "Xcursor.theme")
        {
            continue;
        }
        output.push_str(line);
        output.push('\n');
    }
    output.push_str("Xcursor.theme: Hackneyed\n");
    output
}

pub fn render_cursor_index(existing: &str) -> String {
    let mut output = String::new();
    let mut in_icon_theme = false;
    let mut found = false;
    for line in existing.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if in_icon_theme {
                output.push_str("Inherits=Hackneyed\n");
            }
            in_icon_theme = trimmed == "[Icon Theme]";
            found |= in_icon_theme;
            output.push_str(line);
            output.push('\n');
        } else if in_icon_theme
            && trimmed
                .split_once('=')
                .is_some_and(|(key, _)| key.trim() == "Inherits")
        {
            continue;
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }
    if in_icon_theme {
        output.push_str("Inherits=Hackneyed\n");
    }
    if !found {
        if !output.is_empty() {
            output.push('\n');
        }
        output.push_str("[Icon Theme]\nInherits=Hackneyed\n");
    }
    output
}

fn emit_settings(output: &mut String) {
    for (name, value) in GREETER_SETTINGS {
        output.push_str(name);
        output.push('=');
        output.push_str(value);
        output.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xfce_requires_installation_and_active_x11_session() {
        let ready = XfceFacts {
            session_type: Some("x11"),
            current_desktop: Some("XFCE"),
            display_available: true,
            xfce_session_installed: true,
            xfce_panel_installed: true,
        };
        assert!(check_xfce(ready).is_ok());
        assert!(
            check_xfce(XfceFacts {
                xfce_panel_installed: false,
                ..ready
            })
            .unwrap_err()
            .contains("xfce4-panel")
        );
        assert!(
            check_xfce(XfceFacts {
                current_desktop: Some("KDE"),
                ..ready
            })
            .unwrap_err()
            .contains("não é Xfce")
        );
        assert!(
            check_xfce(XfceFacts {
                session_type: Some("wayland"),
                ..ready
            })
            .unwrap_err()
            .contains("X11")
        );
        assert!(
            check_xfce(XfceFacts {
                display_available: false,
                ..ready
            })
            .is_err()
        );
        assert!(
            check_xfce(XfceFacts {
                current_desktop: Some("NOTXFCE"),
                ..ready
            })
            .is_err()
        );
    }

    #[test]
    fn selects_top_panel_and_reuses_plugins() {
        let panels = vec![
            Panel {
                id: 1,
                position: "p=6;x=0;y=0".into(),
                plugins: vec![1, 2, 3, 4, 5, 6, 7],
            },
            Panel {
                id: 2,
                position: "p=10;x=0;y=0".into(),
                plugins: vec![],
            },
        ];
        let types = HashMap::from([
            (1, "applicationsmenu".into()),
            (2, "verve".into()),
            (3, "tasklist".into()),
            (4, "pager".into()),
            (5, "kbdleds".into()),
            (6, "systray".into()),
            (7, "clock".into()),
        ]);
        let plan = plan_panels(&panels, &types, 7, None, Some(5)).unwrap();
        assert_eq!(plan.top_id, 1);
        assert_eq!(plan.remove_panels, vec![2]);
        assert_eq!(plan.plugin_order, vec![1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(plan.keylock_id, 5);
    }

    #[test]
    fn renders_greeter_without_duplicating_settings() {
        let original =
            "[Other]\nfoo=bar\n[Greeter]\ntheme-name=Old\ncustom=true\n[Seat:*]\nactive=true\n";
        let updated = render_greeter(original);
        assert_eq!(updated.matches("theme-name=Nordic").count(), 1);
        assert!(updated.contains("custom=true\n"));
        assert!(updated.contains("[Seat:*]\nactive=true\n"));
        assert_eq!(render_greeter(&updated), updated);
    }

    #[test]
    fn renders_ly_settings_without_changing_other_options() {
        let original = "# animation = none\nanimation = none\nclock = null\nfont = foo\n";
        let updated = render_ly(original);
        assert!(updated.contains("# animation = none\n"));
        assert!(updated.contains("animation = matrix\n"));
        assert!(updated.contains("clock = %d/%m/%Y %H:%M\n"));
        assert!(updated.contains("font = foo\n"));
        assert_eq!(render_ly(&updated), updated);
    }

    #[test]
    fn cursor_files_keep_other_settings_and_remain_stable() {
        let resources = render_xresources("Xft.dpi: 120\nXcursor.theme: Old\n");
        assert_eq!(resources, "Xft.dpi: 120\nXcursor.theme: Hackneyed\n");
        assert_eq!(render_xresources(&resources), resources);

        let index = render_cursor_index("[Icon Theme]\nName=Custom\nInherits=Old\n[Other]\nFoo=bar\n");
        assert_eq!(
            index,
            "[Icon Theme]\nName=Custom\nInherits=Hackneyed\n[Other]\nFoo=bar\n"
        );
        assert_eq!(render_cursor_index(&index), index);
        assert_eq!(
            render_cursor_index(""),
            "[Icon Theme]\nInherits=Hackneyed\n"
        );
    }

    #[test]
    fn replaces_only_recorded_legacy_genmon_and_allocates_unique_plugins() {
        let panels = vec![Panel {
            id: 9,
            position: "p=6;x=0;y=0".into(),
            plugins: vec![2, 5, 18],
        }];
        let types = HashMap::from([
            (2, "whiskermenu".into()),
            (5, "genmon".into()),
            (18, "systray".into()),
        ]);
        let plan = plan_panels(&panels, &types, 18, Some(5), None).unwrap();
        assert_eq!(plan.menu_id, 2);
        assert_eq!(plan.menu_command, "xfce4-popup-whiskermenu");
        assert_eq!(plan.keylock_id, 5);
        assert_eq!(plan.legacy_genmon_id, Some(5));
        assert_eq!(plan.plugin_order, vec![2, 19, 20, 21, 5, 18, 22]);
        assert_eq!(
            plan.plugin_order.len(),
            plan.plugin_order
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len()
        );
    }
}
