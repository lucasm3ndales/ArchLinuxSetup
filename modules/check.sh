#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
source "$PROJECT_DIR/modules/profile.sh"

usage() {
  printf 'Uso: archsetup check [--profile NOME]\nConfere pré-requisitos e o layout de instalação sem alterar o sistema.\n'
}

profile=''
while (( $# > 0 )); do
  case "$1" in
    --profile)
      (( $# >= 2 )) || { printf 'Erro: falta o nome após --profile.\n' >&2; exit 2; }
      profile="$2"
      shift 2
      ;;
    -h|--help) usage; exit 0 ;;
    *) printf 'Erro: opção desconhecida em check: %s\n' "$1" >&2; exit 2 ;;
  esac
done

if [[ -n "$profile" ]]; then
  [[ "$profile" =~ ^[a-z0-9][a-z0-9-]*$ && -d "$PROJECT_DIR/profiles/$profile" ]] || {
    printf 'Erro: perfil inexistente ou inválido: %s.\n' "$profile" >&2
    exit 2
  }
fi

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
  exit 1
fi
printf 'Checagem concluída. Revise os avisos antes de aplicar mudanças. Nenhuma alteração realizada.\n'
