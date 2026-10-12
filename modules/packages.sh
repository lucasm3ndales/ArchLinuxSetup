#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"

usage() {
  cat <<'EOF'
Uso: ./install.sh packages <plan|apply> [--profile NOME] [CATEGORIA...]

Categorias: base development desktop gaming emulation
Padrão: base development desktop

O perfil é opcional. Sem ele, são usados apenas os pacotes compartilhados.
Use plan para revisar a seleção antes de executar apply.
EOF
}

fail() {
  printf 'Erro: %s\n' "$1" >&2
  exit 1
}

action="${1:-}"
[[ "$action" == plan || "$action" == apply ]] || { usage >&2; exit 2; }
shift

profile=""
categories=()
while (( $# > 0 )); do
  case "$1" in
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

printf 'Perfil: %s\n' "${profile:-nenhum (somente pacotes compartilhados)}"
printf 'Categorias:'
printf ' %s' "${selected_categories[@]}"
printf '\n\nPacotes oficiais (%s):\n' "${#official[@]}"
printf '  %s\n' "${official[@]}"
printf '\nPacotes AUR (%s):\n' "${#aur[@]}"
if (( ${#aur[@]} > 0 )); then
  printf '  %s\n' "${aur[@]}"
else
  printf '  nenhum\n'
fi

if [[ "$action" == plan ]]; then
  if [[ -n "${selected[gaming]:-}" ]]; then
    printf '\nSteam exige [multilib] e bibliotecas gráficas de 32 bits compatíveis com a GPU.\n'
  fi
  printf '\nNenhuma alteração foi feita.\n'
  exit 0
fi

(( EUID != 0 )) || fail 'Execute como usuário comum, sem sudo.'
[[ -r /etc/os-release ]] || fail 'Não foi possível identificar a distribuição.'
os_id=$(awk -F= '$1 == "ID" { gsub(/"/, "", $2); print $2 }' /etc/os-release)
[[ "$os_id" == arch ]] || fail 'A instalação de pacotes só é permitida no Arch Linux.'
command -v pacman >/dev/null 2>&1 || fail 'pacman não encontrado.'
command -v sudo >/dev/null 2>&1 || fail 'sudo não encontrado.'

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
    fail 'Pacotes AUR selecionados. Execute ./install.sh aur plan e ./install.sh aur apply para instalar paru.'
  fi
fi

printf '\nIniciando atualização completa e instalação dos pacotes oficiais.\n'
sudo pacman -Syu --needed "${official[@]}"

if (( ${#aur[@]} > 0 )); then
  printf '\nInstalando pacotes AUR com %s. Revise os PKGBUILDs apresentados.\n' "$aur_helper"
  "$aur_helper" -S --needed "${aur[@]}"
fi

if [[ -n "${selected[desktop]:-}" && -f /usr/share/applications/brave-browser.desktop ]]; then
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
  eval "$(fnm env --shell bash)"
  default_node=$(fnm default 2>/dev/null || true)
  if [[ -n "$default_node" ]]; then
    printf '\nNode.js padrão do fnm já configurado: %s.\n' "$default_node"
  else
    printf '\nInstalando a versão LTS do Node.js com fnm.\n'
    fnm install --lts --use
    fnm default "$(fnm current)"
  fi

  shell_name="${SHELL:-/bin/bash}"
  shell_name="${shell_name##*/}"
  case "$shell_name" in
    bash|zsh)
      shell_rc="$HOME/.$shell_name"rc
      if [[ -f "$shell_rc" ]] && grep -Fq 'fnm env' "$shell_rc"; then
        printf 'O arquivo %s já contém configuração do fnm; preservei a configuração existente.\n' "$shell_rc"
      else
        if [[ -f "$shell_rc" ]]; then
          cp -p -- "$shell_rc" "$shell_rc.archlinuxsetup.bak-$(date +%Y%m%d%H%M%S)"
        fi
        printf '\n%s\n' "eval \"\$(fnm env --use-on-cd --shell $shell_name)\"" >> "$shell_rc"
        printf 'Integração do fnm adicionada a %s. Abra um novo terminal para usá-la.\n' "$shell_rc"
      fi
      ;;
    *)
      printf 'Configure fnm env no shell %s para disponibilizar o Node em novos terminais.\n' "$shell_name"
      ;;
  esac
fi
