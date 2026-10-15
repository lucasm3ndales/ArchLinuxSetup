#!/usr/bin/env bash
set -euo pipefail

fail() {
    printf 'Erro: %s\n' "$1" >&2
    exit 1
}

if (( $# != 0 )); then
    fail 'install.sh não aceita argumentos.'
fi

if (( EUID == 0 )); then
    fail 'Execute como usuário comum, sem sudo.'
fi

if [[ ! -r /etc/os-release ]] || ! grep -Eq '^ID="?arch"?$' /etc/os-release; then
    fail 'Este instalador exige Arch Linux.'
fi

command -v pacman >/dev/null 2>&1 || fail 'pacman não encontrado.'
command -v sudo >/dev/null 2>&1 || fail 'sudo não encontrado.'

repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
[[ -f "$repo_dir/Cargo.toml" && -f "$repo_dir/Cargo.lock" && -f "$repo_dir/src/main.rs" ]] \
    || fail 'install.sh precisa estar na raiz do repositório ArchSetup.'

packages=(git base-devel)
if pacman -Qq rustup >/dev/null 2>&1; then
    packages+=(rustup)
elif pacman -Qq rust >/dev/null 2>&1; then
    packages+=(rust)
elif ! command -v cargo >/dev/null 2>&1 || ! command -v rustc >/dev/null 2>&1; then
    packages+=(rustup)
fi

printf '==> Atualizando o Arch e preparando Git, ferramentas de compilação e Rust\n'
sudo pacman -Syu --needed --noconfirm "${packages[@]}"

if command -v rustup >/dev/null 2>&1 \
    && ! pacman -Qq rust >/dev/null 2>&1 \
    && ! rustup show active-toolchain >/dev/null 2>&1; then
    printf '\n==> Configurando a toolchain Rust estável\n'
    rustup default stable
fi

command -v cargo >/dev/null 2>&1 || fail 'Cargo não foi encontrado após a instalação.'
command -v rustc >/dev/null 2>&1 || fail 'Rust não foi encontrado após a instalação.'

printf '\n==> Compilando o ArchSetup\n'
cd -- "$repo_dir"
cargo build --release --locked

printf '\n==> Criando o comando archsetup\n'
./target/release/archsetup link

printf '\nConcluído. Abra um novo terminal e execute archsetup doctor.\n'
