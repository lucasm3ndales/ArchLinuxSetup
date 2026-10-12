#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

usage() {
  cat <<'EOF'
Uso: ./install.sh <comando>

Comandos:
  diagnose    Mostra informações do sistema sem alterar nada
  packages    Planeja ou instala pacotes por categoria e perfil
  aur         Planeja ou instala o paru para acessar a AUR
  xfce        Planeja ou aplica o visual do Xfce na sessão do usuário
  system      Planeja ou aplica a rolagem natural do touchpad no Xorg
  storage     Inspeciona ou prepara o HD secundário LUKS/ext4
  help        Mostra esta ajuda

Exemplos:
  ./install.sh packages plan --profile acer-aspire-vx5-591g-54pg
  ./install.sh aur plan
  ./install.sh aur apply
  ./install.sh packages apply --profile acer-aspire-vx5-591g-54pg
  ./install.sh packages plan gaming emulation
  ./install.sh xfce plan
  ./install.sh xfce apply
  ./install.sh system plan
  ./install.sh system apply
  ./install.sh storage plan
EOF
}

case "${1:-help}" in
  diagnose)
    if (( $# != 1 )); then
      usage >&2
      exit 2
    fi
    exec bash "$PROJECT_DIR/modules/diagnose.sh"
    ;;
  packages)
    shift
    exec bash "$PROJECT_DIR/modules/packages.sh" "$@"
    ;;
  aur)
    shift
    exec bash "$PROJECT_DIR/modules/aur.sh" "$@"
    ;;
  xfce)
    shift
    exec bash "$PROJECT_DIR/modules/xfce.sh" "$@"
    ;;
  system)
    shift
    exec bash "$PROJECT_DIR/modules/system.sh" "$@"
    ;;
  storage)
    shift
    exec bash "$PROJECT_DIR/modules/storage.sh" "$@"
    ;;
  help|-h|--help)
    usage
    ;;
  *)
    printf 'Comando desconhecido: %s\n' "$1" >&2
    usage >&2
    exit 2
    ;;
esac
