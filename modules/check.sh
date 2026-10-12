#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
source "$PROJECT_DIR/modules/profile.sh"

usage() {
  printf 'Uso: archsetup check [--profile NOME]\nConfere pré-requisitos e o layout de snapshots sem alterar o sistema.\n'
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
  if [[ "$root_fstype" == btrfs && ",$root_opts," == *,subvol=/@,* || "$root_fstype" == btrfs && ",$root_opts," == *,subvol=@,* ]]; then
    printf '[OK] Raiz Btrfs montada no subvolume @\n'
  else
    printf '[AVISO] Raiz não foi identificada como Btrfs @; confira o layout dos snapshots.\n'
  fi
  if [[ "$home_fstype" == btrfs && ",$home_opts," == *,subvol=/@home,* || "$home_fstype" == btrfs && ",$home_opts," == *,subvol=@home,* ]]; then
    printf '[OK] /home Btrfs montado no subvolume @home\n'
  else
    printf '[AVISO] /home não foi identificado como Btrfs @home.\n'
  fi
  if [[ -n "$boot_fstype" ]]; then
    printf '[AVISO] /boot é uma partição separada (%s); snapshots Btrfs de @ não incluem seu conteúdo.\n' "$boot_fstype"
  fi
else
  printf '[AVISO] findmnt indisponível; layout Btrfs não verificado.\n'
fi

if command -v pacman >/dev/null 2>&1; then
  if pacman -Qq snapper >/dev/null 2>&1; then
    printf '[OK] Snapper instalado\n'
  else
    printf '[AVISO] Snapper não instalado; confirme o resultado do archinstall.\n'
  fi
  if pacman -Qq snap-pac >/dev/null 2>&1; then
    printf '[OK] snap-pac instalado para snapshots em transações do pacman\n'
  else
    printf '[AVISO] snap-pac não instalado; está na categoria base do perfil Acer.\n'
  fi
  if pacman -Qq btrfs-assistant >/dev/null 2>&1; then
    printf '[OK] Btrfs Assistant instalado para gerenciar snapshots graficamente\n'
  else
    printf '[AVISO] Btrfs Assistant não instalado; está na categoria base do perfil Acer.\n'
  fi
  if pacman -Qq limine >/dev/null 2>&1; then
    if pacman -Qq limine-snapper-sync >/dev/null 2>&1; then
      printf '[INFO] limine-snapper-sync instalado; confirme serviço, entradas e restauração no notebook.\n'
    else
      printf '[AVISO] Limine sem integração de snapshots detectada. Restaurar @ e /boot exige procedimento validado.\n'
    fi
  fi
fi

if (( failed )); then
  printf 'Checagem concluída com falhas de compatibilidade. Nenhuma alteração realizada.\n'
  exit 1
fi
printf 'Checagem concluída. Revise os avisos antes de aplicar mudanças. Nenhuma alteração realizada.\n'
