#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
source "$PROJECT_DIR/modules/profile.sh"

usage() {
  cat <<'EOF'
Uso: archsetup packages [CATEGORIA...] [--profile NOME] [--dry-run]

Categorias: base development desktop gaming emulation
Padrão: base development desktop

O perfil é opcional. Sem ele, são usados apenas os pacotes compartilhados.
--dry-run mostra a seleção sem atualizar ou instalar nada.
EOF
}

fail() {
  printf 'Erro: %s\n' "$1" >&2
  exit 1
}

profile=""
dry_run=0
categories=()
while (( $# > 0 )); do
  case "$1" in
    --dry-run)
      dry_run=1
      shift
      ;;
    --profile)
      (( $# >= 2 )) || fail 'Falta o nome após --profile.'
      profile="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    base|development|desktop|gaming|emulation)
      categories+=("$1")
      shift
      ;;
    *)
      fail "Opção ou categoria desconhecida: $1"
      ;;
  esac
done

if (( ${#categories[@]} == 0 )); then
  categories=(base development desktop)
fi

if [[ -n "$profile" ]]; then
  [[ "$profile" =~ ^[a-z0-9][a-z0-9-]*$ ]] || fail 'Nome de perfil inválido.'
  [[ -d "$PROJECT_DIR/profiles/$profile" ]] || fail "Perfil inexistente: $profile"
fi

declare -a official=() aur=() selected_categories=()
declare -A seen_official=() seen_aur=() selected=()

read_manifest() {
  local source="$1" kind="$2" package
  [[ -f "$source" ]] || return 0
  while IFS= read -r package || [[ -n "$package" ]]; do
    [[ -z "$package" || "$package" == \#* ]] && continue
    [[ "$package" =~ ^[a-z0-9][a-z0-9@._+-]*$ ]] || fail "Pacote inválido em $source: $package"
    if [[ "$kind" == official ]]; then
      if [[ -z "${seen_official[$package]:-}" ]]; then
        official+=("$package")
        seen_official[$package]=1
      fi
    elif [[ -z "${seen_aur[$package]:-}" ]]; then
      aur+=("$package")
      seen_aur[$package]=1
    fi
  done < "$source"
}

for category in "${categories[@]}"; do
  [[ -z "${selected[$category]:-}" ]] || continue
  selected[$category]=1
  selected_categories+=("$category")
  read_manifest "$PROJECT_DIR/packages/official/$category.txt" official
  read_manifest "$PROJECT_DIR/packages/aur/$category.txt" aur
  if [[ -n "$profile" ]]; then
    read_manifest "$PROJECT_DIR/profiles/$profile/packages/official/$category.txt" official
    read_manifest "$PROJECT_DIR/profiles/$profile/packages/aur/$category.txt" aur
  fi
done

printf '==> %s de pacotes\n' "$([[ "$dry_run" == 1 ]] && printf 'Prévia' || printf 'Instalação')"
printf 'Perfil: %s\n' "${profile:-nenhum (somente pacotes compartilhados)}"
printf 'Categorias:'
printf ' %s' "${selected_categories[@]}"
printf '\n\nPacotes oficiais (%s):\n' "${#official[@]}"
installed_count=0
missing_count=0
for package in "${official[@]}"; do
  if command -v pacman >/dev/null 2>&1; then
    if pacman -Qq "$package" >/dev/null 2>&1; then
      printf '  %-28s instalado\n' "$package"
      (( installed_count += 1 ))
    else
      printf '  %-28s ausente\n' "$package"
      (( missing_count += 1 ))
    fi
  else
    printf '  %-28s não verificado (pacman indisponível)\n' "$package"
  fi
done
printf '\nPacotes AUR (%s):\n' "${#aur[@]}"
if (( ${#aur[@]} > 0 )); then
  for package in "${aur[@]}"; do
    if command -v pacman >/dev/null 2>&1; then
      if pacman -Qq "$package" >/dev/null 2>&1; then
        printf '  %-28s instalado\n' "$package"
        (( installed_count += 1 ))
      else
        printf '  %-28s ausente\n' "$package"
        (( missing_count += 1 ))
      fi
    else
      printf '  %-28s não verificado (pacman indisponível)\n' "$package"
    fi
  done
else
  printf '  nenhum\n'
fi
if command -v pacman >/dev/null 2>&1; then
  printf '\nResumo: %s instalados; %s ausentes.\n' "$installed_count" "$missing_count"
fi

if [[ -n "${seen_official[rustup]:-}" ]] && command -v pacman >/dev/null 2>&1 && pacman -Qq rust >/dev/null 2>&1; then
  printf '\nAviso: rust já está instalado. A categoria development seleciona rustup; revise eventuais conflitos apresentados pelo pacman.\n'
fi

if (( dry_run )); then
  if [[ -n "${selected[development]:-}" ]]; then
    printf '\nDocker: adicionar o usuário atual ao grupo docker, se necessário, e habilitar/iniciar docker.service.\n'
  fi
  if [[ -n "${selected[gaming]:-}" ]]; then
    printf '\nSteam exige [multilib] e bibliotecas gráficas de 32 bits compatíveis com a GPU.\n'
  fi
  printf '\nPrévia concluída. Nenhuma alteração realizada. O estado de dependências não foi calculado.\n'
  exit 0
fi

(( EUID != 0 )) || fail 'Execute como usuário comum, sem sudo.'
[[ -r /etc/os-release ]] || fail 'Não foi possível identificar a distribuição.'
os_id=$(awk -F= '$1 == "ID" { gsub(/"/, "", $2); print $2 }' /etc/os-release)
[[ "$os_id" == arch ]] || fail 'A instalação de pacotes só é permitida no Arch Linux.'
command -v pacman >/dev/null 2>&1 || fail 'pacman não encontrado.'
command -v sudo >/dev/null 2>&1 || fail 'sudo não encontrado.'
if [[ -n "${selected[development]:-}" ]]; then
  command -v getent >/dev/null 2>&1 || fail 'getent não encontrado.'
  command -v groupadd >/dev/null 2>&1 || fail 'groupadd não encontrado.'
  command -v usermod >/dev/null 2>&1 || fail 'usermod não encontrado.'
  command -v systemctl >/dev/null 2>&1 || fail 'systemctl não encontrado.'
fi
if [[ -n "$profile" ]]; then
  validate_profile_compatibility "$profile" || fail 'Aplicação recusada para este perfil.'
fi
if [[ -n "${seen_official[rustup]:-}" ]] && pacman -Qq rust >/dev/null 2>&1; then
  fail 'rust já está instalado e compartilha arquivos com rustup. Remova rust manualmente após conferir seus dependentes e execute o comando novamente.'
fi

if [[ -n "${selected[gaming]:-}" ]]; then
  command -v pacman-conf >/dev/null 2>&1 || fail 'pacman-conf não encontrado.'
  pacman-conf --repo-list | grep -qx multilib || fail 'Ative [multilib] em /etc/pacman.conf antes de instalar gaming.'
fi

aur_helper=""
if (( ${#aur[@]} > 0 )); then
  if command -v paru >/dev/null 2>&1; then
    aur_helper=paru
  elif command -v yay >/dev/null 2>&1; then
    aur_helper=yay
  else
    fail 'Pacotes AUR selecionados. Execute archsetup paru para instalar o auxiliar AUR.'
  fi
fi

printf '\n==> Atualizando o sistema e instalando pacotes oficiais\n'
sudo pacman -Syu --needed "${official[@]}"

if [[ -n "${selected[development]:-}" ]]; then
  printf '\n==> Conferindo acesso ao Docker\n'
  if ! getent group docker >/dev/null; then
    sudo groupadd --system docker
    printf 'Grupo docker criado.\n'
  fi
  current_user=$(id -un)
  if [[ " $(id -nG "$current_user") " == *' docker '* ]]; then
    printf 'Usuário %s já pertence ao grupo docker.\n' "$current_user"
  else
    sudo usermod -aG docker "$current_user"
    printf 'Usuário %s adicionado ao grupo docker. Saia e entre novamente para ativar o acesso.\n' "$current_user"
  fi
  if systemctl is-enabled --quiet docker.service; then
    printf 'Docker já está habilitado para iniciar no boot.\n'
  else
    sudo systemctl enable docker.service
    printf 'Docker habilitado para iniciar no boot.\n'
  fi
  if systemctl is-active --quiet docker.service; then
    printf 'Docker já está em execução.\n'
  else
    sudo systemctl start docker.service
    printf 'Docker iniciado.\n'
  fi
fi

if (( ${#aur[@]} > 0 )); then
  printf '\n==> Instalando pacotes AUR com %s\nRevise os PKGBUILDs apresentados.\n' "$aur_helper"
  "$aur_helper" -S --needed "${aur[@]}"
fi

if [[ -n "${selected[desktop]:-}" && -f /usr/share/applications/brave-browser.desktop ]]; then
  printf '\n==> Conferindo o navegador padrão\n'
  if command -v xdg-settings >/dev/null 2>&1 && [[ -n "${DISPLAY:-}${WAYLAND_DISPLAY:-}" ]]; then
    current_browser=$(xdg-settings get default-web-browser 2>/dev/null || true)
    if [[ "$current_browser" != brave-browser.desktop ]]; then
      xdg-settings set default-web-browser brave-browser.desktop || printf 'Defina o Brave como navegador padrão nas preferências do desktop.\n'
    fi
  else
    printf 'Na sessão gráfica, execute: xdg-settings set default-web-browser brave-browser.desktop\n'
  fi
fi

if [[ -n "${selected[development]:-}" ]]; then
  printf '\n==> Configurando Rust com rustup\n'
  if rustup toolchain list | grep -Fq '(default)'; then
    printf 'Toolchain padrão do Rust já configurada.\n'
  else
    printf 'Instalando a toolchain estável do Rust.\n'
    rustup default stable
  fi
fi

printf '\nConcluído: categorias %s processadas. Pacotes já instalados são ignorados por --needed.\n' "${selected_categories[*]}"
