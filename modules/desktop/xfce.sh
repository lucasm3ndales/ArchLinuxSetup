#!/usr/bin/env bash
set -euo pipefail
PROJECT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'EOF'
Uso: archsetup xfce [--dry-run]

Aplica um visual escuro discreto ao usuário atual em uma sessão X11 do Xfce.
Instale antes a categoria desktop: archsetup packages desktop
EOF
}

fail() {
  printf 'Erro: %s\n' "$1" >&2
  exit 1
}

dry_run=0
case "${1:-}" in
  '') ;;
  --dry-run) dry_run=1; shift ;;
  -h|--help) usage; exit 0 ;;
  *) usage >&2; exit 2 ;;
esac
(( $# == 0 )) || { usage >&2; exit 2; }

if (( dry_run )); then
  printf '==> Prévia do visual do Xfce\n'
else
  printf '==> Configuração do visual do Xfce\n'
fi
cat <<'EOF'
  GTK e bordas das janelas: Nordic
  Outras variantes instaladas pela categoria desktop: Nordic-darker, Nordic-bluish-accent, Nordic-Polar
  Ícones: Nordzy-dark
  Cursor: Nordzy-cursors, 24 px
  Fonte da interface e títulos: Inter 10
  Fonte para código disponível: Fira Code
  Painel único superior: 48 px, menu com logo do Arch, busca, tarefas agrupadas por aplicativo e espaços 1, 2, 3… à direita
  Tarefas: somente janelas do espaço atual; relógio: dia/mês e hora no canto direito
  Indicadores de Caps Lock, Num Lock e Scroll Lock por símbolos
  Super esquerdo: abre o menu de aplicativos ao tocar (ksuperkey)
EOF

if (( dry_run )); then
  printf '\nPrévia concluída. Nenhuma alteração realizada.\n'
  exit 0
fi

(( EUID != 0 )) || fail 'Execute como usuário comum, sem sudo.'
[[ -r /etc/os-release ]] || fail 'Não foi possível identificar a distribuição.'
os_id=$(awk -F= '$1 == "ID" { gsub(/"/, "", $2); print $2 }' /etc/os-release)
[[ "$os_id" == arch ]] || fail 'A aplicação do visual só é permitida no Arch Linux.'
[[ "${XDG_SESSION_TYPE:-}" == x11 && -n "${DISPLAY:-}" ]] || fail 'Abra uma sessão X11 do Xfce antes de aplicar.'
desktop_name="${XDG_CURRENT_DESKTOP:-}"
[[ "${desktop_name,,}" == *xfce* ]] || fail 'A sessão atual não parece ser do Xfce.'
command -v xfconf-query >/dev/null 2>&1 || fail 'xfconf-query não encontrado; instale a categoria desktop.'
command -v xfce4-panel >/dev/null 2>&1 || fail 'xfce4-panel não encontrado; instale a categoria desktop.'
command -v xset >/dev/null 2>&1 || fail 'xset não encontrado; instale a categoria desktop.'
command -v ksuperkey >/dev/null 2>&1 || fail 'ksuperkey não encontrado; reinstale a categoria desktop com o auxiliar AUR.'
command -v pgrep >/dev/null 2>&1 || fail 'pgrep não encontrado.'
command -v pacman >/dev/null 2>&1 || fail 'pacman não encontrado.'
[[ -d /usr/share/themes/Nordic/gtk-3.0 && -d /usr/share/themes/Nordic/xfwm4 ]] || fail 'Tema Nordic incompleto.'
[[ -d /usr/share/icons/Nordzy-dark && -d /usr/share/icons/Nordzy-cursors/cursors ]] || fail 'Ícones Nordzy-dark ou cursor Nordzy-cursors ausentes.'
[[ -f /usr/share/xfce4/panel/plugins/xfce4-verve-plugin.desktop ]] || fail 'Plugin Verve ausente; instale a categoria desktop.'
[[ -f /usr/share/xfce4/panel/plugins/genmon.desktop ]] || fail 'Plugin Genmon ausente.'
[[ -f /usr/share/xfce4/panel/plugins/applicationsmenu.desktop ]] || fail 'Menu de aplicativos do Xfce ausente.'
arch_logo=/usr/share/pixmaps/archlinux-logo.svg
[[ -f "$arch_logo" ]] || fail "Logo do Arch ausente: $arch_logo"
for package in inter-font ttf-fira-code; do
  pacman -Qq "$package" >/dev/null 2>&1 || fail "Fonte não instalada: $package"
done
shortcut_path='/commands/custom/<Alt>F1'
current_shortcut=$(xfconf-query -c xfce4-keyboard-shortcuts -p "$shortcut_path" 2>/dev/null || true)
[[ -z "$current_shortcut" || "$current_shortcut" == xfce4-popup-applicationsmenu || "$current_shortcut" == xfce4-popup-whiskermenu ]] || fail 'Alt+F1 já executa outro comando; revise o atalho no Xfce antes de aplicar.'
super_shortcut=$(xfconf-query -c xfce4-keyboard-shortcuts -p /commands/custom/Super_L 2>/dev/null || true)
[[ -z "$super_shortcut" ]] || fail 'Super_L já tem um atalho próprio no Xfce; remova o conflito antes de aplicar.'

panel_values=$(xfconf-query -c xfce4-panel -p /panels) || fail 'O painel do Xfce ainda não possui configuração. Abra uma sessão completa primeiro.'
mapfile -t panels < <(printf '%s\n' "$panel_values" | awk '/^[0-9]+$/')
(( ${#panels[@]} > 0 )) || fail 'Nenhum painel existente foi encontrado.'

top_panel_id=''
bottom_panels=()
for id in "${panels[@]}"; do
  position=$(xfconf-query -c xfce4-panel -p "/panels/panel-$id/position" 2>/dev/null || true)
  if [[ "$position" == p=6\;* && -z "$top_panel_id" ]]; then
    top_panel_id="$id"
  elif [[ "$position" == p=10\;* ]]; then
    bottom_panels+=("$id")
  fi
done
if [[ -z "$top_panel_id" ]]; then
  for id in "${panels[@]}"; do
    if [[ ! " ${bottom_panels[*]} " == *" $id "* ]]; then
      top_panel_id="$id"
      break
    fi
  done
fi
[[ -n "$top_panel_id" ]] || top_panel_id="${panels[0]}"

mapfile -t top_plugin_ids < <(xfconf-query -c xfce4-panel -p "/panels/panel-$top_panel_id/plugin-ids" 2>/dev/null | awk '/^[0-9]+$/')
max_plugin_id=0
while IFS= read -r property; do
  if [[ "$property" =~ ^/plugins/plugin-([0-9]+)($|/) ]]; then
    id="${BASH_REMATCH[1]}"
    (( id > max_plugin_id )) && max_plugin_id="$id"
  fi
done < <(xfconf-query -c xfce4-panel -l)

find_or_allocate_plugin() {
  local type="$1" id found_type
  for id in "${top_plugin_ids[@]}"; do
    found_type=$(xfconf-query -c xfce4-panel -p "/plugins/plugin-$id" 2>/dev/null || true)
    if [[ "$found_type" == "$type" ]]; then
      printf '%s' "$id"
      return
    fi
  done
  printf '%s' "$((max_plugin_id + 1))"
}
menu_id=''
menu_type=applicationsmenu
for id in "${top_plugin_ids[@]}"; do
  current_type=$(xfconf-query -c xfce4-panel -p "/plugins/plugin-$id" 2>/dev/null || true)
  if [[ "$current_type" == whiskermenu ]]; then
    menu_id="$id"
    menu_type=whiskermenu
    break
  fi
done
[[ -n "$menu_id" ]] || menu_id=$(find_or_allocate_plugin applicationsmenu)
if [[ "$menu_type" == whiskermenu ]]; then
  menu_command=xfce4-popup-whiskermenu
else
  menu_command=xfce4-popup-applicationsmenu
fi
command -v "$menu_command" >/dev/null 2>&1 || fail "Comando do menu ausente: $menu_command"
(( menu_id > max_plugin_id )) && max_plugin_id="$menu_id"
search_id=$(find_or_allocate_plugin verve)
(( search_id > max_plugin_id )) && max_plugin_id="$search_id"
tasklist_id=$(find_or_allocate_plugin tasklist)
(( tasklist_id > max_plugin_id )) && max_plugin_id="$tasklist_id"
pager_id=$(find_or_allocate_plugin pager)
(( pager_id > max_plugin_id )) && max_plugin_id="$pager_id"
clock_id=$(find_or_allocate_plugin clock)
(( clock_id > max_plugin_id )) && max_plugin_id="$clock_id"
keylock_id=$(xfconf-query -c xfce4-panel -p "/panels/panel-$top_panel_id/archlinuxsetup-keylocks-id" 2>/dev/null || true)
if [[ "$keylock_id" =~ ^[0-9]+$ ]]; then
  current_type=$(xfconf-query -c xfce4-panel -p "/plugins/plugin-$keylock_id" 2>/dev/null || true)
  [[ -z "$current_type" || "$current_type" == genmon ]] || keylock_id=''
fi
if [[ ! "$keylock_id" =~ ^[0-9]+$ ]]; then
  max_plugin_id=$((max_plugin_id + 1))
  keylock_id="$max_plugin_id"
fi

printf '\n==> Salvando configuração atual\n'
backup_base="${XDG_STATE_HOME:-$HOME/.local/state}/archlinuxsetup/backups"
mkdir -p -- "$backup_base"
backup_dir=$(mktemp -d -- "$backup_base/xfce-$(date +%Y%m%d-%H%M%S)-XXXXXX")
for channel in xsettings xfwm4 xfce4-panel xfce4-keyboard-shortcuts; do
  xfconf-query -c "$channel" -lv > "$backup_dir/$channel.txt" || true
  source_file="${XDG_CONFIG_HOME:-$HOME/.config}/xfce4/xfconf/xfce-perchannel-xml/$channel.xml"
  if [[ -f "$source_file" ]]; then
    cp -p -- "$source_file" "$backup_dir/$channel.xml"
  fi
done
printf 'Configuração anterior salva em %s\n' "$backup_dir"

printf '\n==> Configurando tema e painéis\n'
keylock_target="$HOME/.local/bin/archlinuxsetup-keylocks"
mkdir -p -- "$(dirname -- "$keylock_target")"
if [[ -e "$keylock_target" ]] && ! cmp -s "$PROJECT_DIR/modules/desktop/keylocks.sh" "$keylock_target"; then
  cp -p -- "$keylock_target" "$backup_dir/archlinuxsetup-keylocks"
fi
install -m 755 "$PROJECT_DIR/modules/desktop/keylocks.sh" "$keylock_target"

set_property() {
  local channel="$1" property="$2" type="$3" value="$4"
  xfconf-query -c "$channel" -p "$property" --create --type "$type" --set "$value"
}

set_property xsettings /Net/ThemeName string Nordic
set_property xsettings /Net/IconThemeName string Nordzy-dark
set_property xsettings /Gtk/CursorThemeName string Nordzy-cursors
set_property xsettings /Gtk/CursorThemeSize int 24
set_property xsettings /Gtk/FontName string 'Inter 10'
set_property xfwm4 /general/theme string Nordic
set_property xfwm4 /general/title_font string 'Inter SemiBold 10'
set_property xfce4-keyboard-shortcuts "$shortcut_path" string "$menu_command"

remaining_panels=()
for id in "${panels[@]}"; do
  if [[ "$id" != "$top_panel_id" ]]; then
    xfconf-query -c xfce4-panel -p "/panels/panel-$id" --recursive --reset
  else
    remaining_panels+=("$id")
  fi
done
if (( ${#remaining_panels[@]} != ${#panels[@]} )); then
  args=()
  for id in "${remaining_panels[@]}"; do
    args+=(--type int --set "$id")
  done
  xfconf-query -c xfce4-panel -p /panels --force-array "${args[@]}"
fi

set_property xfce4-panel "/panels/panel-$top_panel_id/position" string 'p=6;x=0;y=0'
set_property xfce4-panel "/panels/panel-$top_panel_id/length" uint 100
set_property xfce4-panel "/panels/panel-$top_panel_id/size" uint 48
set_property xfce4-panel "/panels/panel-$top_panel_id/autohide-behavior" uint 0
set_property xfce4-panel "/panels/panel-$top_panel_id/position-locked" bool true
set_property xfce4-panel "/plugins/plugin-$menu_id" string "$menu_type"
set_property xfce4-panel "/plugins/plugin-$menu_id/button-icon" string "$arch_logo"
set_property xfce4-panel "/plugins/plugin-$menu_id/show-button-title" bool false
set_property xfce4-panel "/plugins/plugin-$search_id" string verve
set_property xfce4-panel "/plugins/plugin-$tasklist_id" string tasklist
set_property xfce4-panel "/plugins/plugin-$tasklist_id/grouping" bool true
set_property xfce4-panel "/plugins/plugin-$tasklist_id/include-all-workspaces" bool false
set_property xfce4-panel "/plugins/plugin-$pager_id" string pager
set_property xfce4-panel "/plugins/plugin-$pager_id/miniature-view" bool false
set_property xfce4-panel "/plugins/plugin-$pager_id/rows" uint 1
set_property xfce4-panel "/plugins/plugin-$pager_id/numbering" bool false
workspace_count=$(xfconf-query -c xfwm4 -p /general/workspace_count 2>/dev/null || true)
if [[ "$workspace_count" =~ ^[0-9]+$ ]] && (( workspace_count >= 1 && workspace_count <= 20 )); then
  workspace_args=()
  for (( workspace=1; workspace<=workspace_count; workspace++ )); do
    workspace_args+=(--type string --set "$workspace")
  done
  xfconf-query -c xfwm4 -p /general/workspace_names --create --force-array "${workspace_args[@]}"
fi
set_property xfce4-panel "/plugins/plugin-$keylock_id" string genmon
set_property xfce4-panel "/plugins/plugin-$keylock_id/command" string "\"$keylock_target\""
set_property xfce4-panel "/plugins/plugin-$keylock_id/update-period" int 1000
set_property xfce4-panel "/plugins/plugin-$keylock_id/use-label" bool false
set_property xfce4-panel "/panels/panel-$top_panel_id/archlinuxsetup-keylocks-id" int "$keylock_id"
set_property xfce4-panel "/plugins/plugin-$clock_id" string clock
set_property xfce4-panel "/plugins/plugin-$clock_id/mode" uint 2
set_property xfce4-panel "/plugins/plugin-$clock_id/digital-format" string '%d/%m %H:%M'

ordered_ids=("$menu_id" "$search_id" "$tasklist_id")
right_ids=()
for id in "${top_plugin_ids[@]}"; do
  [[ "$id" == "$menu_id" || "$id" == "$search_id" || "$id" == "$tasklist_id" || "$id" == "$pager_id" || "$id" == "$keylock_id" || "$id" == "$clock_id" ]] && continue
  type=$(xfconf-query -c xfce4-panel -p "/plugins/plugin-$id" 2>/dev/null || true)
  case "$type" in
    applicationsmenu|whiskermenu|docklike) ;;
    clock) ;;
    systray|statusnotifier|actions|pulseaudio|notification-plugin) right_ids+=("$id") ;;
    *) ordered_ids+=("$id") ;;
  esac
done
ordered_ids+=("$pager_id" "$keylock_id")
ordered_ids+=("${right_ids[@]}" "$clock_id")
args=()
for id in "${ordered_ids[@]}"; do
  args+=(--type int --set "$id")
done
xfconf-query -c xfce4-panel -p "/panels/panel-$top_panel_id/plugin-ids" --create --force-array "${args[@]}"

xfce4-panel --restart
printf '\n==> Ativando a tecla Super no Xfce\n'
autostart_target="${XDG_CONFIG_HOME:-$HOME/.config}/autostart/ksuperkey.desktop"
autostart_temp=$(mktemp)
trap 'rm -f -- "$autostart_temp"' EXIT
cat > "$autostart_temp" <<'EOF'
[Desktop Entry]
Type=Application
Name=ArchLinuxSetup Super key
Comment=Abre o menu de aplicativos ao tocar a tecla Super
Exec=ksuperkey
OnlyShowIn=XFCE;
Terminal=false
EOF
if [[ ! -f "$autostart_target" ]] || ! cmp -s "$autostart_temp" "$autostart_target"; then
  mkdir -p -- "$(dirname -- "$autostart_target")"
  if [[ -e "$autostart_target" ]]; then
    cp -p -- "$autostart_target" "$backup_dir/ksuperkey.desktop"
  fi
  install -m 644 "$autostart_temp" "$autostart_target"
fi
if ! pgrep -u "$(id -u)" -x ksuperkey >/dev/null 2>&1; then
  ksuperkey
fi
printf '\nConcluído: painel único, busca, tarefas do espaço atual, relógio com data e indicadores configurados.\n'
