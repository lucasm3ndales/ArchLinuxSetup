# ArchLinuxSetup

Pós-instalação do Arch Linux para o [Acer Aspire VX5-591G-54PG](profiles/acer-aspire-vx5-591g-54pg/README.md) e outros computadores. O archinstall instala o sistema; este projeto acrescenta pacotes, configura o visual do Xfce e ativa rolagem natural no touchpad X11. As escolhas iniciais do notebook estão em [user_configuration.json](archinstall/user_configuration.json).

## Começar

Use um usuário comum com acesso a `sudo`. Depois de clonar o repositório, entre na pasta e execute uma vez:

```bash
./archsetup link
```

O comando cria `~/.local/bin/archsetup` apontando para este repositório e configura o PATH no Bash ou Zsh, com backup do arquivo anterior. **Abra um novo terminal** e confira com `archsetup help`. Mantenha o repositório no mesmo lugar; o comando instalado é um link para ele.

Antes de alterar o notebook, confirme o perfil e o layout instalado. O JSON salvo não contém a configuração de criptografia do SSD:

```bash
archsetup check --profile acer-aspire-vx5-591g-54pg
lsblk -f
archsetup doctor
```

Revise UUIDs e dados da máquina antes de compartilhar a saída. `check` é somente leitura: confirma o modelo Acer, `[multilib]`, os subvolumes Btrfs e a presença das ferramentas de snapshot.

## Comandos

| Comando | Ação |
| --- | --- |
| `archsetup help` | Mostra a ajuda. Use `archsetup help packages` para detalhes de um comando. |
| `archsetup check --profile NOME` | Checa compatibilidade e pré-requisitos sem alterar o sistema. |
| `archsetup doctor` | Diagnóstico somente leitura. |
| `archsetup paru` | Instala o auxiliar AUR após revisão do PKGBUILD. |
| `archsetup packages` | Instala pacotes das categorias selecionadas. |
| `archsetup xfce` | Aplica tema, fontes, painéis e indicador das teclas Lock. |
| `archsetup touchpad` | Configura rolagem natural para touchpads X11. |

Adicione `--dry-run` a `paru`, `packages`, `xfce` ou `touchpad` para ver o que será feito sem alterar o sistema. Por exemplo, `archsetup packages gaming --dry-run`. Sem `--dry-run`, o comando executa a ação. Os scripts não suprimem as perguntas do `pacman` ou do auxiliar AUR.

Sequência sugerida para o Acer:

```bash
archsetup check --profile acer-aspire-vx5-591g-54pg
sudo snapper list-configs
sudo snapper -c root create -d "Arch limpo - antes do ArchLinuxSetup"
sudo snapper -c home create -d "Home limpa - antes do ArchLinuxSetup"
archsetup paru --dry-run
archsetup paru
archsetup packages --profile acer-aspire-vx5-591g-54pg --dry-run
archsetup packages --profile acer-aspire-vx5-591g-54pg
archsetup xfce --dry-run
archsetup xfce
archsetup touchpad --dry-run
archsetup touchpad
```

Se você já criou os dois snapshots do sistema limpo, pule as duas linhas de criação. Confira que as configurações `root` e `home` aparecem antes de executar esses comandos. Execute `xfce` numa sessão X11 do Xfce. A regra do touchpad entra em vigor na próxima sessão X11. `paru` é necessário para as categorias que contêm pacotes AUR; `packages` também aceita `yay` se já estiver instalado.

## Pacotes

| Categoria | O que acrescenta |
| --- | --- |
| `base` | `base-devel`, Git, curl, utilitários de terminal e fonte Terminus. O perfil Acer acrescenta headers dos kernels, `snap-pac`, Btrfs Assistant e ferramentas de criptografia. |
| `development` | GCC, Clang, CMake, depuradores, Python e pip, fnm, Rust/Cargo, .NET 10, Podman, DBeaver, Zed, Kleopatra e Bruno (AUR). |
| `desktop` | Utilitários Xorg, libinput, Docklike, temas, fontes, mensageiros, qBittorrent, ALSA, portal GTK, Brave, VSCodium e Spotify (os três últimos pela AUR). O perfil Acer acrescenta Blueman. |
| `gaming` | Steam, GameMode e MangoHud. O perfil Acer acrescenta bibliotecas gráficas Intel de 32 bits. |
| `emulation` | RetroArch, Dolphin, PCSX2 e RPCS3; os dois últimos vêm da AUR. |

Sem categorias, `packages` seleciona `base development desktop`. Você também pode instalar por etapas:

```bash
archsetup packages base development --profile acer-aspire-vx5-591g-54pg
archsetup packages gaming --profile acer-aspire-vx5-591g-54pg
```

Sem `--profile`, só entram as listas compartilhadas. Nenhum perfil é escolhido automaticamente. As listas pressupõem os componentes instalados pelo archinstall conforme o JSON salvo; confira-os em outra máquina antes de configurar Xfce ou touchpad.

