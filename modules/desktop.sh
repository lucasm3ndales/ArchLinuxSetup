#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
source "$PROJECT_DIR/modules/profile.sh"

usage() {
  cat <<'EOF'
Uso: archsetup desktop [--profile NOME] [--dry-run]

Configura o Xfce, o LightDM/Slick Greeter e o touchpad X11.
Execute como usuário comum em uma sessão X11 do Xfce.
O perfil opcional aplica somente ajustes de hardware compatíveis.
EOF
}

fail() {
  printf 'Erro: %s\n' "$1" >&2
  exit 1
}

profile=''
dry_run=0
while (( $# > 0 )); do
  case "$1" in
    --dry-run) dry_run=1; shift ;;
    --profile)
      (( $# >= 2 )) || fail 'Falta o nome após --profile.'
      profile="$2"
      shift 2
      ;;
    -h|--help) usage; exit 0 ;;
    *) usage >&2; exit 2 ;;
  esac
done
if [[ -n "$profile" ]]; then
  [[ "$profile" =~ ^[a-z0-9][a-z0-9-]*$ && -d "$PROJECT_DIR/profiles/$profile" ]] || fail "Perfil inexistente ou inválido: $profile"
fi
required_desktop_packages=(xf86-input-libinput inter-font ttf-fira-code ksuperkey nordic-theme nordzy-icon-theme nordzy-cursors xfce4-kbdleds-plugin)

printf '==> %s do desktop\n' "$([[ "$dry_run" == 1 ]] && printf 'Prévia' || printf 'Configuração')"
printf 'Perfil: %s\n' "${profile:-nenhum}"
cat <<'EOF'
  Xfce: tema Nordic, fontes, painel superior, busca, teclas Lock, espaços de trabalho e tecla Super.
  Login: tema Nordic no LightDM com Slick Greeter.
  Touchpad X11: rolagem natural; o perfil Acer também ativa toque para clicar.
