# Acer Aspire VX5-591G

Este perfil cobre a família Aspire VX5-591G, sem depender do sufixo usado em uma variante regional. O ArchSetup compara `Acer` com o fabricante informado pelo DMI e `VX5-591G` com o nome do modelo. O perfil só é aplicado quando você passa `--profile acer-aspire-vx5-591g`.

```bash
archsetup doctor --profile acer-aspire-vx5-591g
archsetup pkg --profile acer-aspire-vx5-591g --dry-run
archsetup desktop --profile acer-aspire-vx5-591g --dry-run
```

`doctor` pode sugerir o perfil automaticamente, mas não o seleciona. Confira a prévia antes da instalação. O comando `desktop` exige Xfce instalado e uma sessão X11 do Xfce ativa.

## O que o perfil acrescenta

| Categoria | Pacotes adicionais |
| --- | --- |
| `desktop` | `blueman` |
| `gaming` | `lib32-mesa`, `lib32-vulkan-intel`, `nvidia-580xx-dkms`, `nvidia-580xx-utils`, `lib32-nvidia-580xx-utils` |

Ao executar `archsetup desktop --profile acer-aspire-vx5-591g`, o perfil ativa toque para clicar no touchpad. A rolagem natural é uma configuração comum do comando `desktop` e também funciona sem perfil.

Os pacotes e o ajuste estão declarados em [profile.toml](profile.toml). `pkg gaming` seleciona headers de acordo com os kernels instalados porque inclui o driver NVIDIA DKMS; `pkg base` sozinho não os instala. Para `linux` e `linux-zen`, serão selecionados `linux-headers` e `linux-zen-headers`. A categoria `gaming` exige `[multilib]` habilitado. `blueman` oferece interface gráfica para o Bluetooth já ativado no sistema; os pacotes Intel e NVIDIA de 32 bits atendem jogos que usam uma ou outra GPU.

## Antes de usar

O identificador DMI cobre o modelo base, mas as variantes podem ter componentes diferentes. **Use `pkg gaming` com este perfil apenas se o equipamento tiver GPU Intel e NVIDIA GTX 10xx/Pascal**, como a GTX 1050. Confirme com `lspci -nn | grep -Ei 'VGA|3D'`. O ArchSetup recusa drivers oficiais NVIDIA incompatíveis já instalados e não os remove automaticamente. O ramo 580xx é distribuído pela AUR; confira o PKGBUILD e a compilação DKMS antes de reiniciar. O perfil não configura PRIME, boot, criptografia, energia ou snapshots. Para usar a GPU dedicada sob X11, consulte a documentação PRIME do Arch após instalar o driver.

O perfil não reinstala nem substitui Btrfs, LUKS, Snapper, zram, systemd-boot, NetworkManager, PipeWire, serviços de Bluetooth, impressão, energia ou firewall escolhidos na instalação do sistema. Pacotes já presentes são ignorados com `--needed`. `desktop` configura o Xfce e o touchpad, preservando o Ly quando ele é usado.
