#!/usr/bin/env bash
set -euo pipefail

usage() {
  printf 'Uso: archsetup touchpad [--dry-run]\nAtiva rolagem natural apenas para touchpads X11.\n'
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
  printf '==> Prévia da rolagem natural do touchpad\n'
else
  printf '==> Configuração da rolagem natural do touchpad\n'
fi
cat <<'EOF'
Regra Xorg: /etc/X11/xorg.conf.d/90-archlinuxsetup-touchpad.conf
Efeito: touchpads X11 via libinput; disponível na próxima sessão.
EOF

if (( dry_run )); then
  printf 'Prévia concluída. Nenhuma alteração realizada.\n'
  exit 0
fi

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
  printf 'Concluído: rolagem natural configurada para a próxima sessão X11.\n'
else
  printf 'Concluído: regra já está configurada; nenhuma alteração necessária.\n'
fi
