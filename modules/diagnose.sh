#!/usr/bin/env bash
set -euo pipefail

section() {
  printf '\n== %s ==\n' "$1"
}

optional_command() {
  if command -v "$1" >/dev/null 2>&1; then
    "$@" || printf 'Não foi possível executar: %s\n' "$1"
  else
    printf 'Comando indisponível: %s\n' "$1"
  fi
}

if (( EUID == 0 )); then
  printf 'Execute o diagnóstico como usuário comum, sem sudo.\n' >&2
  exit 1
fi

section 'Sistema e sessão'
printf 'Kernel: %s\n' "$(uname -r)"
printf 'Hostname: %s\n' "$(hostname)"
printf 'Sessão: %s\n' "${XDG_SESSION_TYPE:-desconhecida}"
printf 'Desktop: %s\n' "${XDG_CURRENT_DESKTOP:-desconhecido}"
if command -v timedatectl >/dev/null 2>&1; then
  timedatectl show -p Timezone --value || true
fi
if [[ -r /etc/locale.conf ]]; then
  printf 'Locale: '; awk -F= '$1 == "LANG" { print $2 }' /etc/locale.conf
fi
if [[ -r /etc/vconsole.conf ]]; then
  printf 'Console: '; tr '\n' ' ' < /etc/vconsole.conf; printf '\n'
fi
for field in sys_vendor product_name product_version bios_version; do
  source_file="/sys/class/dmi/id/$field"
  if [[ -r "$source_file" ]]; then
    IFS= read -r value < "$source_file" || true
    printf '%s: %s\n' "$field" "$value"
  fi
done

section 'Processador'
if command -v lscpu >/dev/null 2>&1; then
  LC_ALL=C lscpu | awk -F: '/^(Architecture|Model name):/ { sub(/^[[:space:]]+/, "", $2); print $1 ": " $2 }'
else
  printf 'Comando indisponível: lscpu\n'
fi

section 'Placas de vídeo e drivers em uso'
optional_command lspci -nnk -d ::03xx

section 'Rede, áudio e armazenamento PCI'
for class in 0280 0200 0403 0108; do
  optional_command lspci -nnk -d "::$class"
done

section 'Dispositivos USB e bloqueios de rádio'
optional_command lsusb
optional_command rfkill list

section 'Discos e sistema de arquivos (confira modelo, UUID e partições antes de qualquer alteração)'
optional_command lsblk -o NAME,MODEL,SIZE,TYPE,FSTYPE,UUID,MOUNTPOINTS
optional_command findmnt -no SOURCE,FSTYPE,OPTIONS /
optional_command findmnt -no SOURCE,FSTYPE,OPTIONS /home
optional_command swapon --show
optional_command zramctl

section 'Entrada e touchpad'
if [[ -r /proc/bus/input/devices ]]; then
  awk '/^N: Name=/ { print }' /proc/bus/input/devices
fi
if [[ "${XDG_SESSION_TYPE:-}" == x11 && -n "${DISPLAY:-}" ]]; then
  optional_command xinput list
  if command -v xfconf-query >/dev/null 2>&1; then
    xfconf-query -c pointers -lv || true
  fi
else
  printf 'Abra uma sessão X11 para consultar xinput e as opções do Xfce.\n'
fi

section 'Módulos e iluminação'
if command -v lsmod >/dev/null 2>&1; then
  lsmod | awk 'NR == 1 || $1 ~ /^(acer_wmi|i2c_hid|hid_multitouch|iwlwifi|ath10k_pci|snd_hda_intel|nvidia|nouveau|i915)$/ { print }'
fi
if [[ -d /sys/class/leds ]]; then
  find /sys/class/leds -mindepth 1 -maxdepth 1 -printf '%f\n'
fi

section 'Pacotes relevantes instalados'
if command -v pacman >/dev/null 2>&1; then
  for package in \
    mesa vulkan-intel intel-media-driver xf86-video-intel \
    xf86-input-libinput xf86-input-synaptics \
    nvidia nvidia-open nvidia-dkms nvidia-580xx-dkms nvidia-utils nvidia-prime \
    btrfs-progs snapper snap-pac btrfs-assistant zram-generator ly \
    tlp power-profiles-daemon; do
    if pacman -Qq "$package" >/dev/null 2>&1; then
      pacman -Q "$package"
    fi
  done
else
  printf 'Comando indisponível: pacman\n'
fi

section 'Configurações Snapper'
optional_command snapper list-configs

section 'Serviços de login e rede'
if command -v systemctl >/dev/null 2>&1; then
  for service in ly@tty1.service getty@tty1.service lightdm.service NetworkManager.service; do
    state=$(systemctl is-enabled "$service" 2>/dev/null) || true
    printf '%s: %s\n' "$service" "${state:-indisponível}"
  done
fi

section 'Serviços de energia'
if command -v systemctl >/dev/null 2>&1; then
  for service in tlp.service power-profiles-daemon.service; do
    state=$(systemctl is-active "$service" 2>/dev/null) || true
    printf '%s: %s\n' "$service" "${state:-indisponível}"
  done
else
  printf 'Comando indisponível: systemctl\n'
fi
