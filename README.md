# ArchSetup

Pós-instalação do Arch Linux para o [Acer Aspire VX5-591G-54PG](profiles/acer-aspire-vx5-591g-54pg/README.md) e outros computadores. O archinstall instala o sistema; este projeto acrescenta pacotes, configura o visual do Xfce e ajusta o touchpad X11. As configurações do instalador são feitas no notebook e não ficam neste repositório.

## Começar

Use um usuário comum com acesso a `sudo`. Depois de clonar o repositório, entre na pasta e execute uma vez:

```bash
./archsetup link
```

O comando cria `~/.local/bin/archsetup` apontando para este repositório e configura o PATH no Bash ou Zsh, com backup do arquivo anterior. **Abra um novo terminal** e confira com `archsetup help`. Mantenha o repositório no mesmo lugar; o comando instalado é um link para ele.

Antes de alterar o notebook, confirme o perfil e o layout instalado:

```bash
archsetup check --profile acer-aspire-vx5-591g-54pg
lsblk -f
archsetup doctor
```

Revise UUIDs e dados da máquina antes de compartilhar a saída. `check` é somente leitura: confirma o modelo Acer, `[multilib]`, os subvolumes Btrfs e a presença de Timeshift e GRUB.

## Comandos

| Comando | Ação |
| --- | --- |
| `archsetup help` | Mostra a ajuda. Use `archsetup help packages` para detalhes de um comando. |
| `archsetup check --profile NOME` | Checa compatibilidade e pré-requisitos sem alterar o sistema. |
| `archsetup doctor` | Diagnóstico somente leitura. |
| `archsetup paru` | Instala o auxiliar AUR após revisão do PKGBUILD. |
| `archsetup packages` | Instala pacotes das categorias selecionadas. |
| `archsetup xfce` | Aplica tema, fontes, painéis, indicador das teclas Lock e atalho Super. |
| `archsetup greeter` | Aplica Nordic à tela de login do LightDM com Slick Greeter. |
| `archsetup touchpad` | Configura rolagem natural; com o perfil Acer, também ativa toque para clicar. |

Adicione `--dry-run` a `paru`, `packages`, `xfce`, `greeter` ou `touchpad` para ver o que será feito sem alterar o sistema. Por exemplo, `archsetup packages gaming --dry-run`. Sem `--dry-run`, o comando executa a ação. Os scripts não suprimem as perguntas do `pacman` ou do auxiliar AUR.

Sequência sugerida para o Acer:

```bash
archsetup check --profile acer-aspire-vx5-591g-54pg
archsetup paru --dry-run
archsetup paru
archsetup packages --profile acer-aspire-vx5-591g-54pg --dry-run
archsetup packages --profile acer-aspire-vx5-591g-54pg
archsetup xfce --dry-run
archsetup xfce
archsetup greeter --dry-run
archsetup greeter
archsetup touchpad --profile acer-aspire-vx5-591g-54pg --dry-run
archsetup touchpad --profile acer-aspire-vx5-591g-54pg
```

