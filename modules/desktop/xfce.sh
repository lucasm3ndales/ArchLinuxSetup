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
  GTK e bordas das janelas: Materia-dark
  Ícones: Papirus-Dark
  Cursor: capitaine-cursors, 24 px
  Fonte da interface e títulos: Inter 10
  Fontes de desenvolvimento disponíveis: Fira Code e Hack
  Painel inferior: logo do Arch abre o menu de aplicativos; Docklike Taskbar, 42 px, ocultação inteligente
  Painel superior: indicador de Caps Lock, Num Lock e Scroll Lock
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
command -v pacman >/dev/null 2>&1 || fail 'pacman não encontrado.'
[[ -d /usr/share/themes/Materia-dark/gtk-3.0 && -d /usr/share/themes/Materia-dark/xfwm4 ]] || fail 'Tema Materia-dark incompleto.'
[[ -d /usr/share/icons/Papirus-Dark && -d /usr/share/icons/capitaine-cursors/cursors ]] || fail 'Tema de ícones ou cursores ausente.'
[[ -f /usr/share/xfce4/panel/plugins/docklike.desktop ]] || fail 'Plugin Docklike ausente.'
[[ -f /usr/share/xfce4/panel/plugins/genmon.desktop ]] || fail 'Plugin Genmon ausente.'
[[ -f /usr/share/xfce4/panel/plugins/applicationsmenu.desktop ]] || fail 'Menu de aplicativos do Xfce ausente.'
arch_logo=/usr/share/pixmaps/archlinux-logo.svg
[[ -f "$arch_logo" ]] || fail "Logo do Arch ausente: $arch_logo"
for package in inter-font ttf-fira-code ttf-hack; do
  pacman -Qq "$package" >/dev/null 2>&1 || fail "Fonte não instalada: $package"
done

