#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
source "$PROJECT_DIR/modules/profile.sh"

usage() {
  cat <<'EOF'
Uso: archsetup doctor [--profile NOME] [--verbose]

Confere o sistema sem alterar arquivos. --verbose também mostra o diagnóstico
detalhado de hardware, discos, sessão e serviços.
EOF
}

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

profile=''
verbose=0
while (( $# > 0 )); do
  case "$1" in
    --profile)
      (( $# >= 2 )) || { printf 'Erro: falta o nome após --profile.\n' >&2; exit 2; }
      profile="$2"
      shift 2
      ;;
    --verbose) verbose=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) printf 'Erro: opção desconhecida em doctor: %s\n' "$1" >&2; exit 2 ;;
  esac
done

if [[ -n "$profile" ]]; then
  [[ "$profile" =~ ^[a-z0-9][a-z0-9-]*$ && -d "$PROJECT_DIR/profiles/$profile" ]] || {
    printf 'Erro: perfil inexistente ou inválido: %s.\n' "$profile" >&2
    exit 2
  }
fi

check_system() {
  printf '==> Checagem do sistema\n'
  printf 'Somente leitura. Perfil: %s\n' "${profile:-nenhum}"
  failed=0
  if (( EUID == 0 )); then
    printf '[FALHA] Execute como usuário comum, sem sudo.\n'
    failed=1
  else
    printf '[OK] Usuário comum\n'
  fi

  os_id=''
  if [[ -r /etc/os-release ]]; then
    os_id=$(awk -F= '$1 == "ID" { gsub(/"/, "", $2); print $2 }' /etc/os-release)
  fi
  if [[ "$os_id" == arch ]]; then
    printf '[OK] Arch Linux\n'
  else
    printf '[FALHA] Esta máquina não informa Arch Linux em /etc/os-release.\n'
    failed=1
  fi

  if [[ -n "$profile" ]]; then
    if validate_profile_compatibility "$profile"; then
      printf '[OK] Hardware compatível com %s\n' "$profile"
    else
      printf '[FALHA] Perfil incompatível com este equipamento.\n'
      failed=1
    fi
  fi

  if command -v sudo >/dev/null 2>&1; then
    printf '[OK] sudo disponível\n'
  else
    printf '[AVISO] sudo não encontrado; instalação de pacotes exige sudo.\n'
  fi

  if command -v pacman-conf >/dev/null 2>&1; then
    if pacman-conf --repo-list 2>/dev/null | grep -qx multilib; then
      printf '[OK] [multilib] habilitado para gaming\n'
    else
      printf '[AVISO] [multilib] desabilitado; gaming não poderá ser instalado.\n'
    fi
  else
    printf '[AVISO] pacman-conf indisponível; [multilib] não verificado.\n'
  fi

  if command -v findmnt >/dev/null 2>&1; then
    root_fstype=$(findmnt -rnM / -o FSTYPE 2>/dev/null || true)
    root_opts=$(findmnt -rnM / -o OPTIONS 2>/dev/null || true)
    home_fstype=$(findmnt -rnM /home -o FSTYPE 2>/dev/null || true)
    home_opts=$(findmnt -rnM /home -o OPTIONS 2>/dev/null || true)
    boot_fstype=$(findmnt -rnM /boot -o FSTYPE 2>/dev/null || true)
    efi_fstype=$(findmnt -rnM /boot/efi -o FSTYPE 2>/dev/null || true)
    [[ -n "$efi_fstype" ]] || efi_fstype=$(findmnt -rnM /efi -o FSTYPE 2>/dev/null || true)
    if [[ "$root_fstype" == btrfs && ",$root_opts," == *,subvol=/@,* || "$root_fstype" == btrfs && ",$root_opts," == *,subvol=@,* ]]; then
      printf '[OK] Raiz Btrfs montada no subvolume @\n'
    else
      printf '[AVISO] Raiz não foi identificada como Btrfs @; confira o layout do Timeshift.\n'
    fi
    if [[ "$home_fstype" == btrfs && ",$home_opts," == *,subvol=/@home,* || "$home_fstype" == btrfs && ",$home_opts," == *,subvol=@home,* ]]; then
      printf '[OK] /home Btrfs montado no subvolume @home\n'
    else
      printf '[AVISO] /home não foi identificado como Btrfs @home.\n'
    fi
    if [[ -n "$boot_fstype" ]]; then
      printf '[AVISO] /boot é uma partição separada (%s); snapshots Btrfs de @ não incluem seu conteúdo.\n' "$boot_fstype"
    elif [[ -n "$efi_fstype" ]]; then
      printf '[OK] /boot faz parte da raiz; partição EFI montada separadamente (%s)\n' "$efi_fstype"
    else
      printf '[AVISO] Partição EFI não identificada em /boot/efi ou /efi.\n'
    fi
  else
    printf '[AVISO] findmnt indisponível; layout Btrfs não verificado.\n'
  fi

  if command -v pacman >/dev/null 2>&1; then
    if pacman -Qq timeshift >/dev/null 2>&1; then
      printf '[OK] Timeshift instalado\n'
    else
      printf '[AVISO] Timeshift não instalado; selecione-o no archinstall ou instale-o antes de criar snapshots.\n'
    fi
    if pacman -Qq grub >/dev/null 2>&1; then
      printf '[OK] GRUB instalado; confirme que é o gerenciador ativo antes de confiar no restore.\n'
    else
      printf '[AVISO] GRUB não instalado; o Timeshift documenta suporte a GRUB2 para restauração.\n'
    fi
    if pacman -Qq grub-btrfs >/dev/null 2>&1; then
      printf '[INFO] grub-btrfs instalado; teste as entradas de snapshots antes de depender delas.\n'
    fi
  fi

  if (( failed )); then
    printf 'Checagem concluída com falhas de compatibilidade. Nenhuma alteração realizada.\n'
    return 1
  fi
  printf 'Checagem concluída. Revise os avisos antes de aplicar mudanças. Nenhuma alteração realizada.\n'
}

show_details() {
  printf '==> Diagnóstico do Arch e do hardware\n'
  printf 'Somente leitura. Revise UUIDs e dados do equipamento antes de compartilhar a saída.\n'

  section 'Sistema e sessão'
  printf 'Kernel: %s\n' "$(uname -r)"
  printf 'Hostname: %s\n' "$(hostname)"
  printf 'Sessão: %s\n' "${XDG_SESSION_TYPE:-desconhecida}"
  printf 'Desktop: %s\n' "${XDG_CURRENT_DESKTOP:-desconhecido}"
  if command -v timedatectl >/dev/null 2>&1; then
    timezone=$(timedatectl show -p Timezone --value 2>/dev/null || true)
    printf 'Fuso horário: %s\n' "${timezone:-indisponível}"
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
      linux linux-headers linux-zen linux-zen-headers dkms grub grub-btrfs \
      mesa vulkan-intel intel-media-driver xf86-video-intel \
      xf86-input-libinput xf86-input-synaptics \
      nvidia nvidia-open nvidia-dkms nvidia-580xx-dkms nvidia-utils nvidia-prime \
      btrfs-progs timeshift cronie zram-generator ly \
      tlp power-profiles-daemon; do
      if pacman -Qq "$package" >/dev/null 2>&1; then
        pacman -Q "$package"
      fi
    done
  else
    printf 'Comando indisponível: pacman\n'
  fi

  section 'Boot e snapshots'
  if command -v findmnt >/dev/null 2>&1; then
    for mountpoint in /boot /boot/efi /efi; do
      if findmnt -rnM "$mountpoint" >/dev/null 2>&1; then
        printf '%s: ' "$mountpoint"
        findmnt -rnM "$mountpoint" -o SOURCE,FSTYPE,OPTIONS
      fi
    done
  else
    printf 'Comando indisponível: findmnt\n'
  fi

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

  printf '\nConcluído: diagnóstico exibido. Nenhuma alteração realizada.\n'
}

status=0
check_system || status=$?
if (( verbose )) && (( EUID != 0 )); then
  printf '\n'
  show_details || { detail_status=$?; (( status != 0 )) || status=$detail_status; }
fi
exit "$status"
