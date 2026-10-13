#!/usr/bin/env bash
set -euo pipefail

usage() {
  printf 'Uso: archsetup aur [--dry-run]\nInstala o auxiliar paru para pacotes da AUR.\n'
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

if command -v pacman >/dev/null 2>&1; then
  if pacman -Qq rustup >/dev/null 2>&1; then
    printf 'Rust: rustup já instalado.\n'
  elif pacman -Qq rust >/dev/null 2>&1; then
    printf 'Rust: rust já instalado; rustup não será instalado.\n'
  else
    printf 'Rust: rustup será instalado e configurado antes da compilação.\n'
  fi
else
  printf 'Rust: estado não verificado (pacman indisponível).\n'
fi

cat <<'EOF'
Etapas:
  1. Atualizar o Arch e instalar git, base-devel e uma ferramenta Rust, se necessário.
  2. Clonar https://aur.archlinux.org/paru.git em diretório temporário.
  3. Mostrar o PKGBUILD e listar os demais arquivos.
  4. Compilar e instalar com makepkg -si --noconfirm como usuário comum.
EOF
if (( dry_run )); then
  printf 'Prévia concluída. Nenhuma alteração realizada.\n'
  exit 0
fi

(( EUID != 0 )) || fail 'Execute como usuário comum, sem sudo.'
[[ -t 0 ]] || fail 'Um terminal interativo é necessário caso sudo solicite autenticação.'
[[ -r /etc/os-release ]] || fail 'Não foi possível identificar a distribuição.'
os_id=$(awk -F= '$1 == "ID" { gsub(/"/, "", $2); print $2 }' /etc/os-release)
[[ "$os_id" == arch ]] || fail 'Este comando só pode ser aplicado no Arch Linux.'
for command_name in sudo pacman mktemp; do
  command -v "$command_name" >/dev/null 2>&1 || fail "Comando ausente: $command_name"
done

printf '\n==> Preparando ferramentas de compilação\n'
sudo -v || fail 'Não foi possível obter acesso sudo.'
build_packages=(git base-devel)
if pacman -Qq rustup >/dev/null 2>&1; then
  printf 'rustup já instalado; será usado para compilar o paru.\n'
elif pacman -Qq rust >/dev/null 2>&1; then
  printf 'rust já instalado; será usado para compilar o paru.\n'
else
  build_packages+=(rustup)
  printf 'rustup será instalado antes de compilar o paru.\n'
fi
sudo pacman -Syu --needed --noconfirm "${build_packages[@]}"
command -v git >/dev/null 2>&1 || fail 'git não encontrado após instalar git.'
command -v makepkg >/dev/null 2>&1 || fail 'makepkg não encontrado após instalar base-devel.'
if pacman -Qq rustup >/dev/null 2>&1; then
  if ! rustup toolchain list | grep -Fq '(default)'; then
    printf 'Configurando a toolchain Rust estável para compilar o paru.\n'
    rustup default stable
  fi
fi
command -v cargo >/dev/null 2>&1 || fail 'cargo não encontrado após preparar Rust.'

printf '\n==> Baixando o PKGBUILD do paru\n'
build_dir=$(mktemp -d -t archlinuxsetup-paru.XXXXXXXX)
trap 'rm -rf -- "$build_dir"' EXIT
git clone -- https://aur.archlinux.org/paru.git "$build_dir/paru"
[[ -f "$build_dir/paru/PKGBUILD" ]] || fail 'O repositório clonado não contém PKGBUILD.'

printf '\nArquivos recebidos da AUR:\n'
git -C "$build_dir/paru" ls-files
printf '\nPKGBUILD recebido:\n'
cat -- "$build_dir/paru/PKGBUILD"
printf '\nDemais arquivos recebidos: %s\n' "$build_dir/paru"

printf '\n==> Compilando e instalando o paru\n'
(cd "$build_dir/paru" && makepkg -si --noconfirm)
command -v paru >/dev/null 2>&1 || fail 'makepkg terminou, mas paru não foi encontrado no PATH.'
printf '\nConcluído: paru instalado. Confira os pacotes com archsetup packages --dry-run.\n'
