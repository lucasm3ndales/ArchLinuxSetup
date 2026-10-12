# ArchLinuxSetup

Scripts de pós-instalação do Arch Linux. As listas de pacotes são compartilhadas; o perfil adiciona apenas o que depende do computador. Cada etapa tem um comando `plan` para revisão e um `apply` para execução.

O projeto **não instala o Arch no SSD**. Essa parte é feita pelo archinstall. Também não configura o boot, o driver NVIDIA, a recuperação pelo Snapper ou ajustes de energia.

## Antes de começar

Use o sistema Arch já instalado, com um usuário comum que tenha acesso a `sudo`. Execute os comandos na raiz deste repositório, sem `sudo` antes de `./install.sh`.

No primeiro boot do Acer, confira a instalação antes de aplicar os scripts:

```bash
lsblk -f
sudo snapper list-configs
./install.sh diagnose
```

Confirme que `/` está em Btrfs sobre LUKS, que as configurações do Snapper correspondem aos subvolumes instalados e que as entradas do Limine iniciam os kernels escolhidos. O arquivo [user_configuration.json](archinstall/user_configuration.json) registra a seleção do archinstall, mas **não registra a criptografia do SSD**. O diagnóstico mostra dispositivos e montagens; revise a saída antes de compartilhá-la.

Se `snapper list-configs` mostrar `root` e `home`, você pode marcar o estado inicial antes de clonar o projeto:

```bash
sudo snapper -c root create -d "Marco zero - Arch limpo" -p
sudo snapper -c home create -d "Marco zero - Home limpa" -p
```

Os números são independentes. Esses snapshots não cobrem `/boot`, `/var/log` ou `/var/cache/pacman/pkg`. A restauração completa exige conferir a montagem do Btrfs e o boot pelo Limine; o projeto ainda não automatiza esse procedimento.

## Sequência de uso

```bash
./install.sh aur plan
./install.sh aur apply

./install.sh packages plan --profile acer-aspire-vx5-591g-54pg
./install.sh packages apply --profile acer-aspire-vx5-591g-54pg

./install.sh system plan
./install.sh system apply

./install.sh xfce plan
./install.sh xfce apply
```

