#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Uso:
  ./install.sh storage plan
  ./install.sh storage apply --profile acer-aspire-vx5-591g-54pg --disk /dev/disk/by-id/ID_DO_HD
  ./install.sh storage unlock --profile acer-aspire-vx5-591g-54pg

apply APAGA completamente o disco indicado. Leia o plano e confirme o identificador no notebook.
unlock solicita a senha LUKS e monta /mnt/dados após a preparação.
EOF
}

fail() { printf 'Erro: %s\n' "$1" >&2; exit 1; }

action="${1:-}"
case "$action" in plan|apply|unlock) shift ;; *) usage >&2; exit 2 ;; esac
acer_profile=acer-aspire-vx5-591g-54pg

if [[ "$action" == plan ]]; then
  (( $# == 0 )) || { usage >&2; exit 2; }
  printf 'Discos presentes; confirme o HD de 1 TB por modelo, capacidade e caminho by-id:\n'
  lsblk -dn -o NAME,MODEL,SIZE,ROTA,TYPE
  printf '\nPartições, sistemas de arquivos e montagens:\n'
  lsblk -o NAME,SIZE,TYPE,FSTYPE,UUID,MOUNTPOINTS
  cat <<'EOF'

Destino: HD secundário rotativo de aproximadamente 1 TB, LUKS2 + ext4.
Ponto de montagem: /mnt/dados; pastas Dados e Jogos para o usuário dev.
Desbloqueio manual: ./install.sh storage unlock (pede senha LUKS).
Nenhuma alteração foi feita.
EOF
  exit 0
fi

(( EUID != 0 )) || fail 'Execute como usuário comum, sem sudo.'
[[ -r /etc/os-release ]] || fail 'Não foi possível identificar a distribuição.'
os_id=$(awk -F= '$1 == "ID" { gsub(/"/, "", $2); print $2 }' /etc/os-release)
[[ "$os_id" == arch ]] || fail 'Este comando só pode ser aplicado no Arch Linux.'
for command_name in sudo lsblk cryptsetup blkid mount systemctl readlink findmnt awk; do
  command -v "$command_name" >/dev/null 2>&1 || fail "Comando ausente: $command_name"
done
id dev >/dev/null 2>&1 || fail 'O usuário dev precisa existir antes de preparar o HD.'
mapper=archlinuxsetup-dados
mountpoint=/mnt/dados

if [[ "$action" == unlock ]]; then
  [[ $# == 2 && "$1" == --profile && "$2" == "$acer_profile" ]] || { usage >&2; exit 2; }
  if findmnt -rn --mountpoint "$mountpoint" >/dev/null 2>&1; then
    printf 'HD já montado em %s.\n' "$mountpoint"
    exit 0
  fi
  [[ -f /etc/crypttab ]] || fail 'Entrada do HD ausente em /etc/crypttab.'
  luks_uuid=$(awk '$1 == "archlinuxsetup-dados" && $2 ~ /^UUID=/ { sub(/^UUID=/, "", $2); print $2 }' /etc/crypttab)
  [[ "$luks_uuid" =~ ^[a-fA-F0-9-]{36}$ ]] || fail 'UUID LUKS inválido ou ausente em /etc/crypttab.'
  [[ -e "/dev/disk/by-uuid/$luks_uuid" ]] || fail 'HD criptografado não encontrado.'
  if [[ ! -e "/dev/mapper/$mapper" ]]; then
    sudo cryptsetup open "/dev/disk/by-uuid/$luks_uuid" "$mapper"
  fi
  sudo mount "$mountpoint"
  printf 'HD desbloqueado e montado em %s.\n' "$mountpoint"
  exit 0
fi

[[ $# == 4 && "$1" == --profile && "$2" == "$acer_profile" && "$3" == --disk ]] || { usage >&2; exit 2; }
requested_disk="$4"
[[ -r /sys/class/dmi/id/product_name ]] || fail 'Não foi possível confirmar o modelo do notebook pelo firmware.'
IFS= read -r model < /sys/class/dmi/id/product_name || true
[[ "$model" == *VX5-591G* ]] || fail "Perfil Acer incompatível com o modelo detectado: ${model:-desconhecido}."
[[ "$requested_disk" == /dev/disk/by-id/* ]] || fail 'Use um caminho estável /dev/disk/by-id/... para o HD.'
[[ -b "$requested_disk" ]] || fail 'O caminho informado não é um dispositivo de bloco.'
disk=$(readlink -f -- "$requested_disk")
[[ -b "$disk" ]] || fail 'Não foi possível resolver o disco.'
[[ "$(lsblk -dnro TYPE "$disk")" == disk ]] || fail 'O caminho precisa apontar para o disco inteiro.'
[[ "$(lsblk -dnro ROTA "$disk")" == 1 ]] || fail 'O disco informado não é rotativo; confira se selecionou o HD.'
size_bytes=$(lsblk -bdnro SIZE "$disk")
(( size_bytes >= 850000000000 && size_bytes <= 1200000000000 )) || fail 'Capacidade fora da faixa esperada para o HD de 1 TB.'
root_source=$(findmnt -nro SOURCE /)
root_source="${root_source%%\[*}"
if [[ -b "$root_source" ]] && lsblk -spno NAME "$root_source" | grep -Fxq "$disk"; then
  fail 'O disco informado contém o sistema raiz.'
fi
[[ -z "$(lsblk -nrpo MOUNTPOINTS "$disk" | awk 'NF { print; exit }')" ]] || fail 'O disco ou alguma partição está montado ou em uso como swap.'
[[ ! -e "/dev/mapper/$mapper" ]] || fail "Já existe um mapeamento /dev/mapper/$mapper."
[[ -f /etc/fstab ]] || fail 'Arquivo /etc/fstab ausente.'
if findmnt -rn --mountpoint "$mountpoint" >/dev/null 2>&1; then
  fail "$mountpoint já é um ponto de montagem."
fi
if [[ -e "$mountpoint" ]]; then
  [[ ! -L "$mountpoint" && -d "$mountpoint" && -z "$(find "$mountpoint" -mindepth 1 -maxdepth 1 -print -quit)" ]] || fail "$mountpoint precisa estar ausente ou vazio."
fi
if [[ -f /etc/crypttab ]] && awk '$1 == "archlinuxsetup-dados" { found=1 } END { exit !found }' /etc/crypttab; then
  fail 'Já existe uma entrada archlinuxsetup-dados em /etc/crypttab.'
fi
if [[ -f /etc/fstab ]] && awk '$2 == "/mnt/dados" { found=1 } END { exit !found }' /etc/fstab; then
  fail 'Já existe uma montagem /mnt/dados em /etc/fstab.'
fi
for command_name in sfdisk mkfs.ext4 udevadm find; do
  command -v "$command_name" >/dev/null 2>&1 || fail "Comando ausente: $command_name"
done

printf '\nATENÇÃO: todas as partições e arquivos de %s (%s) serão perdidos.\n' "$requested_disk" "$disk"
lsblk -o NAME,MODEL,SIZE,TYPE,FSTYPE,UUID,MOUNTPOINTS "$disk"
printf 'Digite exatamente APAGAR %s para continuar: ' "$disk"
IFS= read -r confirmation
[[ "$confirmation" == "APAGAR $disk" ]] || fail 'Confirmação incorreta; nenhuma alteração foi feita.'
sudo -v

printf 'label: gpt\n,\n' | sudo sfdisk --wipe always --wipe-partitions always "$disk"
sudo udevadm settle
mapfile -t partitions < <(lsblk -nrpo NAME,TYPE "$disk" | awk '$2 == "part" { print $1 }')
(( ${#partitions[@]} == 1 )) || fail 'Partição nova não identificada; interrompido antes da formatação.'
partition="${partitions[0]}"
sudo cryptsetup luksFormat --type luks2 "$partition"
sudo cryptsetup open "$partition" "$mapper"
sudo mkfs.ext4 -L dados "/dev/mapper/$mapper"

luks_uuid=$(sudo cryptsetup luksUUID "$partition")
fs_uuid=$(sudo blkid -s UUID -o value "/dev/mapper/$mapper")
[[ "$luks_uuid" =~ ^[a-fA-F0-9-]{36}$ && "$fs_uuid" =~ ^[a-fA-F0-9-]{36}$ ]] || fail 'UUID não pôde ser validado; confira o disco antes de montar.'
stamp=$(date +%Y%m%d%H%M%S)
sudo cp -p /etc/fstab "/etc/fstab.archlinuxsetup.bak-$stamp"
if [[ -e /etc/crypttab ]]; then
  sudo cp -p /etc/crypttab "/etc/crypttab.archlinuxsetup.bak-$stamp"
fi
printf '%s\n' "$mapper UUID=$luks_uuid none noauto" | sudo tee -a /etc/crypttab >/dev/null
printf '%s\n' "UUID=$fs_uuid $mountpoint ext4 noauto,nofail 0 2" | sudo tee -a /etc/fstab >/dev/null
sudo mkdir -p "$mountpoint"
sudo systemctl daemon-reload
sudo mount "$mountpoint"
sudo mkdir -p "$mountpoint/Dados" "$mountpoint/Jogos"
sudo chown -R "dev:$(id -gn dev)" "$mountpoint/Dados" "$mountpoint/Jogos"
printf 'HD preparado em %s. No próximo boot, use ./install.sh storage unlock.\n' "$mountpoint"