Antes de executar `paru` ou `packages`, crie um snapshot manual chamado **Arch limpo — antes do ArchSetup** no Timeshift, conforme a seção [Snapshots e recuperação](#snapshots-e-recuperação). Execute `xfce` numa sessão X11 do Xfce. A regra do touchpad entra em vigor na próxima sessão X11. `paru` é necessário para as categorias que contêm pacotes AUR; `packages` também aceita `yay` se já estiver instalado.

## Pacotes

| Categoria | O que acrescenta |
| --- | --- |
| `base` | `base-devel`, Git, curl, utilitários de terminal e fonte Terminus. O perfil Acer acrescenta headers dos kernels e ferramentas de criptografia e ext4 para o HD secundário. |
| `development` | GCC, Clang, CMake, depuradores, Rust/Cargo gerenciados por rustup, Docker Engine, Docker Compose, DBeaver e Kleopatra. |
| `desktop` | Utilitários Xorg e libinput, fontes Inter e Fira Code, Nordic (escuro, darker, bluish e Polar claro), ícones e cursor Nordzy, mensageiros, qBittorrent, ALSA, portal GTK, Brave, Spotify, ChatGPT Desktop e `ksuperkey`. Temas, Nordzy e os quatro últimos vêm da AUR. O perfil Acer acrescenta Blueman. |
| `gaming` | Steam, GameMode e MangoHud. O perfil Acer acrescenta bibliotecas gráficas Intel de 32 bits. |
| `emulation` | RetroArch, Dolphin, PCSX2 e RPCS3; os dois últimos vêm da AUR. |

Sem categorias, `packages` seleciona `base development desktop`. Você também pode instalar por etapas:

```bash
archsetup packages base development --profile acer-aspire-vx5-591g-54pg
archsetup packages gaming --profile acer-aspire-vx5-591g-54pg
```

Sem `--profile`, só entram as listas compartilhadas. Nenhum perfil é escolhido automaticamente. Confira os componentes realmente instalados pelo archinstall antes de aplicar os comandos: este repositório não instala boot, Timeshift, LightDM, Slick Greeter ou zram.

O script elimina nomes repetidos e usa `pacman -Syu --needed` e `paru -S --needed` (ou `yay`). Reexecutar uma categoria não duplica pacotes, mas pode atualizar o sistema. `gaming` exige `[multilib]` habilitado. O `--dry-run` indica quais pacotes da lista já estão instalados; não calcula dependências nem alterações de versão. Se `rust` já estiver instalado, a aplicação de `development` para antes da instalação; confira seus dependentes antes de trocá-lo por `rustup`.

Ao usar `--profile`, a aplicação confere o fabricante e o modelo antes de instalar. A prévia pode ser executada em outra máquina para revisar as listas.

`paru` instala `git` e `base-devel`, baixa o PKGBUILD da AUR, mostra seu conteúdo e só executa `makepkg -si` depois que você digitar `INSTALAR PARU`. Revise também os PKGBUILDs apresentados pelo auxiliar AUR ao instalar outros programas. Veja o [guia da ArchWiki sobre a AUR](https://wiki.archlinux.org/title/Arch_User_Repository).

Em `development`, o script instala a toolchain estável do Rust pelo rustup somente se não houver uma padrão. Docker Engine e Compose são instalados pelos repositórios oficiais; Docker Desktop não faz parte da lista. O script adiciona o usuário atual ao grupo `docker` se necessário, inicia `docker.service` e o habilita para iniciar no boot. Saia e entre novamente para ativar a associação ao grupo; depois use `docker compose up`. Membros do grupo `docker` têm privilégios equivalentes aos de root. Em `desktop`, define o Brave como navegador padrão somente numa sessão gráfica, caso ainda não seja o padrão.

## Xfce e touchpad

`xfce` exige uma sessão X11 do Xfce e os pacotes da categoria `desktop`. Faz backup da configuração em `~/.local/state/archlinuxsetup/backups/`, aplica Nordic escuro nas janelas e aplicativos, ícones Nordzy-dark, cursor Nordzy-cursors e fonte Inter. Fira Code fica instalada para editores de código. Outras variantes Nordic, inclusive Nordic-Polar claro, ficam instaladas para escolha manual. Usa um único painel superior de 48 px com botão do menu exibindo o logo do Arch, campo Verve para abrir aplicativos e comandos, lista de tarefas do Xfce e espaços de trabalho `1`, `2`, `3` à direita. A lista agrupa janelas do mesmo aplicativo e mostra apenas as do espaço atual. O Xfce atual oferece agrupamento ligado ou desligado; quando a barra lota, as tarefas excedentes vão para um menu. O relógio no canto direito mostra dia/mês e hora. Reaproveita o Whisker Menu e o relógio existentes, se houver, e remove botões de menu duplicados e os outros painéis, inclusive a dock inferior. Mostra Caps, Num e Scroll Lock com símbolos e dicas ao passar o mouse. Também associa `Alt+F1` ao menu e inicia `ksuperkey`, que faz um toque na tecla Super esquerda abrir esse menu sem impedir os atalhos com Super. A inicialização automática é criada uma vez. Repetir o comando cria outro backup, sem duplicar os plugins.

`greeter` exige LightDM habilitado, Slick Greeter instalado e os temas da categoria `desktop`. Configura a tela de login com Nordic, Nordzy-dark, Nordzy-cursors, Inter, fundo escuro e logo do Arch. Salva backup dos arquivos que alterar em `/etc/lightdm/`, preserva outras opções existentes do Slick e não reinicia o gerenciador de login. Repetir o comando sem mudanças não cria novos backups. A aparência aparece no próximo login. [Opções do Slick Greeter](https://github.com/linuxmint/slick-greeter#configuration).

`touchpad` exige Xorg e libinput. Cria uma regra Xorg restrita a touchpads para rolagem natural. Com `--profile acer-aspire-vx5-591g-54pg`, valida o modelo e cria uma segunda regra para **toque para clicar**. Faz backup antes de substituir suas próprias regras; repetir o comando não duplica arquivos. As mudanças valem na próxima sessão X11. Se o Xfce continuar desativando o toque, confira **Configurações → Mouse e touchpad → Touchpad → Toque para clicar**, pois a preferência da sessão pode prevalecer. Os comandos não reconfiguram zram.

## Snapshots e recuperação

Para a nova instalação do Acer, a escolha é **Timeshift em modo Btrfs com GRUB2**. No archinstall, selecione Timeshift na opção de snapshots Btrfs e use `@` para `/` e `@home` para `/home`. O proprietário relata que uma instalação anterior entrou em loop de boot sem a combinação de **Unified Kernel Images (UKI)** e **Install to removable location**; não foi testado qual opção resolveu o problema. Mantenha **Install to removable location** ativado: ela instala o GRUB no caminho EFI de fallback do próprio disco, não no pendrive. Para simplificar GRUB e snapshots, **UKI desativado** é a escolha proposta para esta instalação, mas essa combinação ainda precisa ser validada no Acer. Se a prioridade for repetir exatamente o boot já testado, mantenha UKI ativado. O archinstall [instala Timeshift e configura `grub-btrfs` quando GRUB é escolhido](https://github.com/archlinux/archinstall/blob/master/archinstall/lib/installer.py), mas confira o resultado no primeiro boot. Este projeto não instala nem configura essas ferramentas.

O SSD será instalado com LUKS e Btrfs. Confira o ponto de montagem da partição EFI no resumo do archinstall antes de iniciar: com EFI em `/boot`, kernel, initramfs e UKIs ficam fora do snapshot Btrfs. Uma restauração após atualizar o kernel pode exigir recuperar os arquivos de boot correspondentes; não presuma rollback completo só pelo snapshot. O archinstall atual preserva um initramfs tradicional em certas instalações com UKI, GRUB e snapshots Btrfs, mas confirme isso na versão da ISO usada e teste as entradas do GRUB antes de depender delas. Não mova `/boot` para dentro do Btrfs criptografado sem planejar como o GRUB o desbloqueará: GRUB comum não lê LUKS2 com Argon2 nesse caso. [Timeshift: layouts suportados](https://github.com/linuxmint/timeshift#supported-system-configurations), [ArchWiki: `/boot` criptografado](https://wiki.archlinux.org/title/GRUB#Encrypted_/boot).

No primeiro boot, confirme com `archsetup check --profile acer-aspire-vx5-591g-54pg` e `archsetup doctor` que as montagens e os pacotes correspondem ao plano. Abra **Timeshift**, selecione o modo **Btrfs**, escolha o disco do sistema e crie um snapshot manual com o comentário **Arch limpo — antes do ArchSetup**, antes de executar `archsetup paru` ou `archsetup packages`. O Timeshift protege o sistema; mantenha cópias separadas dos seus arquivos pessoais em `/home`. Para restaurar, selecione o snapshot na interface e use **Restore**. Teste o caminho de recuperação no notebook antes de depender dele; iniciar um snapshot pelo menu do GRUB não equivale a restaurá-lo definitivamente. [Timeshift](https://github.com/linuxmint/timeshift), [grub-btrfs](https://github.com/Antynea/grub-btrfs).

## HD secundário

O HD de 1 TB do Acer será configurado manualmente. Este repositório **não contém comando para particionar, formatar ou desbloquear discos**. Antes de qualquer operação no notebook, confirme modelo, partições e montagens do HD por um caminho estável em `/dev/disk/by-id/`; o SSD do sistema não é alvo desse preparo.

## Organização

| Caminho | Conteúdo |
| --- | --- |
| `archsetup` | Interface de comandos e instalação do link local. |
| `modules/` | Diagnóstico, pacotes, paru e módulos do desktop. |
| `packages/official/`, `packages/aur/` | Pacotes compartilhados, um nome por linha. |
| `profiles/<identificador>/` | Dados e pacotes específicos da máquina. |

O [perfil Acer](profiles/acer-aspire-vx5-591g-54pg/README.md) registra o hardware conhecido e as verificações pendentes. Para adicionar outro computador, crie seu perfil e confira a seleção com `archsetup packages --profile <identificador> --dry-run`.