EOF
if (( dry_run )); then
  if command -v pacman >/dev/null 2>&1; then
    missing_desktop_packages=()
    for package in "${required_desktop_packages[@]}"; do
      pacman -Qq "$package" >/dev/null 2>&1 || missing_desktop_packages+=("$package")
    done
    if (( ${#missing_desktop_packages[@]} > 0 )); then
      printf 'Pacotes ausentes: %s\nDependência: archsetup packages desktop.\n' "${missing_desktop_packages[*]}"
    else
      printf 'Pacotes necessários da categoria desktop já instalados.\n'
    fi
  else
    printf 'Pacotes necessários não verificados (pacman indisponível).\n'
  fi
  printf 'Prévia concluída. Nenhuma alteração realizada.\n'
  exit 0
fi

(( EUID != 0 )) || fail 'Execute como usuário comum, sem sudo.'
[[ -r /etc/os-release ]] || fail 'Não foi possível identificar a distribuição.'
os_id=$(awk -F= '$1 == "ID" { gsub(/"/, "", $2); print $2 }' /etc/os-release)
[[ "$os_id" == arch ]] || fail 'A configuração do desktop só é permitida no Arch Linux.'
[[ "${XDG_SESSION_TYPE:-}" == x11 && -n "${DISPLAY:-}" ]] || fail 'Abra uma sessão X11 do Xfce antes de aplicar.'
desktop_name="${XDG_CURRENT_DESKTOP:-}"
[[ "${desktop_name,,}" == *xfce* ]] || fail 'A sessão atual não parece ser do Xfce.'
command -v pacman >/dev/null 2>&1 || fail 'pacman não encontrado.'
command -v sudo >/dev/null 2>&1 || fail 'sudo não encontrado.'
if [[ -n "$profile" ]]; then
  validate_profile_compatibility "$profile" || fail 'Aplicação recusada para este perfil.'
fi

pacman -Qq xorg-server >/dev/null 2>&1 || fail 'xorg-server ausente; configure a sessão X11 do Xfce antes de aplicar.'
for command_name in xfconf-query xfce4-panel pgrep lightdm install cmp mktemp; do
  command -v "$command_name" >/dev/null 2>&1 || fail "Comando necessário ausente: $command_name"
done
[[ -f /usr/share/xgreeters/lightdm-slick-greeter.desktop ]] || fail 'Slick Greeter ausente; instale e ative o LightDM/Slick antes de aplicar.'
active_dm=$(readlink -f -- /etc/systemd/system/display-manager.service 2>/dev/null || true)
[[ "$active_dm" == */lightdm.service ]] || fail 'LightDM não é o gerenciador de login habilitado.'
[[ -f /usr/share/xfce4/panel/plugins/xfce4-verve-plugin.desktop ]] || fail 'Plugin Verve ausente; confira a instalação do Xfce.'
[[ -f /usr/share/xfce4/panel/plugins/applicationsmenu.desktop ]] || fail 'Menu de aplicativos do Xfce ausente.'
[[ -f /usr/share/pixmaps/archlinux-logo.svg ]] || fail 'Logo do Arch ausente.'
shortcut_path='/commands/custom/<Alt>F1'
current_shortcut=$(xfconf-query -c xfce4-keyboard-shortcuts -p "$shortcut_path" 2>/dev/null || true)
[[ -z "$current_shortcut" || "$current_shortcut" == xfce4-popup-applicationsmenu || "$current_shortcut" == xfce4-popup-whiskermenu ]] || fail 'Alt+F1 já executa outro comando; revise o atalho no Xfce antes de aplicar.'
super_shortcut=$(xfconf-query -c xfce4-keyboard-shortcuts -p /commands/custom/Super_L 2>/dev/null || true)
[[ -z "$super_shortcut" ]] || fail 'Super_L já tem um atalho próprio no Xfce; remova o conflito antes de aplicar.'
panel_values=$(xfconf-query -c xfce4-panel -p /panels) || fail 'O painel do Xfce ainda não possui configuração. Abra uma sessão completa primeiro.'
mapfile -t panels < <(printf '%s\n' "$panel_values" | awk '/^[0-9]+$/')
(( ${#panels[@]} > 0 )) || fail 'Nenhum painel existente foi encontrado.'
for path in /etc/lightdm/slick-greeter.conf /etc/lightdm/lightdm.conf.d/90-archsetup-slick.conf; do
  [[ ! -L "$path" ]] || fail "Não vou substituir um link simbólico: $path"
  [[ ! -e "$path" || -f "$path" ]] || fail "Caminho não é um arquivo comum: $path"
done
[[ ! -f /etc/lightdm/slick-greeter.conf || -r /etc/lightdm/slick-greeter.conf ]] || fail 'Não é possível ler a configuração existente do Slick Greeter.'

missing_desktop_packages=()
for package in "${required_desktop_packages[@]}"; do
  pacman -Qq "$package" >/dev/null 2>&1 || missing_desktop_packages+=("$package")
done
if (( ${#missing_desktop_packages[@]} > 0 )); then
  printf 'Pacotes necessários ausentes: %s\n' "${missing_desktop_packages[*]}"
  printf 'Dependência: archsetup packages desktop.\n'
  [[ -t 0 ]] || fail 'Execute archsetup packages desktop em um terminal interativo e tente novamente.'
  printf 'Executar archsetup packages desktop agora? [s/N] '
  IFS= read -r answer || fail 'Não foi possível ler a resposta.'
  case "$answer" in
    s|S|sim|SIM|Sim) bash "$PROJECT_DIR/archsetup" packages desktop ;;
    *) fail 'Dependência não executada. Configure os pacotes e tente novamente.' ;;
  esac
fi

for package in xorg-server xf86-input-libinput inter-font ttf-fira-code; do
  pacman -Qq "$package" >/dev/null 2>&1 || fail "Pacote necessário ausente: $package"
done
for command_name in xfconf-query xfce4-panel ksuperkey pgrep lightdm install cmp mktemp; do
  command -v "$command_name" >/dev/null 2>&1 || fail "Comando necessário ausente: $command_name"
done
[[ -f /usr/share/xfce4/panel/plugins/kbdleds.desktop ]] || fail 'Plugin Keyboard LEDs ausente após instalar a categoria desktop.'
[[ -d /usr/share/themes/Nordic/gtk-3.0 && -d /usr/share/themes/Nordic/xfwm4 ]] || fail 'Tema Nordic incompleto.'
[[ -d /usr/share/icons/Nordzy-dark && -d /usr/share/icons/Nordzy-cursors/cursors ]] || fail 'Ícones Nordzy-dark ou cursor Nordzy-cursors ausentes.'
sudo -v || fail 'Não foi possível obter acesso sudo.'

configure_xfce() (
(( EUID != 0 )) || fail 'Execute como usuário comum, sem sudo.'
[[ -r /etc/os-release ]] || fail 'Não foi possível identificar a distribuição.'
os_id=$(awk -F= '$1 == "ID" { gsub(/"/, "", $2); print $2 }' /etc/os-release)
[[ "$os_id" == arch ]] || fail 'A aplicação do visual só é permitida no Arch Linux.'
[[ "${XDG_SESSION_TYPE:-}" == x11 && -n "${DISPLAY:-}" ]] || fail 'Abra uma sessão X11 do Xfce antes de aplicar.'
desktop_name="${XDG_CURRENT_DESKTOP:-}"
[[ "${desktop_name,,}" == *xfce* ]] || fail 'A sessão atual não parece ser do Xfce.'
command -v xfconf-query >/dev/null 2>&1 || fail 'xfconf-query não encontrado; instale a categoria desktop.'
command -v xfce4-panel >/dev/null 2>&1 || fail 'xfce4-panel não encontrado; instale a categoria desktop.'
command -v ksuperkey >/dev/null 2>&1 || fail 'ksuperkey não encontrado; reinstale a categoria desktop com o auxiliar AUR.'
command -v pgrep >/dev/null 2>&1 || fail 'pgrep não encontrado.'
command -v pacman >/dev/null 2>&1 || fail 'pacman não encontrado.'
[[ -d /usr/share/themes/Nordic/gtk-3.0 && -d /usr/share/themes/Nordic/xfwm4 ]] || fail 'Tema Nordic incompleto.'
[[ -d /usr/share/icons/Nordzy-dark && -d /usr/share/icons/Nordzy-cursors/cursors ]] || fail 'Ícones Nordzy-dark ou cursor Nordzy-cursors ausentes.'
[[ -f /usr/share/xfce4/panel/plugins/xfce4-verve-plugin.desktop ]] || fail 'Plugin Verve ausente; instale a categoria desktop.'
[[ -f /usr/share/xfce4/panel/plugins/kbdleds.desktop ]] || fail 'Plugin Keyboard LEDs ausente; instale a categoria desktop.'
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
legacy_keylock_id=$(xfconf-query -c xfce4-panel -p "/panels/panel-$top_panel_id/archlinuxsetup-keylocks-id" 2>/dev/null || true)
legacy_genmon_id=''
if [[ "$legacy_keylock_id" =~ ^[0-9]+$ && " ${top_plugin_ids[*]} " == *" $legacy_keylock_id "* ]]; then
  current_type=$(xfconf-query -c xfce4-panel -p "/plugins/plugin-$legacy_keylock_id" 2>/dev/null || true)
  [[ "$current_type" != genmon ]] || legacy_genmon_id="$legacy_keylock_id"
fi
keylock_id=$(xfconf-query -c xfce4-panel -p "/panels/panel-$top_panel_id/archlinuxsetup-kbdleds-id" 2>/dev/null || true)
if [[ "$keylock_id" =~ ^[0-9]+$ && " ${top_plugin_ids[*]} " == *" $keylock_id "* ]]; then
  current_type=$(xfconf-query -c xfce4-panel -p "/plugins/plugin-$keylock_id" 2>/dev/null || true)
  [[ "$current_type" == kbdleds ]] || keylock_id=''
else
  keylock_id=''
fi
if [[ -z "$keylock_id" ]]; then
  for id in "${top_plugin_ids[@]}"; do
    current_type=$(xfconf-query -c xfce4-panel -p "/plugins/plugin-$id" 2>/dev/null || true)
    if [[ "$current_type" == kbdleds ]]; then
      keylock_id="$id"
      break
    fi
  done
fi
[[ -n "$keylock_id" ]] || keylock_id="$legacy_genmon_id"
if [[ -z "$keylock_id" ]]; then
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
if [[ -e "$keylock_target" || -L "$keylock_target" ]]; then
  mv -- "$keylock_target" "$backup_dir/archlinuxsetup-keylocks"
  printf 'Indicador antigo preservado em %s\n' "$backup_dir/archlinuxsetup-keylocks"
fi

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
if [[ -n "$legacy_genmon_id" ]]; then
  xfconf-query -c xfce4-panel -p "/plugins/plugin-$legacy_genmon_id" --recursive --reset
fi
if [[ -n "$legacy_keylock_id" ]]; then
  xfconf-query -c xfce4-panel -p "/panels/panel-$top_panel_id/archlinuxsetup-keylocks-id" --reset
fi
set_property xfce4-panel "/plugins/plugin-$keylock_id" string kbdleds
set_property xfce4-panel "/panels/panel-$top_panel_id/archlinuxsetup-kbdleds-id" int "$keylock_id"
set_property xfce4-panel "/plugins/plugin-$clock_id" string clock
set_property xfce4-panel "/plugins/plugin-$clock_id/mode" uint 2
set_property xfce4-panel "/plugins/plugin-$clock_id/digital-format" string '%d/%m %H:%M'

ordered_ids=("$menu_id" "$search_id" "$tasklist_id")
right_ids=()
for id in "${top_plugin_ids[@]}"; do
  [[ "$id" == "$menu_id" || "$id" == "$search_id" || "$id" == "$tasklist_id" || "$id" == "$pager_id" || "$id" == "$keylock_id" || "$id" == "$legacy_genmon_id" || "$id" == "$clock_id" ]] && continue
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
)

configure_greeter() (
config=/etc/lightdm/slick-greeter.conf
seat_config=/etc/lightdm/lightdm.conf.d/90-archsetup-slick.conf
(( EUID != 0 )) || fail 'Execute como usuário comum, sem sudo.'
[[ -r /etc/os-release ]] || fail 'Não foi possível identificar a distribuição.'
os_id=$(awk -F= '$1 == "ID" { gsub(/"/, "", $2); print $2 }' /etc/os-release)
[[ "$os_id" == arch ]] || fail 'A configuração da tela de login só é permitida no Arch Linux.'
command -v sudo >/dev/null 2>&1 || fail 'sudo não encontrado.'
command -v lightdm >/dev/null 2>&1 || fail 'LightDM não encontrado.'
[[ -f /usr/share/xgreeters/lightdm-slick-greeter.desktop ]] || fail 'Instale lightdm-slick-greeter antes de aplicar.'
active_dm=$(readlink -f -- /etc/systemd/system/display-manager.service 2>/dev/null || true)
[[ "$active_dm" == */lightdm.service ]] || fail 'LightDM não é o gerenciador de login habilitado. Confira display-manager.service.'
[[ -d /usr/share/themes/Nordic/gtk-3.0 ]] || fail 'Tema Nordic ausente; instale a categoria desktop.'
[[ -d /usr/share/icons/Nordzy-dark ]] || fail 'Ícones Nordzy-dark ausentes; instale a categoria desktop.'
[[ -d /usr/share/icons/Nordzy-cursors/cursors ]] || fail 'Cursores Nordzy-cursors ausentes; instale a categoria desktop.'
[[ -f /usr/share/pixmaps/archlinux-logo.svg ]] || fail 'Logo do Arch ausente.'
command -v pacman >/dev/null 2>&1 || fail 'pacman não encontrado.'
pacman -Qq inter-font >/dev/null 2>&1 || fail 'Fonte Inter ausente; instale a categoria desktop.'
for path in "$config" "$seat_config"; do
  [[ ! -L "$path" ]] || fail "Não vou substituir um link simbólico: $path"
  [[ ! -e "$path" || -f "$path" ]] || fail "Caminho não é um arquivo comum: $path"
done
[[ ! -f "$config" || -r "$config" ]] || fail "Não é possível ler a configuração existente: $config"
sudo -v || fail 'Não foi possível obter acesso sudo.'

tmp_config=$(mktemp)
tmp_seat=$(mktemp)
trap 'rm -f -- "$tmp_config" "$tmp_seat"' EXIT

awk '
function emit_settings() {
  print "theme-name=Nordic"
  print "icon-theme-name=Nordzy-dark"
  print "cursor-theme-name=Nordzy-cursors"
  print "cursor-theme-size=24"
  print "font-name=Inter 10"
  print "background=#2e3440"
  print "background-color=#2e3440"
  print "draw-user-backgrounds=false"
  print "logo=/usr/share/pixmaps/archlinux-logo.svg"
}
/^\[[^]]+\][[:space:]]*$/ {
  if (in_greeter) emit_settings()
  in_greeter = ($0 ~ /^\[Greeter\][[:space:]]*$/)
  if (in_greeter) found = 1
  print
  next
}
in_greeter && /^[[:space:]]*(theme-name|icon-theme-name|cursor-theme-name|cursor-theme-size|font-name|background|background-color|draw-user-backgrounds|logo)[[:space:]]*=/ { next }
{ print }
END {
  if (in_greeter) emit_settings()
  if (!found) {
    if (NR) print ""
    print "[Greeter]"
    emit_settings()
  }
}
' "$([[ -f "$config" ]] && printf '%s' "$config" || printf '/dev/null')" > "$tmp_config"

cat > "$tmp_seat" <<'EOF'
[Seat:*]
greeter-session=lightdm-slick-greeter
EOF

update_file() {
  local target="$1" source="$2" backup
  if [[ -f "$target" ]] && sudo cmp -s -- "$source" "$target"; then
    printf 'Já configurado: %s\n' "$target"
    return 0
  fi
  if [[ -f "$target" ]]; then
    backup="$target.archsetup.bak-$(date +%Y%m%d%H%M%S%N)"
    sudo cp -p -- "$target" "$backup"
    printf 'Backup: %s\n' "$backup"
  fi
  sudo install -m 644 -- "$source" "$target"
  printf 'Atualizado: %s\n' "$target"
}

sudo install -d -m 755 -- /etc/lightdm/lightdm.conf.d
update_file "$config" "$tmp_config"
update_file "$seat_config" "$tmp_seat"
printf 'Concluído. A nova aparência aparecerá no próximo login; a sessão atual continua aberta.\n'
)

configure_touchpad() (
(( EUID != 0 )) || fail 'Execute como usuário comum, sem sudo.'
[[ -r /etc/os-release ]] || fail 'Não foi possível identificar a distribuição.'
os_id=$(awk -F= '$1 == "ID" { gsub(/"/, "", $2); print $2 }' /etc/os-release)
[[ "$os_id" == arch ]] || fail 'Este comando só pode ser aplicado no Arch Linux.'
for command_name in sudo pacman install cmp mktemp; do
  command -v "$command_name" >/dev/null 2>&1 || fail "Comando ausente: $command_name"
done
for package in xorg-server xf86-input-libinput; do
  pacman -Qq "$package" >/dev/null 2>&1 || fail "Instale antes o pacote $package."
done
if [[ -n "$profile" ]]; then
  validate_profile_compatibility "$profile" || fail 'Aplicação recusada para este perfil.'
fi

touchpad_path=/etc/X11/xorg.conf.d/90-archlinuxsetup-touchpad.conf
temporary_touchpad=$(mktemp)
trap 'rm -f -- "$temporary_touchpad"' EXIT
cat > "$temporary_touchpad" <<'EOF'
Section "InputClass"
    Identifier "ArchLinuxSetup touchpad natural scroll"
    MatchIsTouchpad "on"
    MatchDevicePath "/dev/input/event*"
    Driver "libinput"
    Option "NaturalScrolling" "true"
EndSection
EOF
if [[ ! -f "$touchpad_path" ]] || ! cmp -s "$temporary_touchpad" "$touchpad_path"; then
  if [[ -e "$touchpad_path" ]]; then
    backup="$touchpad_path.archlinuxsetup.bak-$(date +%Y%m%d%H%M%S)"
    sudo cp -p -- "$touchpad_path" "$backup"
    printf 'Backup da configuração do touchpad: %s\n' "$backup"
  fi
  sudo install -D -m 644 "$temporary_touchpad" "$touchpad_path"
  printf 'Rolagem natural configurada para a próxima sessão X11.\n'
else
  printf 'Rolagem natural já configurada; nenhuma alteração necessária.\n'
fi

if [[ -n "$profile" ]]; then
  tapping_path=/etc/X11/xorg.conf.d/91-archlinuxsetup-acer-tapping.conf
  temporary_tapping=$(mktemp)
  trap 'rm -f -- "$temporary_touchpad" "$temporary_tapping"' EXIT
  cat > "$temporary_tapping" <<'EOF'
Section "InputClass"
    Identifier "ArchLinuxSetup Acer touchpad tap-to-click"
    MatchIsTouchpad "on"
    MatchDevicePath "/dev/input/event*"
    Driver "libinput"
    Option "Tapping" "true"
EndSection
EOF
  if [[ ! -f "$tapping_path" ]] || ! cmp -s "$temporary_tapping" "$tapping_path"; then
    if [[ -e "$tapping_path" ]]; then
      backup="$tapping_path.archlinuxsetup.bak-$(date +%Y%m%d%H%M%S)"
      sudo cp -p -- "$tapping_path" "$backup"
      printf 'Backup da configuração de toque: %s\n' "$backup"
    fi
    sudo install -D -m 644 "$temporary_tapping" "$tapping_path"
    printf 'Toque para clicar configurado para a próxima sessão X11.\n'
  else
    printf 'Toque para clicar já configurado; nenhuma alteração necessária.\n'
  fi
fi

printf 'Concluído: configurações do touchpad processadas.\n'
)

printf '\n==> Configurando Xfce\n'
configure_xfce
printf '\n==> Configurando tela de login\n'
configure_greeter
printf '\n==> Configurando touchpad\n'
configure_touchpad
printf '\nConcluído: desktop configurado.\n'
