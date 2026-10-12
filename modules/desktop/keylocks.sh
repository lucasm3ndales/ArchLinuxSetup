#!/usr/bin/env bash
set -euo pipefail

# Genmon lê essas tags a cada atualização; LC_ALL fixa os nomes retornados pelo xset.
if ! state=$(LC_ALL=C xset q 2>/dev/null); then
  printf '<txt><span foreground="#9ca3af">⇪  #  ⇳</span></txt><tool>Estado do teclado indisponível</tool>\n'
  exit 0
fi

lock_state() {
  local name="$1"
  if [[ "$state" =~ $name[[:space:]]+Lock:[[:space:]]+on ]]; then
    printf 'on'
  else
    printf 'off'
  fi
}

caps=$(lock_state Caps)
num=$(lock_state Num)
scroll=$(lock_state Scroll)

label() {
  local name="$1" status="$2"
  if [[ "$status" == on ]]; then
    printf '<span foreground="#fbbf24"><b>%s</b></span>' "$name"
  else
    printf '<span foreground="#9ca3af">%s</span>' "$name"
  fi
}

printf '<txt>'
label '⇪' "$caps"
printf '  '
label '#' "$num"
printf '  '
label '⇳' "$scroll"
printf '</txt><tool>Caps Lock: %s\nNum Lock: %s\nScroll Lock: %s</tool>\n' "$caps" "$num" "$scroll"
