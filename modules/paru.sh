#!/usr/bin/env bash
set -euo pipefail

usage() {
  printf 'Uso: archsetup paru [--dry-run]\nInstala paru após mostrar o PKGBUILD para revisão.\n'
}
fail() { printf 'Erro: %s\n' "$1" >&2; exit 1; }

dry_run=0
case "${1:-}" in
  '') ;;
  --dry-run) dry_run=1; shift ;;
  -h|--help) usage; exit 0 ;;
  *) usage >&2; exit 2 ;;
esac
(( $# == 0 )) || { usage >&2; exit 2; }

printf '==> %s do paru\n' "$([[ "$dry_run" == 1 ]] && printf 'Prévia da instalação' || printf 'Instalação')"

if command -v paru >/dev/null 2>&1; then
  printf 'paru já está disponível: %s\n' "$(command -v paru)"
  printf 'Concluído: nenhuma instalação necessária.\n'
  exit 0
fi

cat <<'EOF'
Etapas:
  1. Atualizar o Arch e instalar git e base-devel pelos repositórios oficiais.
  2. Clonar https://aur.archlinux.org/paru.git em diretório temporário.
  3. Mostrar o PKGBUILD e listar os demais arquivos para revisão.
  4. Após confirmação digitada, compilar e instalar com makepkg -si como usuário comum.
EOF
if (( dry_run )); then
  printf 'Prévia concluída. Nenhuma alteração realizada.\n'
  exit 0
fi

(( EUID != 0 )) || fail 'Execute como usuário comum, sem sudo.'
[[ -t 0 ]] || fail 'Um terminal interativo é necessário para revisar o PKGBUILD.'
[[ -r /etc/os-release ]] || fail 'Não foi possível identificar a distribuição.'
os_id=$(awk -F= '$1 == "ID" { gsub(/"/, "", $2); print $2 }' /etc/os-release)
[[ "$os_id" == arch ]] || fail 'Este comando só pode ser aplicado no Arch Linux.'
for command_name in sudo pacman mktemp; do
  command -v "$command_name" >/dev/null 2>&1 || fail "Comando ausente: $command_name"
done

printf '\n==> Preparando ferramentas de compilação\n'
sudo pacman -Syu --needed git base-devel
command -v git >/dev/null 2>&1 || fail 'git não encontrado após instalar git.'
command -v makepkg >/dev/null 2>&1 || fail 'makepkg não encontrado após instalar base-devel.'

printf '\n==> Baixando o PKGBUILD do paru\n'
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

printf '\n==> Compilando e instalando o paru\n'
(cd "$build_dir/paru" && makepkg -si)
command -v paru >/dev/null 2>&1 || fail 'makepkg terminou, mas paru não foi encontrado no PATH.'
printf '\nConcluído: paru instalado. Confira os pacotes com archsetup packages --dry-run.\n'
