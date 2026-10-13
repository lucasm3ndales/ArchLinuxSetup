#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
state_dir="${XDG_STATE_HOME:-$HOME/.local/state}/archlinuxsetup"
backup_base="$state_dir/backups"
target="$HOME/.local/bin/archsetup"
path_line='export PATH="$HOME/.local/bin:$PATH"'
restore=0
remove_packages=0
dry_run=0

usage() {
  cat <<'EOF'
Uso: archsetup uninstall [--restore [--packages]] [--dry-run]

Sem opções, remove o comando e a linha PATH adicionada pelo ArchSetup.
--restore também recupera as configurações com backup anterior.
--packages remove somente pacotes registrados como novos pelo ArchSetup.
--dry-run mostra as ações sem alterar o sistema.
Execute --restore fora da sessão gráfica do Xfce para restaurar seus arquivos.
EOF
}
fail() { printf 'Erro: %s\n' "$1" >&2; exit 1; }

while (( $# )); do
  case "$1" in
    --restore) restore=1 ;;
    --packages) remove_packages=1 ;;
    --dry-run) dry_run=1 ;;
    -h|--help) usage; exit 0 ;;
    *) fail "Opção desconhecida: $1" ;;
  esac
  shift
done
(( ! remove_packages || restore )) || fail '--packages exige --restore.'
(( EUID != 0 )) || fail 'Execute como usuário comum, sem sudo.'

timestamp=$(date +%Y%m%d%H%M%S)
uninstall_backup="$backup_base/uninstall-$timestamp-$$"
rc=''
if [[ -f "$state_dir/path-rc" ]]; then
  IFS= read -r rc < "$state_dir/path-rc" || true
  [[ "$rc" == "$HOME/.bashrc" || "$rc" == "$HOME/.zshrc" ]] || fail 'Registro de PATH inválido.'
else
  for candidate_rc in "$HOME/.bashrc" "$HOME/.zshrc"; do
    [[ -f "$candidate_rc" ]] || continue
    grep -Fqx "$path_line" "$candidate_rc" || continue
    while IFS= read -r candidate_backup; do
      [[ -f "$candidate_backup" ]] || continue
      if ! grep -Fqx "$path_line" "$candidate_backup"; then
        rc="$candidate_rc"
        break 2
      fi
    done < <(find "$HOME" -maxdepth 1 -type f -name "$(basename -- "$candidate_rc").archsetup.bak-*" -print | LC_ALL=C sort)
  done
fi

printf '==> Remoção do ArchSetup\n'
if [[ -L "$target" && "$(readlink -- "$target")" == "$PROJECT_DIR/archsetup" ]]; then
  printf 'Comando: remover %s\n' "$target"
elif [[ -e "$target" || -L "$target" ]]; then
  fail "$target existe, mas não pertence a este repositório."
else
  printf 'Comando: link já ausente.\n'
fi
if [[ -n "$rc" && -f "$rc" ]] && grep -Fqx "$path_line" "$rc"; then
  printf 'PATH: remover a linha adicionada em %s\n' "$rc"
fi

xfce_backup=''
if (( restore )); then
  if [[ -d "$backup_base" ]]; then
    while IFS= read -r candidate; do
      [[ -d "$candidate" ]] || continue
      xfce_backup="$candidate"
      break
    done < <(find "$backup_base" -maxdepth 1 -type d -name 'xfce-*' -print | LC_ALL=C sort)
  fi
  if [[ -n "$xfce_backup" ]]; then
    printf 'Xfce: restaurar configuração anterior de %s\n' "$xfce_backup"
  else
    printf 'Xfce: nenhum backup anterior encontrado; nada será alterado.\n'
  fi
  printf 'LightDM e touchpad: restaurar backups anteriores quando disponíveis.\n'
  printf 'Docker e navegador: restaurar estado anterior quando registrado.\n'
fi

