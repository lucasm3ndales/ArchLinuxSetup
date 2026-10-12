#!/usr/bin/env bash

validate_profile_compatibility() {
  local profile="$1" vendor product
  case "$profile" in
    acer-aspire-vx5-591g-54pg)
      if [[ ! -r /sys/class/dmi/id/sys_vendor || ! -r /sys/class/dmi/id/product_name ]]; then
        printf 'Erro: não foi possível ler o fabricante e o modelo desta máquina.\n' >&2
        return 1
      fi
      IFS= read -r vendor < /sys/class/dmi/id/sys_vendor || true
      IFS= read -r product < /sys/class/dmi/id/product_name || true
      if [[ "${vendor,,}" != *acer* || "${product,,}" != *vx5-591g* ]]; then
        printf 'Erro: o perfil %s exige um Acer VX5-591G; esta máquina informa %s / %s.\n' "$profile" "$vendor" "$product" >&2
        return 1
      fi
      ;;
    *)
      printf 'Erro: perfil sem validação de hardware: %s.\n' "$profile" >&2
      return 1
      ;;
  esac
}
