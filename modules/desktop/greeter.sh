#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Uso: archsetup greeter [--dry-run]

Aplica o visual Nordic à tela de login do LightDM com Slick Greeter.
Execute como usuário comum depois de instalar a categoria desktop.
EOF
}

fail() {
  printf 'Erro: %s\n' "$1" >&2
  exit 1
}

dry_run=0
case "${1:-}" in
  '') ;;
  --dry-run) dry_run=1; shift ;;
  -h|--help) usage; exit 0 ;;
  *) usage >&2; exit 2 ;;
esac
(( $# == 0 )) || { usage >&2; exit 2; }

config=/etc/lightdm/slick-greeter.conf
seat_config=/etc/lightdm/lightdm.conf.d/90-archsetup-slick.conf

printf '==> %s da tela de login\n' "$([[ "$dry_run" == 1 ]] && printf 'Prévia' || printf 'Configuração')"
cat <<EOF
  Gerenciador: LightDM
  Tela de login: Slick Greeter
  GTK: Nordic; ícones: Nordzy-dark; cursor: Nordzy-cursors
  Fonte: Inter 10; fundo: #2e3440; logo: Arch Linux
  Arquivos: $config e $seat_config
EOF

if (( dry_run )); then
  printf '\nPrévia concluída. Nenhuma alteração realizada.\n'
  exit 0
fi

(( EUID != 0 )) || fail 'Execute como usuário comum, sem sudo.'
[[ -r /etc/os-release ]] || fail 'Não foi possível identificar a distribuição.'
os_id=$(awk -F= '$1 == "ID" { gsub(/"/, "", $2); print $2 }' /etc/os-release)
[[ "$os_id" == arch ]] || fail 'A configuração da tela de login só é permitida no Arch Linux.'
command -v sudo >/dev/null 2>&1 || fail 'sudo não encontrado.'
command -v lightdm >/dev/null 2>&1 || fail 'LightDM não encontrado.'
[[ -f /usr/share/xgreeters/lightdm-slick-greeter.desktop ]] || fail 'Instale lightdm-slick-greeter antes de aplicar.'
active_dm=$(readlink -f -- /etc/systemd/system/display-manager.service 2>/dev/null || true)
[[ "$active_dm" == */lightdm.service ]] || fail 'LightDM não é o gerenciador de login habilitado. Confira display-manager.service.'
[[ -d /usr/share/themes/Nordic/gtk-3.0 ]] || fail 'Tema Nordic ausente; instale a categoria desktop.'
[[ -d /usr/share/icons/Nordzy-dark ]] || fail 'Ícones Nordzy-dark ausentes; instale a categoria desktop.'
[[ -d /usr/share/icons/Nordzy-cursors/cursors ]] || fail 'Cursores Nordzy-cursors ausentes; instale a categoria desktop.'
[[ -f /usr/share/pixmaps/archlinux-logo.svg ]] || fail 'Logo do Arch ausente.'
command -v pacman >/dev/null 2>&1 || fail 'pacman não encontrado.'
pacman -Qq inter-font >/dev/null 2>&1 || fail 'Fonte Inter ausente; instale a categoria desktop.'
for path in "$config" "$seat_config"; do
  [[ ! -L "$path" ]] || fail "Não vou substituir um link simbólico: $path"
  [[ ! -e "$path" || -f "$path" ]] || fail "Caminho não é um arquivo comum: $path"
done
[[ ! -f "$config" || -r "$config" ]] || fail "Não é possível ler a configuração existente: $config"
sudo -v || fail 'Não foi possível obter acesso sudo.'

tmp_config=$(mktemp)
tmp_seat=$(mktemp)
trap 'rm -f -- "$tmp_config" "$tmp_seat"' EXIT

awk '
function emit_settings() {
  print "theme-name=Nordic"
  print "icon-theme-name=Nordzy-dark"
  print "cursor-theme-name=Nordzy-cursors"
  print "cursor-theme-size=24"
  print "font-name=Inter 10"
  print "background=#2e3440"
  print "background-color=#2e3440"
  print "draw-user-backgrounds=false"
  print "logo=/usr/share/pixmaps/archlinux-logo.svg"
}
/^\[[^]]+\][[:space:]]*$/ {
  if (in_greeter) emit_settings()
  in_greeter = ($0 ~ /^\[Greeter\][[:space:]]*$/)
  if (in_greeter) found = 1
  print
  next
}
in_greeter && /^[[:space:]]*(theme-name|icon-theme-name|cursor-theme-name|cursor-theme-size|font-name|background|background-color|draw-user-backgrounds|logo)[[:space:]]*=/ { next }
{ print }
END {
  if (in_greeter) emit_settings()
  if (!found) {
    if (NR) print ""
    print "[Greeter]"
    emit_settings()
  }
}
' "$([[ -f "$config" ]] && printf '%s' "$config" || printf '/dev/null')" > "$tmp_config"

cat > "$tmp_seat" <<'EOF'
[Seat:*]
greeter-session=lightdm-slick-greeter
EOF

update_file() {
  local target="$1" source="$2" backup
  if [[ -f "$target" ]] && sudo cmp -s -- "$source" "$target"; then
    printf 'Já configurado: %s\n' "$target"
    return 0
  fi
  if [[ -f "$target" ]]; then
    backup="$target.archsetup.bak-$(date +%Y%m%d%H%M%S%N)"
    sudo cp -p -- "$target" "$backup"
    printf 'Backup: %s\n' "$backup"
  fi
  sudo install -m 644 -- "$source" "$target"
  printf 'Atualizado: %s\n' "$target"
}

sudo install -d -m 755 -- /etc/lightdm/lightdm.conf.d
update_file "$config" "$tmp_config"
update_file "$seat_config" "$tmp_seat"
printf 'Concluído. A nova aparência aparecerá no próximo login; a sessão atual continua aberta.\n'