packages_to_remove=()
if (( remove_packages )); then
  manifest="$state_dir/packages-installed"
  [[ -f "$manifest" ]] || fail 'Registro de pacotes novos ausente. Não é seguro inferir o que remover.'
  command -v pacman >/dev/null 2>&1 || fail 'pacman não encontrado.'
  while IFS= read -r package || [[ -n "$package" ]]; do
    [[ "$package" =~ ^[a-z0-9][a-z0-9@._+-]*$ ]] || fail "Nome de pacote inválido no registro: $package"
    if pacman -Qq "$package" >/dev/null 2>&1; then
      packages_to_remove+=("$package")
    fi
  done < "$manifest"
  printf 'Pacotes registrados e ainda instalados: %s\n' "${#packages_to_remove[@]}"
  printf 'A lista inclui dependências novas da instalação e pode incluir pacotes usados depois por você.\n'
  printf 'Registro: %s\n' "$manifest"
  if (( ${#packages_to_remove[@]} > 0 )); then
    printf '  %s\n' "${packages_to_remove[@]}"
    if (( ! dry_run )); then
      pacman -R --print -- "${packages_to_remove[@]}" >/dev/null || fail 'A remoção quebraria dependências. Revise os pacotes antes de tentar novamente.'
    fi
  fi
fi

if (( dry_run )); then
  printf 'Prévia concluída. Nenhuma alteração realizada.\n'
  exit 0
fi

if (( restore )) && [[ -n "$xfce_backup" ]]; then
  [[ -z "${DISPLAY:-}${WAYLAND_DISPLAY:-}" ]] || fail 'Saia da sessão gráfica e execute --restore em um TTY para restaurar o Xfce.'
  if command -v pgrep >/dev/null 2>&1; then
    pgrep -u "$(id -u)" -x xfconfd >/dev/null 2>&1 && fail 'xfconfd está ativo. Encerre a sessão Xfce antes de restaurar.'
  fi
fi

backup_user_file() {
  local file="$1" name="${1#"$HOME/"}"
  if [[ -e "$file" || -L "$file" ]]; then
    mkdir -p -- "$uninstall_backup/$(dirname -- "$name")"
    cp -a -- "$file" "$uninstall_backup/$name"
  fi
}

restore_user_file() {
  local file="$1" original="$2"
  [[ ! -L "$file" ]] || fail "Link simbólico inesperado: $file"
  backup_user_file "$file"
  if [[ -f "$original" ]]; then
    mkdir -p -- "$(dirname -- "$file")"
    cp -p -- "$original" "$file"
  else
    rm -f -- "$file"
  fi
}

oldest_backup() {
  local path="$1" found=''
  while IFS= read -r candidate; do
    [[ -f "$candidate" ]] || continue
    found="$candidate"
    break
  done < <(find "$(dirname -- "$path")" -maxdepth 1 -type f \( -name "$(basename -- "$path").archsetup.bak-*" -o -name "$(basename -- "$path").archlinuxsetup.bak-*" \) -print 2>/dev/null | LC_ALL=C sort)
  printf '%s' "$found"
}

restore_root_file() {
  local file="$1" backup="$2" remove_owned="$3" marker="$state_dir/root-created/$(basename -- "$1")"
  [[ ! -L "$file" ]] || fail "Link simbólico inesperado: $file"
  if [[ "$remove_owned" == yes && -f "$marker" ]]; then
    if [[ -f "$file" ]] && grep -Fqx "$(sudo sha256sum -- "$file" | cut -d ' ' -f 1) $file" "$marker"; then
      sudo cp -p -- "$file" "$file.archsetup.pre-uninstall-$timestamp"
      sudo rm -f -- "$file"
      printf 'Removido arquivo criado pelo ArchSetup: %s\n' "$file"
    elif [[ -f "$file" ]]; then
      printf 'Mantido porque foi alterado após o ArchSetup: %s\n' "$file"
    fi
  elif [[ -n "$backup" ]]; then
    if [[ -f "$file" ]]; then sudo cp -p -- "$file" "$file.archsetup.pre-uninstall-$timestamp"; fi
    sudo cp -p -- "$backup" "$file"
    printf 'Restaurado: %s\n' "$file"
  elif [[ -f "$file" ]]; then
    printf 'Mantido sem backup anterior comprovado: %s\n' "$file"
  fi
}

if (( restore )); then
  root_files=(/etc/lightdm/slick-greeter.conf /etc/lightdm/lightdm.conf.d/90-archsetup-slick.conf /etc/X11/xorg.conf.d/90-archlinuxsetup-touchpad.conf /etc/X11/xorg.conf.d/91-archlinuxsetup-acer-tapping.conf)
  root_backups=()
  root_changes=0
  for file in "${root_files[@]}"; do
    [[ ! -L "$file" ]] || fail "Link simbólico inesperado: $file"
    root_backups+=("$(oldest_backup "$file")")
  done
  for i in "${!root_files[@]}"; do
    [[ -n "${root_backups[$i]}" || ( -f "${root_files[$i]}" && -f "$state_dir/root-created/$(basename -- "${root_files[$i]}")" ) ]] && root_changes=1
  done
  [[ ! -f "$state_dir/docker-before" ]] || root_changes=1
  if [[ -f "$state_dir/docker-before" ]]; then
    mapfile -t docker_before < "$state_dir/docker-before"
    [[ ${#docker_before[@]} -eq 4 && "${docker_before[0]}" == "$(id -un)" ]] || fail 'Registro anterior do Docker inválido.'
    for value in "${docker_before[@]:1}"; do [[ "$value" == yes || "$value" == no ]] || fail 'Registro anterior do Docker inválido.'; done
  fi
  if (( root_changes )); then
    command -v sudo >/dev/null 2>&1 || fail 'sudo não encontrado.'
    sudo -v || fail 'Não foi possível obter acesso sudo.'
  fi
  mkdir -p -- "$uninstall_backup"
  if [[ -n "$xfce_backup" ]]; then
    for channel in xsettings xfwm4 xfce4-panel xfce4-keyboard-shortcuts; do
      file="${XDG_CONFIG_HOME:-$HOME/.config}/xfce4/xfconf/xfce-perchannel-xml/$channel.xml"
      restore_user_file "$file" "$xfce_backup/$channel.xml"
    done
    autostart="${XDG_CONFIG_HOME:-$HOME/.config}/autostart/ksuperkey.desktop"
    if [[ -f "$xfce_backup/ksuperkey.desktop" ]]; then
      restore_user_file "$autostart" "$xfce_backup/ksuperkey.desktop"
    elif [[ -f "$autostart" ]] && grep -Fqx 'Name=ArchLinuxSetup Super key' "$autostart"; then
      restore_user_file "$autostart" /dev/null
    fi
    if [[ -f "$xfce_backup/archlinuxsetup-keylocks" ]]; then
      restore_user_file "$HOME/.local/bin/archlinuxsetup-keylocks" "$xfce_backup/archlinuxsetup-keylocks"
    fi
    printf 'Xfce restaurado. A configuração será carregada na próxima sessão.\n'
  fi
  if [[ -f "$state_dir/browser-before/recorded" ]]; then
    for label in .config/mimeapps.list .local/share/applications/mimeapps.list; do
      restore_user_file "$HOME/$label" "$state_dir/browser-before/$label"
    done
    printf 'Associações do navegador restauradas.\n'
  fi
  for i in "${!root_files[@]}"; do
    owned=yes
    restore_root_file "${root_files[$i]}" "${root_backups[$i]}" "$owned"
  done
  if [[ -f "$state_dir/docker-before" ]]; then
    if [[ "${docker_before[1]}" == no ]] && [[ " $(id -nG) " == *' docker '* ]]; then sudo gpasswd -d "$(id -un)" docker; fi
    if [[ "${docker_before[1]}" == yes ]] && [[ " $(id -nG) " != *' docker '* ]]; then sudo usermod -aG docker "$(id -un)"; fi
    if [[ "${docker_before[2]}" == no ]]; then sudo systemctl disable docker.service; else sudo systemctl enable docker.service; fi
    if [[ "${docker_before[3]}" == no ]]; then sudo systemctl stop docker.service; else sudo systemctl start docker.service; fi
    printf 'Estado anterior de grupo e serviço Docker restaurado.\n'
  fi
  if (( remove_packages )) && (( ${#packages_to_remove[@]} > 0 )); then
    sudo pacman -R --noconfirm -- "${packages_to_remove[@]}"
    printf 'Pacotes registrados removidos. Dependências não registradas foram preservadas.\n'
  fi
  if (( remove_packages )); then
    cp -p -- "$manifest" "$uninstall_backup/packages-installed"
    : > "$manifest"
    printf 'Registro de pacotes ativos arquivado em %s.\n' "$uninstall_backup/packages-installed"
  fi
fi

if [[ -n "$rc" && -f "$rc" ]] && grep -Fqx "$path_line" "$rc"; then
  backup_user_file "$rc"
  temp_rc=$(mktemp -- "${rc}.archsetup-uninstall.XXXXXXXX")
  grep -Fvx "$path_line" "$rc" > "$temp_rc" || true
  cat -- "$temp_rc" > "$rc"
  rm -f -- "$temp_rc"
  printf 'PATH removido de %s.\n' "$rc"
fi
if [[ -L "$target" ]]; then
  rm -- "$target"
  printf 'Comando removido.\n'
fi
printf 'Concluído. Backups preservados em %s.\n' "$backup_base"
