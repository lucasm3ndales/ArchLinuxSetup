# Acer Aspire VX5-591G-54PG

O proprietário confirmou o modelo, CPU Intel Core i5-7300HQ, GTX 1050, 8 GB de RAM, SSD NVMe de 256 GB adicionado e HD de 1 TB. IDs dos dispositivos, modelos dos discos e drivers carregados ainda precisam ser conferidos com `archsetup doctor` no notebook. Não registre número de série ou SNID.

## Instalação do Arch

O SSD é alvo do archinstall. A configuração instalada foi planejada com LUKS, Btrfs com `compress=zstd`, Snapper, Limine e kernels `linux` e `linux-zen`. O HD de 1 TB fica fora do archinstall e será preparado manualmente.

| Opção | Valor escolhido |
| --- | --- |
| Hostname e usuário | `dev-machine`; usuário `dev` com acesso a `sudo` |
| Localização | Espelhos brasileiros; `America/Sao_Paulo`; `en_US.UTF-8`; teclado ABNT2 |
| Rede e áudio | NetworkManager; PipeWire |
| Interface | Xfce X11; Ly instalado e configurado pelo archinstall |
| Console | O JSON seleciona `Lat2-Terminus16`; conferir no primeiro boot. A categoria `base` inclui `terminus-font` para eventual uso de `ter-v16n`. |

O [JSON salvo](../../archinstall/user_configuration.json) registra escolhas do instalador, mas não a senha nem a configuração de criptografia do SSD. Depois do primeiro boot, confirme LUKS/Btrfs com `lsblk -f`, os subvolumes e configurações do Snapper, zram e as entradas do Limine para os dois kernels. Rode `archsetup check --profile acer-aspire-vx5-591g-54pg`. Os comandos deste projeto não configuram o boot nem o desbloqueio do SSD. A restauração com Limine e `/boot` separado ainda não foi validada; consulte a seção de recuperação do [README principal](../../README.md#snapshots-e-recuperação).

## Pacotes do perfil

| Categoria | Pacotes específicos |
| --- | --- |
| `base` | Headers de `linux` e `linux-zen`, `snap-pac`, Btrfs Assistant, `cryptsetup` e `e2fsprogs`. |
| `desktop` | `blueman`. |
| `gaming` | `lib32-mesa` e `lib32-vulkan-intel`. |

`xf86-input-libinput` pertence à lista compartilhada. O archinstall já selecionou microcódigo Intel, Mesa/Vulkan Intel, driver Intel VA-API, Btrfs, Snapper, Bluetooth, Ly e zram; os scripts não reinstalam esses pacotes nem reconfiguram Ly ou zram. O perfil ainda não instala driver NVIDIA.

## Pendências de hardware

| Área | Conferir antes de automatizar |
| --- | --- |
| NVIDIA | ID PCI, kernels, headers, Secure Boot e driver atual. Escolher um ramo compatível com Pascal e testar PRIME e bibliotecas de 32 bits no notebook. |
| Wi-Fi/Bluetooth | Chipset, firmware, `rfkill`, áudio e periféricos. Confirmar o estado de `bluetooth.service` no notebook. |
| Áudio | Microfone, alto-falantes, fone e saída HDMI. Instalar firmware adicional só se o dispositivo exigir. |
| Touchpad | Detecção pelo libinput, Fn+F7, toque para clicar, rolagem com dois dedos e rejeição da palma. A rolagem natural compartilhada vem de `archsetup touchpad`. |
| Energia e teclas | Suspensão, tampa, bateria, brilho, iluminação do teclado e retorno do suspend. |
| SSD e Snapper | Modelo e saúde do SSD; subvolumes, montagens, retenção de snapshots e integração de `/boot`/Limine. |

O módulo `acer_wmi` já faz parte do kernel. Não adicione parâmetros de kernel ou controle manual de ventoinhas sem um problema reproduzido no aparelho. Os headers para os dois kernels estão na categoria `base` para uma futura configuração DKMS.

## HD secundário

O HD será preparado manualmente como uma partição LUKS2 com ext4 para dados e jogos. Antes de formatar, confirme o modelo, as partições e as montagens por um caminho `/dev/disk/by-id/...` no notebook. O repositório não automatiza essa operação nem define uma montagem persistente.