O script elimina nomes repetidos e usa `pacman -Syu --needed` e `paru -S --needed` (ou `yay`). Reexecutar uma categoria não duplica pacotes, mas pode atualizar o sistema. `gaming` exige `[multilib]` habilitado. O `--dry-run` indica quais pacotes da lista já estão instalados; não calcula dependências nem alterações de versão. Se `rustup` estiver instalado, avisa antes de solicitar `rust`.

Ao usar `--profile`, a aplicação confere o fabricante e o modelo antes de instalar. A prévia pode ser executada em outra máquina para revisar as listas.

`paru` instala `git` e `base-devel`, baixa o PKGBUILD da AUR, mostra seu conteúdo e só executa `makepkg -si` depois que você digitar `INSTALAR PARU`. Revise também os PKGBUILDs apresentados pelo auxiliar AUR ao instalar outros programas. Veja o [guia da ArchWiki sobre a AUR](https://wiki.archlinux.org/title/Arch_User_Repository).

Em `development`, o script instala o Node.js LTS pelo fnm somente se não houver uma versão padrão e adiciona a inicialização ao `.bashrc` ou `.zshrc` uma vez, com backup. Em `desktop`, define o Brave como navegador padrão somente numa sessão gráfica, caso ainda não seja o padrão.

## Xfce e touchpad

`xfce` exige uma sessão X11 do Xfce e os pacotes da categoria `desktop`. Faz backup da configuração em `~/.local/state/archlinuxsetup/backups/`, aplica Materia-dark, Papirus-Dark, cursor Capitaine e fonte Inter. Configura uma dock inferior com Docklike e menu do Arch; no painel superior, usa Genmon para mostrar Caps, Num e Scroll Lock. Repetir o comando cria outro backup, sem duplicar os plugins.

`touchpad` exige Xorg e libinput. Cria uma regra Xorg restrita a touchpads para rolagem natural; faz backup se uma regra anterior existir. Repetir o comando não duplica a regra. Ly e zram pertencem à instalação feita pelo archinstall e não são reconfigurados por estes comandos.

## Snapshots e recuperação

O `archinstall` selecionou Snapper e criou as configurações `root` e `home`. O perfil Acer instala [Btrfs Assistant](https://archlinux.org/packages/extra/x86_64/btrfs-assistant/) para criar, listar e inspecionar snapshots pela interface gráfica. Também instala [snap-pac](https://archlinux.org/packages/extra/any/snap-pac/), que cria snapshots antes e depois de transações do pacman. Confirme as duas configurações no notebook:

```bash
sudo snapper list-configs
sudo snapper -c root list
sudo snapper -c home list
```

Para marcar o sistema limpo **antes de qualquer instalação do projeto**, use Snapper, que já veio do archinstall. Se você já tem esses snapshots, não precisa criá-los novamente. Depois de instalar `base`, abra **Btrfs Assistant** no Xfce para criar e consultar os próximos snapshots pela interface gráfica. Os comandos para o marco inicial são:

```bash
sudo snapper -c root create -d "Arch limpo - antes do ArchLinuxSetup"
sudo snapper -c home create -d "Home limpa - antes do ArchLinuxSetup"
```

**Restauração:** o comando `snapper rollback` cria um novo subvolume e o define como padrão. No layout do archinstall, a raiz é montada explicitamente como `@`; essa troca pode não mudar a raiz usada no boot. Além disso, `/boot` é uma partição FAT32 separada com Limine e UKIs, fora do snapshot Btrfs. Uma recuperação manual a partir da ISO pode precisar montar o Btrfs no topo, restaurar `@` e alinhar os arquivos de `/boot`. Não execute `snapper rollback` presumindo que ele resolve esse layout sozinho.

A [ArchWiki documenta `limine-snapper-sync`](https://wiki.archlinux.org/title/Limine#Snapper_snapshot_integration_for_Btrfs), disponível na AUR, para criar entradas de snapshots no Limine e restaurar o sistema com versões correspondentes do kernel. É o próximo facilitador a configurar após conferir `limine.conf`, UKIs, Snapper e as montagens no notebook. Este projeto ainda não o instala nem altera o boot automaticamente. O Btrfs Assistant ajuda a gerenciar snapshots, mas não substitui essa integração para uma restauração completa.

## HD secundário

O HD de 1 TB do Acer será configurado manualmente. Este repositório **não contém comando para particionar, formatar ou desbloquear discos**. Antes de qualquer operação no notebook, confirme modelo, partições e montagens do HD por um caminho estável em `/dev/disk/by-id/`; o SSD do sistema não é alvo desse preparo.

## Organização

| Caminho | Conteúdo |
| --- | --- |
| `archsetup` | Interface de comandos e instalação do link local. |
| `modules/` | Diagnóstico, pacotes, paru e módulos do desktop. |
| `packages/official/`, `packages/aur/` | Pacotes compartilhados, um nome por linha. |
| `profiles/<identificador>/` | Dados e pacotes específicos da máquina. |
| `archinstall/` | Configuração salva do instalador; sem senha de disco. |

O [perfil Acer](profiles/acer-aspire-vx5-591g-54pg/README.md) registra o hardware conhecido e as verificações pendentes. Para adicionar outro computador, crie seu perfil e confira a seleção com `archsetup packages --profile <identificador> --dry-run`.
