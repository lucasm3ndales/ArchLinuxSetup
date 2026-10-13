#!/usr/bin/env bash
# Sourced by aur.sh and packages.sh. Records only packages added during this run.
package_tracking_begin() {
  local state_dir="${XDG_STATE_HOME:-$HOME/.local/state}/archlinuxsetup"
  mkdir -p -- "$state_dir"
  package_tracking_before=$(mktemp -- "$state_dir/packages-before.XXXXXXXX")
  pacman -Qq | LC_ALL=C sort -u > "$package_tracking_before" || {
    rm -f -- "$package_tracking_before"
    printf 'Erro: não foi possível registrar os pacotes existentes.\n' >&2
    return 1
  }
  if [[ ! -e "$state_dir/packages-baseline" ]]; then
    cp -- "$package_tracking_before" "$state_dir/packages-baseline"
  fi
  package_tracking_state_dir="$state_dir"
}

package_tracking_finish() {
  [[ -n "${package_tracking_before:-}" ]] || return 0
  local after delta merged manifest="$package_tracking_state_dir/packages-installed"
  after=$(mktemp -- "$package_tracking_state_dir/packages-after.XXXXXXXX") || return 1
  delta=$(mktemp -- "$package_tracking_state_dir/packages-delta.XXXXXXXX") || { rm -f -- "$after"; return 1; }
  merged=$(mktemp -- "$package_tracking_state_dir/packages-merged.XXXXXXXX") || { rm -f -- "$after" "$delta"; return 1; }
  if pacman -Qq | LC_ALL=C sort -u > "$after"; then
    LC_ALL=C comm -13 "$package_tracking_before" "$after" > "$delta"
    { [[ ! -f "$manifest" ]] || cat -- "$manifest"; cat -- "$delta"; } | LC_ALL=C sort -u > "$merged"
    mv -- "$merged" "$manifest"
    if [[ -s "$delta" ]]; then
      printf 'Pacotes novos registrados para possível remoção: %s\n' "$(wc -l < "$delta")"
    fi
  else
    printf 'Aviso: não foi possível atualizar o registro de pacotes após a instalação.\n' >&2
  fi
  rm -f -- "$package_tracking_before" "$after" "$delta" "$merged"
  package_tracking_before=''
}
