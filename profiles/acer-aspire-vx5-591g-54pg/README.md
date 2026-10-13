# Acer Aspire VX5-591G-54PG

O proprietário confirmou o modelo, CPU Intel Core i5-7300HQ, GTX 1050, 8 GB de RAM, SSD NVMe de 256 GB adicionado e HD de 1 TB. IDs dos dispositivos, modelos dos discos e drivers carregados ainda precisam ser conferidos com `archsetup doctor` no notebook. Não registre número de série ou SNID.

## Instalação do Arch

O SSD é alvo do archinstall. Para a nova instalação, o plano é LUKS, Btrfs com subvolumes `@` e `@home`, Timeshift e GRUB2. Mantenha **Install to removable location** ativado por causa do loop de boot observado no notebook. UKI desativado simplifica o uso de GRUB e snapshots, mas essa combinação ainda não foi testada no Acer; UKI ativado reproduz a combinação que iniciou anteriormente. Confira o ponto de montagem EFI no resumo do instalador e leia as limitações em [Recuperação do sistema](#recuperação-do-sistema). O repositório não guarda uma configuração do instalador. O HD de 1 TB fica fora do archinstall e será preparado manualmente.

| Opção | Valor escolhido |
| --- | --- |
| Hostname e usuário | `dev-machine`; usuário `dev` com acesso a `sudo` |
| Localização | Espelhos brasileiros; `America/Sao_Paulo`; `en_US.UTF-8`; teclado ABNT2 |
| Rede e áudio | NetworkManager; PipeWire |
| Interface | Xfce X11; LightDM com Slick Greeter selecionados no archinstall. `archsetup desktop --profile acer-aspire-vx5-591g-54pg` aplica o visual e os ajustes do touchpad após o primeiro login. |
| Console | Conferir a fonte escolhida no primeiro boot. O ArchSetup não configura a fonte do console. |

Depois do primeiro boot, confira com `lsblk -f`, `findmnt /`, `findmnt /boot` e `archsetup check --profile acer-aspire-vx5-591g-54pg` o sistema de arquivos, as montagens e as ferramentas instaladas. Verifique também zram, GRUB, Timeshift e os kernels escolhidos. Os comandos deste projeto não configuram boot, criptografia ou recuperação; consulte [Recuperação do sistema](#recuperação-do-sistema).

## Recuperação do sistema

Se a partição EFI estiver montada em `/boot`, kernel, initramfs e eventuais UKIs ficarão fora dos snapshots Btrfs. Uma restauração após atualizar o kernel pode exigir recuperar também os arquivos de boot correspondentes. O archinstall atual pode preservar um initramfs tradicional mesmo com UKI em certas instalações com GRUB e snapshots Btrfs; confira o resultado na versão instalada antes de confiar nas entradas do `grub-btrfs`. Não mova `/boot` para dentro do Btrfs criptografado sem planejar o desbloqueio: GRUB comum não lê LUKS2 com Argon2 nesse layout. [Timeshift: layouts suportados](https://github.com/linuxmint/timeshift#supported-system-configurations), [ArchWiki: `/boot` criptografado](https://wiki.archlinux.org/title/GRUB#Encrypted_/boot).

Antes de executar `archsetup packages`, crie no Timeshift um snapshot manual do sistema limpo. Teste a restauração pela interface no notebook antes de depender dela; iniciar um snapshot pelo GRUB não equivale a restaurá-lo definitivamente. Mantenha backup separado dos arquivos pessoais em `/home`.

## Pacotes do perfil

| Categoria | Pacotes específicos |
| --- | --- |
| `base` | Headers de `linux` e `linux-zen`, `cryptsetup` e `e2fsprogs`. |
| `desktop` | `blueman`. |
| `gaming` | `lib32-mesa` e `lib32-vulkan-intel`. |

`xf86-input-libinput` pertence à lista compartilhada. O projeto pressupõe que o archinstall tenha instalado os componentes escolhidos para o sistema, como microcódigo Intel, Mesa/Vulkan Intel, driver Intel VA-API, Btrfs, Bluetooth, LightDM com Slick Greeter e zram; confira o resultado no primeiro boot. O perfil ainda não instala driver NVIDIA.

## Pendências de hardware

| Área | Conferir antes de automatizar |
| --- | --- |
| NVIDIA | ID PCI, kernels, headers, Secure Boot e driver atual. Escolher um ramo compatível com Pascal e testar PRIME e bibliotecas de 32 bits no notebook. |
| Wi-Fi/Bluetooth | Chipset, firmware, `rfkill`, áudio e periféricos. Confirmar o estado de `bluetooth.service` no notebook. |
| Áudio | Microfone, alto-falantes, fone e saída HDMI. Instalar firmware adicional só se o dispositivo exigir. |
| Touchpad | Detecção pelo libinput, Fn+F7, toque para clicar, rolagem com dois dedos e rejeição da palma. Os ajustes compartilhados e o toque para clicar vêm de `archsetup desktop --profile acer-aspire-vx5-591g-54pg`. |
| Energia e teclas | Suspensão, tampa, bateria, brilho, iluminação do teclado e retorno do suspend. |
| SSD e Timeshift | Modelo e saúde do SSD; subvolumes, montagens, retenção de snapshots e teste da restauração com GRUB. |

O módulo `acer_wmi` já faz parte do kernel. Não adicione parâmetros de kernel ou controle manual de ventoinhas sem um problema reproduzido no aparelho. Os headers para os dois kernels estão na categoria `base` para uma futura configuração DKMS.

## HD secundário

O HD será preparado manualmente como uma partição LUKS2 com ext4 para dados e jogos. Antes de formatar, confirme o modelo, as partições e as montagens por um caminho `/dev/disk/by-id/...` no notebook. O repositório não automatiza essa operação nem define uma montagem persistente.
