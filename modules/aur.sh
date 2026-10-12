#!/usr/bin/env bash
set -euo pipefail

usage() { printf 'Uso: ./install.sh aur <plan|apply>\n'; }
fail() { printf 'Erro: %s\n' "$1" >&2; exit 1; }

action="${1:-}"
[[ "$action" == plan || "$action" == apply ]] || { usage >&2; exit 2; }
(( $# == 1 )) || { usage >&2; exit 2; }

if command -v paru >/dev/null 2>&1; then
  printf 'paru já está disponível: %s\n' "$(command -v paru)"
  exit 0
fi

cat <<'EOF'
Plano AUR:
  1. Atualizar o Arch e instalar git e base-devel pelos repositórios oficiais.
  2. Clonar https://aur.archlinux.org/paru.git em diretório temporário.
  3. Mostrar o PKGBUILD e listar os demais arquivos para revisão.
  4. Após confirmação digitada, compilar e instalar com makepkg -si como usuário comum.
EOF
[[ "$action" == plan ]] && { printf 'Nenhuma alteração foi feita.\n'; exit 0; }

(( EUID != 0 )) || fail 'Execute como usuário comum, sem sudo.'
[[ -t 0 ]] || fail 'Um terminal interativo é necessário para revisar o PKGBUILD.'
[[ -r /etc/os-release ]] || fail 'Não foi possível identificar a distribuição.'
os_id=$(awk -F= '$1 == "ID" { gsub(/"/, "", $2); print $2 }' /etc/os-release)
[[ "$os_id" == arch ]] || fail 'Este comando só pode ser aplicado no Arch Linux.'
for command_name in sudo pacman mktemp; do
  command -v "$command_name" >/dev/null 2>&1 || fail "Comando ausente: $command_name"
done

sudo pacman -Syu --needed git base-devel
command -v git >/dev/null 2>&1 || fail 'git não encontrado após instalar git.'
command -v makepkg >/dev/null 2>&1 || fail 'makepkg não encontrado após instalar base-devel.'

build_dir=$(mktemp -d -t archlinuxsetup-paru.XXXXXXXX)
trap 'rm -rf -- "$build_dir"' EXIT
git clone -- https://aur.archlinux.org/paru.git "$build_dir/paru"
[[ -f "$build_dir/paru/PKGBUILD" ]] || fail 'O repositório clonado não contém PKGBUILD.'

printf '\nArquivos recebidos da AUR:\n'
git -C "$build_dir/paru" ls-files
printf '\nPKGBUILD para revisão:\n'
cat -- "$build_dir/paru/PKGBUILD"
printf '\nConfira também os demais arquivos listados em %s\n' "$build_dir/paru"
printf 'Digite INSTALAR PARU após revisar para continuar: '
IFS= read -r confirmation || fail 'Entrada encerrada antes da confirmação.'
[[ "$confirmation" == 'INSTALAR PARU' ]] || fail 'Instalação do paru cancelada após preparar os pacotes oficiais.'

(cd "$build_dir/paru" && makepkg -si)
command -v paru >/dev/null 2>&1 || fail 'makepkg terminou, mas paru não foi encontrado no PATH.'
printf 'paru instalado. Agora execute ./install.sh packages plan e depois packages apply.\n'