panel_values=$(xfconf-query -c xfce4-panel -p /panels) || fail 'O painel do Xfce ainda não possui configuração. Abra uma sessão completa primeiro.'
mapfile -t panels < <(printf '%s\n' "$panel_values" | awk '/^[0-9]+$/')
(( ${#panels[@]} > 0 )) || fail 'Nenhum painel existente foi encontrado.'

top_panel_id=''
for id in "${panels[@]}"; do
  position=$(xfconf-query -c xfce4-panel -p "/panels/panel-$id/position" 2>/dev/null || true)
  if [[ "$position" == p=6\;* ]]; then
    top_panel_id="$id"
    break
  fi
done

panel_id=''
for id in "${panels[@]}"; do
  position=$(xfconf-query -c xfce4-panel -p "/panels/panel-$id/position" 2>/dev/null || true)
  if [[ "$position" == p=10\;* ]]; then
    panel_id="$id"
    break
  fi
done
if [[ -z "$panel_id" ]]; then
  for id in "${panels[@]}"; do
    if [[ "$id" != "$top_panel_id" ]]; then
      panel_id="$id"
      break
    fi
  done
fi
new_panel=0
if [[ -z "$panel_id" ]]; then
  panel_id=1
  for id in "${panels[@]}"; do
    (( id >= panel_id )) && panel_id=$((id + 1))
  done
  new_panel=1
fi

new_top_panel=0
if [[ -z "$top_panel_id" ]]; then
  top_panel_id=$((panel_id + 1))
  for id in "${panels[@]}"; do
    (( id >= top_panel_id )) && top_panel_id=$((id + 1))
  done
  new_top_panel=1
fi

plugin_id=$(xfconf-query -c xfce4-panel -p "/panels/panel-$panel_id/archlinuxsetup-docklike-id" 2>/dev/null || true)
if [[ "$plugin_id" =~ ^[0-9]+$ ]]; then
  current_type=$(xfconf-query -c xfce4-panel -p "/plugins/plugin-$plugin_id" 2>/dev/null || true)
  [[ -z "$current_type" || "$current_type" == docklike ]] || plugin_id=''
fi
if [[ ! "$plugin_id" =~ ^[0-9]+$ ]]; then
  plugin_id=0
  while IFS= read -r property; do
    if [[ "$property" =~ ^/plugins/plugin-([0-9]+)($|/) ]]; then
      id="${BASH_REMATCH[1]}"
      (( id > plugin_id )) && plugin_id="$id"
    fi
  done < <(xfconf-query -c xfce4-panel -l)
  plugin_id=$((plugin_id + 1))
fi

keylock_id=$(xfconf-query -c xfce4-panel -p "/panels/panel-$top_panel_id/archlinuxsetup-keylocks-id" 2>/dev/null || true)
if [[ "$keylock_id" =~ ^[0-9]+$ ]]; then
  current_type=$(xfconf-query -c xfce4-panel -p "/plugins/plugin-$keylock_id" 2>/dev/null || true)
  [[ -z "$current_type" || "$current_type" == genmon ]] || keylock_id=''
  [[ "$keylock_id" != "$plugin_id" ]] || keylock_id=''
fi
if [[ ! "$keylock_id" =~ ^[0-9]+$ ]]; then
  max_plugin_id="$plugin_id"
  while IFS= read -r property; do
    if [[ "$property" =~ ^/plugins/plugin-([0-9]+)($|/) ]]; then
      id="${BASH_REMATCH[1]}"
      (( id > max_plugin_id )) && max_plugin_id="$id"
    fi
  done < <(xfconf-query -c xfce4-panel -l)
  keylock_id=$((max_plugin_id + 1))
fi

menu_id=$(xfconf-query -c xfce4-panel -p "/panels/panel-$panel_id/archlinuxsetup-menu-id" 2>/dev/null || true)
if [[ "$menu_id" =~ ^[0-9]+$ ]]; then
  current_type=$(xfconf-query -c xfce4-panel -p "/plugins/plugin-$menu_id" 2>/dev/null || true)
  [[ -z "$current_type" || "$current_type" == applicationsmenu ]] || menu_id=''
  [[ "$menu_id" != "$plugin_id" && "$menu_id" != "$keylock_id" ]] || menu_id=''
fi
if [[ ! "$menu_id" =~ ^[0-9]+$ ]]; then
  max_plugin_id="$keylock_id"
  (( plugin_id > max_plugin_id )) && max_plugin_id="$plugin_id"
  while IFS= read -r property; do
    if [[ "$property" =~ ^/plugins/plugin-([0-9]+)($|/) ]]; then
      id="${BASH_REMATCH[1]}"
      (( id > max_plugin_id )) && max_plugin_id="$id"
    fi
  done < <(xfconf-query -c xfce4-panel -l)
  menu_id=$((max_plugin_id + 1))
fi

printf '\n==> Salvando configuração atual\n'
backup_base="${XDG_STATE_HOME:-$HOME/.local/state}/archlinuxsetup/backups"
mkdir -p -- "$backup_base"
backup_dir=$(mktemp -d -- "$backup_base/xfce-$(date +%Y%m%d-%H%M%S)-XXXXXX")
for channel in xsettings xfwm4 xfce4-panel; do
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

set_property xsettings /Net/ThemeName string Materia-dark
set_property xsettings /Net/IconThemeName string Papirus-Dark
set_property xsettings /Gtk/CursorThemeName string capitaine-cursors
set_property xsettings /Gtk/CursorThemeSize int 24
set_property xsettings /Gtk/FontName string 'Inter 10'
set_property xfwm4 /general/theme string Materia-dark
set_property xfwm4 /general/title_font string 'Inter SemiBold 10'

set_property xfce4-panel "/plugins/plugin-$plugin_id" string docklike
set_property xfce4-panel "/plugins/plugin-$menu_id" string applicationsmenu
set_property xfce4-panel "/plugins/plugin-$menu_id/button-icon" string "$arch_logo"
set_property xfce4-panel "/plugins/plugin-$menu_id/show-button-title" bool false
set_property xfce4-panel "/panels/panel-$panel_id/position" string 'p=10;x=0;y=0'
set_property xfce4-panel "/panels/panel-$panel_id/length" uint 1
set_property xfce4-panel "/panels/panel-$panel_id/size" uint 42
set_property xfce4-panel "/panels/panel-$panel_id/autohide-behavior" uint 2
set_property xfce4-panel "/panels/panel-$panel_id/position-locked" bool true
set_property xfce4-panel "/panels/panel-$panel_id/archlinuxsetup-docklike-id" int "$plugin_id"
set_property xfce4-panel "/panels/panel-$panel_id/archlinuxsetup-menu-id" int "$menu_id"
xfconf-query -c xfce4-panel -p "/panels/panel-$panel_id/plugin-ids" --create --force-array --type int --set "$menu_id" --type int --set "$plugin_id"

if (( new_panel )); then
  panels+=("$panel_id")
fi
if (( new_top_panel )); then
  set_property xfce4-panel "/panels/panel-$top_panel_id/position" string 'p=6;x=0;y=0'
  set_property xfce4-panel "/panels/panel-$top_panel_id/length" uint 100
  set_property xfce4-panel "/panels/panel-$top_panel_id/size" uint 32
  set_property xfce4-panel "/panels/panel-$top_panel_id/position-locked" bool true
  panels+=("$top_panel_id")
fi
if (( new_panel || new_top_panel )); then
  args=()
  for id in "${panels[@]}"; do
    args+=(--type int --set "$id")
  done
  xfconf-query -c xfce4-panel -p /panels --force-array "${args[@]}"
fi

set_property xfce4-panel "/plugins/plugin-$keylock_id" string genmon
set_property xfce4-panel "/plugins/plugin-$keylock_id/command" string "\"$keylock_target\""
set_property xfce4-panel "/plugins/plugin-$keylock_id/update-period" int 1000
set_property xfce4-panel "/plugins/plugin-$keylock_id/use-label" bool false
set_property xfce4-panel "/panels/panel-$top_panel_id/archlinuxsetup-keylocks-id" int "$keylock_id"

mapfile -t top_plugin_ids < <(xfconf-query -c xfce4-panel -p "/panels/panel-$top_panel_id/plugin-ids" 2>/dev/null | awk '/^[0-9]+$/')
found_keylocks=0
for id in "${top_plugin_ids[@]}"; do
  [[ "$id" == "$keylock_id" ]] && found_keylocks=1
done
if (( ! found_keylocks )); then
  top_plugin_ids+=("$keylock_id")
  args=()
  for id in "${top_plugin_ids[@]}"; do
    args+=(--type int --set "$id")
  done
  xfconf-query -c xfce4-panel -p "/panels/panel-$top_panel_id/plugin-ids" --create --force-array "${args[@]}"
fi

xfce4-panel --restart
printf '\nConcluído: tema, menu com logo do Arch e indicador de teclas configurados.\n'
printf 'Fixe aplicativos na dock pelo menu de contexto do Docklike.\n'
