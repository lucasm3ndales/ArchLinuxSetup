# Acer Aspire VX5-591G-54PG

Modelo e especificações de fábrica confirmados pelo proprietário:

| Componente | Configuração original |
| --- | --- |
| Processador | Intel Core i5-7300HQ, 7ª geração, 4 núcleos, até 3,5 GHz; GPU Intel integrada. |
| GPU dedicada | NVIDIA GeForce GTX 1050 com 4 GB GDDR5. |
| Memória | 8 GB DDR4, expansível até 32 GB. |
| Armazenamento | HD de 1 TB. |
| Tela | 15,6 polegadas Full HD (1920 × 1080). |
| Sistema original | Windows 10. |

**Alteração no aparelho atual:** SSD NVMe de 256 GB destinado ao Arch Linux. O proprietário o descreveu como “NVMe 2.0”; o modelo do SSD e a interface serão conferidos no diagnóstico. O HD de 1 TB permanece instalado e está sem uso. Será reparticionado separadamente para dados e jogos com LUKS2 + ext4 após instalar o Arch no SSD. O script exige o caminho real `/dev/disk/by-id/...` e confirmação digitada; não presume `/dev/sda` nem age sobre o HD no archinstall.

Os IDs dos dispositivos e drivers carregados serão conferidos com `./install.sh diagnose` no notebook antes de aplicar ajustes específicos.

## Pacotes já selecionados

- `base`: `intel-ucode`, `mesa`, `vulkan-intel`, `intel-media-driver` para VA-API, `btrfs-progs`, `snapper`, `snap-pac`, `btrfs-assistant`, `cryptsetup` e `e2fsprogs`. `zram-generator` e `xf86-input-libinput` agora fazem parte das listas compartilhadas.
- `gaming`: `lib32-mesa` e `lib32-vulkan-intel` para jogos de 32 bits.
- `desktop`: `bluez`, `bluez-utils` e `blueman` para Bluetooth.

## Hardware a verificar antes de automatizar

| Área | Pacotes ou configuração candidatos | Verificação necessária |
| --- | --- | --- |
| NVIDIA Pascal | `nvidia-580xx-dkms` (AUR), utilitários da mesma série, `nvidia-prime`, `dkms` e headers do kernel instalado; biblioteca NVIDIA de 32 bits para Steam | Confirmar ID PCI da GTX 1050, kernel, Secure Boot e conflitos com driver atual. Usar Intel no desktop e PRIME nos jogos. |
| Wi-Fi | Firmware compatível com o chipset detectado; se for Intel, `linux-firmware-intel` | Conferir `lspci -nnk` e firmware carregado. Unidades da série podem ter placas de rede diferentes. |
| Bluetooth | `bluez`, `bluez-utils` e serviço `bluetooth.service` | Confirmar o adaptador em `lsusb`/`rfkill` e testar áudio/periféricos. |
| Áudio | `pipewire`, `pipewire-audio`, `pipewire-pulse`, `pipewire-alsa`, `wireplumber`, `alsa-utils` | Conferir controlador, microfone, alto-falantes, fone e HDMI. Só adicionar `sof-firmware` se o dispositivo exigir. |
| Touchpad | `xf86-input-libinput` no desktop compartilhado; rolagem natural por regra Xorg do módulo `system` | Conferir se o dispositivo aparece em `/proc/bus/input/devices` e `xinput`. Testar Fn+F7 caso esteja desligado. |
| Energia e teclas | `xfce4-power-manager`, suporte ACPI/WMI do kernel, eventual controle de brilho | Testar suspensão, tampa, bateria, brilho, teclado iluminado e retorno do suspend antes de ajustar serviços ou parâmetros. |
| SSD NVMe | Driver NVMe do kernel; `smartmontools` e `nvme-cli` só para monitoramento | Conferir modelo, saúde e sistema de arquivos. Não requer driver Acer. |

O módulo `acer_wmi` faz parte do kernel Linux e pode oferecer teclas especiais e controles Acer. Ele não é um pacote de driver proprietário da Acer. As páginas de suporte Acer para a série VX5-591G apresentam downloads voltados ao Windows; não há um pacote Acer para Linux que deva ser instalado automaticamente. Evite parâmetros de kernel, BIOS ou controle manual de ventoinhas sem um problema reproduzido neste aparelho.

O driver NVIDIA principal 590+ deixou de atender Pascal. A troca pelo ramo proprietário legado requer uma etapa separada e revisável porque mexe no módulo do kernel e pode afetar o login gráfico. Para DKMS, instale os headers correspondentes a **cada kernel** usado (`linux-headers`, `linux-lts-headers` etc.). A categoria de pacotes atual não instala nem substitui a NVIDIA.

## Instalação e primeiro boot

Use archinstall no SSD NVMe com LUKS, Btrfs `compress=zstd`, Snapper, Limine, espelhos brasileiros, timezone `America/Sao_Paulo`, locale `en_US.UTF-8`, teclado ABNT2, hostname `dev-machine`, PipeWire e NetworkManager. Crie `dev` com acesso a `sudo`; `root` permanece como conta administrativa. A fonte de console desejada é `ter-v16n` (Terminus 16). Confira o mapa ABNT2 e a configuração da fonte após instalar o pacote `terminus-font`.

Após o boot, confira os subvolumes Btrfs, a montagem de `/.snapshots`, `snapper list-configs`, zram e os serviços com `./install.sh diagnose`. O `system apply` habilita Ly no tty1 e cria zram com zstd. Ele oculta as sessões shell/xinitrc/Wayland do Ly e mantém tty2 livre. O Limine e a integração de snapshots inicializáveis precisam de validação separada.

Para o HD, execute `./install.sh storage plan` no notebook, identifique o dispositivo `by-id` e só então use `storage apply --profile acer-aspire-vx5-591g-54pg --disk ...`. Ele apaga o HD, pede senha LUKS, cria ext4 e monta em `/mnt/dados`; boots seguintes usam `storage unlock --profile acer-aspire-vx5-591g-54pg`. O SSD não é tocado pelo módulo de armazenamento.

## Referências

- [Acer: suporte ao VX5-591G](https://www.acer.com/pl-pl/support/product-support/VX5-591G/downloads)
- [Acer Community: identificação do VX5-591G-54PG e i5-7300HQ](https://community.acer.com/pt/discussion/553751/notebook-acer-aspire-vx15-e-muito-lento-quero-uma-solucao)
- [ArchWiki: compatibilidade da série VX5-591G](https://wiki.archlinux.org/title/Laptop/Acer)
- [Arch Linux: mudança de suporte NVIDIA Pascal](https://archlinux.org/news/nvidia-590-driver-drops-pascal-support-main-packages-switch-to-open-kernel-modules/)
- [ArchWiki: DKMS](https://wiki.archlinux.org/title/Dynamic_Kernel_Module_Support)
- [Arch Linux Forums: touchpad VX5-591G e Fn+F7](https://bbs.archlinux.org/viewtopic.php?id=244524)
