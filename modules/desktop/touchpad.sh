#!/usr/bin/env bash
set -euo pipefail
PROJECT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$PROJECT_DIR/modules/profile.sh"

usage() {
  printf 'Uso: archsetup touchpad [--profile NOME] [--dry-run]\nAtiva rolagem natural em touchpads X11. O perfil Acer também ativa toque para clicar.\n'
}

fail() {
  printf 'Erro: %s\n' "$1" >&2
  exit 1
}

dry_run=0
profile=''
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

if (( dry_run )); then
  printf '==> Prévia da rolagem natural do touchpad\n'
else
  printf '==> Configuração da rolagem natural do touchpad\n'
fi
cat <<'EOF'
Regra Xorg: /etc/X11/xorg.conf.d/90-archlinuxsetup-touchpad.conf
Efeito: touchpads X11 via libinput; disponível na próxima sessão.
EOF
if [[ -n "$profile" ]]; then
  printf 'Perfil: %s\nRegra adicional: toque para clicar no touchpad Acer.\n' "$profile"
fi

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