`aur apply` prepara o `paru` para as categorias com pacotes AUR. Se `paru` ou `yay` já estiver instalado, `packages apply` pode usá-lo. Execute `system apply` após instalar `desktop`; execute `xfce apply` dentro de uma sessão X11 do Xfce. O HD secundário tem um procedimento separado em [Armazenamento](#armazenamento-do-acer).

| Comando | O que faz |
| --- | --- |
| `diagnose` | Mostra informações do sistema; não altera nada. |
| `aur plan/apply` | Planeja ou instala o `paru`. |
| `packages plan/apply` | Mostra ou instala os pacotes selecionados. |
| `system plan/apply` | Configura Ly, zram e rolagem natural do touchpad no Xorg. |
| `xfce plan/apply` | Configura tema, fontes, painéis e indicador das teclas Lock. |
| `storage plan/apply/unlock` | Inspeciona, prepara ou desbloqueia o HD secundário do Acer. |

`plan` não altera o sistema. `packages plan` mostra as listas, mas não calcula dependências nem verifica quais pacotes já estão instalados. Se um `apply` falhar no meio, confira a saída antes de repetir o comando.

## Pacotes

| Categoria | Principais pacotes |
| --- | --- |
| `base` | `base-devel`, Git, curl, wget, utilitários de terminal, firmware, fonte Terminus e `zram-generator`. O perfil Acer acrescenta microcódigo Intel, headers dos kernels, Btrfs, Snapper e ferramentas de criptografia. |
| `development` | GCC, Clang, CMake, depuradores, Python e pip, fnm, Rust/Cargo, .NET 10, Podman, DBeaver, Zed, Kleopatra e Bruno (AUR). |
| `desktop` | Xfce/Xorg, Ly, libinput, PipeWire, fontes e temas, mensageiros, qBittorrent, `xdg-utils`, Brave, VSCodium e Spotify (os três últimos pela AUR). O perfil Acer acrescenta Bluetooth. |
| `gaming` | Steam, GameMode, MangoHud e fontes. O perfil Acer acrescenta bibliotecas gráficas Intel de 32 bits. |
| `emulation` | RetroArch, Dolphin, PCSX2 e RPCS3; os dois últimos vêm da AUR. |

Sem categorias explícitas, `packages` usa `base development desktop`. Ao indicar uma ou mais categorias, instala somente as selecionadas. Sem `--profile`, usa somente as listas compartilhadas. O perfil Acer nunca é escolhido automaticamente.

Você pode instalar em etapas:

```bash
./install.sh packages apply --profile acer-aspire-vx5-591g-54pg base development
./install.sh packages apply --profile acer-aspire-vx5-591g-54pg gaming
```

O script elimina nomes repetidos e usa `pacman -Syu --needed` e `paru -S --needed` (ou `yay`). Repetir uma categoria não duplica pacotes, mas `-Syu` pode atualizar o sistema. `gaming` exige `[multilib]` habilitado. Os pacotes oficiais são instalados antes dos pacotes AUR; o gerenciador resolve as dependências, independentemente da ordem das linhas nas listas.

Em `development`, o script instala o Node.js LTS pelo fnm apenas quando não há versão padrão. Ele acrescenta a inicialização ao `.bashrc` ou `.zshrc` uma vez, após fazer backup do arquivo existente. Em `desktop`, define o Brave como navegador padrão somente numa sessão gráfica e se ainda não for o padrão.

### AUR

Numa instalação nova, `aur apply` instala `git` e `base-devel` pelos repositórios oficiais, baixa o [PKGBUILD do paru](https://aur.archlinux.org/packages/paru) e o mostra para revisão. Só executa `makepkg -si` como usuário comum depois que você digitar `INSTALAR PARU`. Revise também os PKGBUILDs apresentados pelo auxiliar AUR ao instalar ou atualizar outros pacotes. O [guia da ArchWiki](https://wiki.archlinux.org/title/Arch_User_Repository) explica essa revisão.

## Sistema e Xfce

`system apply` exige Ly, Xfce X11, libinput e zram-generator instalados. Faz backup de `/etc/ly/config.ini` antes de editá-lo, habilita Ly no tty1 e mantém tty2 livre. Também configura zram com zstd para metade da RAM e cria uma regra Xorg restrita a touchpads para rolagem natural. A zram entra em vigor no próximo boot; a regra do touchpad, na próxima sessão X11. Se outro gerenciador de login estiver habilitado, o comando para antes de alterar arquivos.

`xfce apply` exige uma sessão X11 do Xfce. Faz backup em `~/.local/state/archlinuxsetup/backups/`, aplica Materia-dark, Papirus-Dark, cursor Capitaine e fonte Inter. Configura a dock inferior com Docklike e menu do Arch; no painel superior, usa Genmon para indicar Caps, Num e Scroll Lock. Repetir o comando cria outro backup, sem duplicar esses plugins.

Após reiniciar, confira `zramctl`, `swapon --show` e o touchpad. Verifique também `systemctl is-enabled NetworkManager`; se necessário, habilite-o com `sudo systemctl enable --now NetworkManager`. O módulo `system` não altera Limine, LUKS, partições ou Snapper.

## Armazenamento do Acer

`storage apply` apaga **somente o disco informado**. O destino previsto é o HD secundário de 1 TB, preparado como LUKS2 com ext4 e montado em `/mnt/dados`. O SSD do sistema fica fora desse comando.

No notebook, identifique o HD pelo modelo e por um caminho estável `/dev/disk/by-id/...`:

```bash
./install.sh storage plan
ls -l /dev/disk/by-id/
./install.sh storage apply --profile acer-aspire-vx5-591g-54pg --disk /dev/disk/by-id/ID_REAL_DO_HD
```

Substitua `ID_REAL_DO_HD` pelo identificador visto no notebook. Antes de formatar, o script confirma o modelo Acer, tipo e capacidade do disco, partições e montagens, mostra o destino e exige que você digite `APAGAR /dev/...`. Ele recusa um HD já preparado. A formatação elimina os arquivos existentes; não faz apagamento seguro dos blocos antigos.

O comando cria `/mnt/dados/Dados` e `/mnt/dados/Jogos`, salva backups de `/etc/crypttab` e `/etc/fstab` antes de editá-los e pede uma senha LUKS própria para o HD. Nos boots seguintes, desbloqueie-o manualmente:

```bash
./install.sh storage unlock --profile acer-aspire-vx5-591g-54pg
```

## Organização

| Caminho | Conteúdo |
| --- | --- |
| `install.sh` | Interface de comandos. |
| `modules/` | Implementação das etapas. |
| `packages/official/`, `packages/aur/` | Pacotes compartilhados, um nome por linha. |
| `profiles/<identificador>/` | Dados e pacotes específicos da máquina. |
| `archinstall/` | Configuração salva do instalador; não contém senha de disco. |

O [perfil Acer](profiles/acer-aspire-vx5-591g-54pg/README.md) registra o hardware conhecido, as escolhas do archinstall e o que ainda precisa de teste no notebook. Para adicionar outro computador, crie seu perfil, coloque apenas os pacotes próprios dele nas mesmas categorias e confira a seleção com `./install.sh packages plan --profile <identificador>`.
