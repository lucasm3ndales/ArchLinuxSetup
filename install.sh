#!/usr/bin/env bash
set -euo pipefail

fail() {
    printf 'Error: %s\n' "$1" >&2
    exit 1
}

if (( $# != 0 )); then
    fail 'install.sh does not accept arguments.'
fi

if (( EUID == 0 )); then
    fail 'Run as a regular user, without sudo.'
fi

if [[ ! -r /etc/os-release ]] || ! grep -Eq '^ID="?arch"?$' /etc/os-release; then
    fail 'This installer requires Arch Linux.'
fi

command -v pacman >/dev/null 2>&1 || fail 'pacman not found.'
command -v sudo >/dev/null 2>&1 || fail 'sudo not found.'

repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
[[ -f "$repo_dir/Cargo.toml" && -f "$repo_dir/Cargo.lock" && -f "$repo_dir/src/main.rs" ]] \
    || fail 'install.sh must be in the ArchSetup repository root.'

packages=(git base-devel)
if pacman -Qq rustup >/dev/null 2>&1; then
    packages+=(rustup)
elif pacman -Qq rust >/dev/null 2>&1; then
    packages+=(rust)
elif ! command -v cargo >/dev/null 2>&1 || ! command -v rustc >/dev/null 2>&1; then
    packages+=(rustup)
fi

missing_packages=()
for package in "${packages[@]}"; do
    if ! pacman -Qq "$package" >/dev/null 2>&1; then
        missing_packages+=("$package")
    fi
done
if (( ${#missing_packages[@]} > 0 )); then
    printf '==> Updating Arch and installing missing build tools: %s\n' "${missing_packages[*]}"
    sudo pacman -Syu --needed --noconfirm "${missing_packages[@]}"
else
    printf '==> Build tools are already installed; skipping package update\n'
fi

if command -v rustup >/dev/null 2>&1 \
    && ! pacman -Qq rust >/dev/null 2>&1; then
    toolchains="$(rustup toolchain list)" || fail 'Could not list rustup toolchains.'
    if ! grep -Fq '(default)' <<< "$toolchains"; then
        printf '\n==> Configuring the stable Rust toolchain\n'
        rustup default stable || fail 'Could not configure the stable Rust toolchain.'
    fi
fi

command -v cargo >/dev/null 2>&1 || fail 'Cargo was not found after installation.'
command -v rustc >/dev/null 2>&1 || fail 'Rust was not found after installation.'
cargo --version >/dev/null 2>&1 || fail 'Cargo is in PATH but does not work. Check rustup toolchain list and run rustup default stable.'
rustc --version >/dev/null 2>&1 || fail 'Rust is in PATH but does not work. Check rustup toolchain list and run rustup default stable.'

printf '\n==> Building ArchSetup\n'
cd -- "$repo_dir"
cargo build --release --locked

printf '\n==> Creating the archsetup command\n'
./target/release/archsetup link

printf '\nComplete. Open a new terminal and run archsetup doctor.\n'
