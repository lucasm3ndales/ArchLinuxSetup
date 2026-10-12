#!/usr/bin/env bash
set -euo pipefail

usage() {
  printf 'Uso: ./install.sh system <plan|apply>\n'
}

fail() {
  printf 'Erro: %s\n' "$1" >&2
  exit 1
}

action="${1:-}"
[[ "$action" == plan || "$action" == apply ]] || { usage >&2; exit 2; }
(( $# == 1 )) || { usage >&2; exit 2; }

cat <<'EOF'
Plano do sistema:
  - Ly no tty1, com sessões X11 instaladas; esconder entradas shell, xinitrc e Wayland.
  - Manter tty2 disponível para recuperação.
  - zram0 com metade da RAM e compressão zstd, ativada no próximo boot.
  - Rolagem natural em touchpads X11 via libinput; roda do mouse preservada.
  - Não alterar Limine, LUKS, partições ou Snapper.
EOF

[[ "$action" == plan ]] && { printf 'Nenhuma alteração foi feita.\n'; exit 0; }

(( EUID != 0 )) || fail 'Execute como usuário comum, sem sudo.'
[[ -r /etc/os-release ]] || fail 'Não foi possível identificar a distribuição.'
os_id=$(awk -F= '$1 == "ID" { gsub(/"/, "", $2); print $2 }' /etc/os-release)
[[ "$os_id" == arch ]] || fail 'Este comando só pode ser aplicado no Arch Linux.'
for command_name in sudo pacman systemctl awk install cmp mktemp; do
  command -v "$command_name" >/dev/null 2>&1 || fail "Comando ausente: $command_name"
done
for package in ly xorg-xauth xfce4-session zram-generator xf86-input-libinput; do
  pacman -Qq "$package" >/dev/null 2>&1 || fail "Instale antes o pacote $package."
done
[[ -f /etc/ly/config.ini ]] || fail 'Configuração /etc/ly/config.ini ausente.'
[[ -f /usr/share/xsessions/xfce.desktop ]] || fail 'Sessão Xfce X11 ausente em /usr/share/xsessions/xfce.desktop.'
[[ -f /usr/lib/systemd/system/ly@.service ]] || fail 'Serviço ly@.service ausente.'

for manager in lightdm.service sddm.service gdm.service lxdm.service; do
  if systemctl is-enabled --quiet "$manager" 2>/dev/null; then
    fail "Desabilite $manager após confirmar o Ly e execute novamente."
  fi
done

temporary_ly=$(mktemp)
trap 'rm -f -- "${temporary_ly:-}"' EXIT
awk '
  /^[[:space:]]*#?[[:space:]]*(shell|xinitrc|waylandsessions|xsessions)[[:space:]]*=/ { next }
  { print }
  END {
    print "shell = false"
    print "xinitrc = null"
    print "waylandsessions = null"
    print "xsessions = /usr/share/xsessions"
  }
' /etc/ly/config.ini > "$temporary_ly"

if ! cmp -s "$temporary_ly" /etc/ly/config.ini; then
  backup="/etc/ly/config.ini.archlinuxsetup.bak-$(date +%Y%m%d%H%M%S)"
  sudo cp -p -- /etc/ly/config.ini "$backup"
  sudo install -m 644 "$temporary_ly" /etc/ly/config.ini
  printf 'Ly configurado; backup: %s\n' "$backup"
else
  printf 'Ly já configurado.\n'
fi

zram_configuration=$'[zram0]\nzram-size = ram / 2\ncompression-algorithm = zstd\n'
zram_path=/etc/systemd/zram-generator.conf
if [[ ! -f "$zram_path" ]] || [[ "$(cat "$zram_path")" != "${zram_configuration%$'\n'}" ]]; then
  if [[ -e "$zram_path" ]]; then
    backup="$zram_path.archlinuxsetup.bak-$(date +%Y%m%d%H%M%S)"
    sudo cp -p -- "$zram_path" "$backup"
    printf 'Backup da configuração zram: %s\n' "$backup"
  fi
  printf '%s' "$zram_configuration" | sudo install -m 644 /dev/stdin "$zram_path"
  printf 'zram configurada para o próximo boot.\n'
else
  printf 'zram já configurada.\n'
fi

sudo systemctl enable ly@tty1.service
sudo systemctl disable getty@tty1.service

touchpad_path=/etc/X11/xorg.conf.d/90-archlinuxsetup-touchpad.conf
temporary_touchpad=$(mktemp)
trap 'rm -f -- "${temporary_ly:-}" "${temporary_touchpad:-}"' EXIT
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
  printf 'Rolagem natural do touchpad configurada para a próxima sessão X11.\n'
else
  printf 'Rolagem natural do touchpad já configurada.\n'
fi
printf 'Ly habilitado para o próximo boot. tty2 continua disponível. Reinicie após revisar a configuração.\n'
