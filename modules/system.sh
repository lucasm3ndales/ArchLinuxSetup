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
  - Rolagem natural em touchpads X11 via libinput; roda do mouse preservada.
  - Não alterar Ly, zram, Limine, LUKS, partições ou Snapper.
EOF

[[ "$action" == plan ]] && { printf 'Nenhuma alteração foi feita.\n'; exit 0; }

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
  printf 'Rolagem natural do touchpad configurada para a próxima sessão X11.\n'
else
  printf 'Rolagem natural do touchpad já configurada.\n'
fi
