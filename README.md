# ArchSetup

ArchSetup automatiza parte da pós-instalação do Arch Linux: instala pacotes por categoria e aplica ajustes opcionais ao Xfce, à tela de login e ao touchpad. Use `--dry-run` para conferir cada ação antes de executá-la. O projeto não instala o sistema operacional nem configura partições ou boot.

## Instalação do comando

Execute como usuário comum com acesso a `sudo`, a partir do repositório clonado:

```bash
./archsetup install
```

Isso cria `~/.local/bin/archsetup` como link para o script do repositório e, se necessário, adiciona esse diretório ao PATH do Bash ou Zsh. Abra um novo terminal e confira com `archsetup help`. Mantenha o repositório no mesmo lugar para preservar o link. Se o link já aponta para este repositório, um `git pull` atualiza o comando sem exigir outra instalação.

## Comandos e opções

| Comando | O que faz |
| --- | --- |
| `archsetup install [--dry-run]` | Disponibiliza o comando em `~/.local/bin` e ajusta o PATH do Bash ou Zsh; não instala o Arch Linux nem os pacotes das categorias. |
| `archsetup uninstall [--restore [--packages]] [--dry-run]` | Remove o comando. `--restore` recupera configurações anteriores com backup; `--packages` também remove pacotes novos registrados pelo ArchSetup. |
| `archsetup doctor [--profile NOME] [--verbose]` | Confere Arch, usuário, perfil e alguns sinais do layout Btrfs, Timeshift e GRUB. Com `--verbose`, também mostra hardware, discos, sessão e serviços. Não altera o sistema. |
| `archsetup aur [--dry-run]` | Instala o auxiliar `paru` para pacotes da AUR; se ele já existir, não o reinstala. |
| `archsetup packages [CATEGORIA...] [--profile NOME] [--dry-run]` | Instala os pacotes oficiais e AUR das categorias escolhidas. Sem categoria, usa `base development desktop`. |
| `archsetup desktop [--profile NOME] [--dry-run]` | Configura Xfce, LightDM/Slick Greeter e touchpad X11. Deve ser aplicado numa sessão X11 do Xfce. |
| `archsetup help [COMANDO]` | Mostra a ajuda geral ou a sintaxe de um comando. `archsetup --help` também mostra a ajuda geral. |

| Opção | Onde usar | Efeito |
| --- | --- | --- |
| `--dry-run` | `install`, `uninstall`, `aur`, `packages`, `desktop` | Mostra uma prévia sem instalar pacotes ou alterar arquivos. A prévia não garante que todos os pré-requisitos da aplicação estejam atendidos. |
| `--restore` | `uninstall` | Recupera configurações anteriores e remove o comando; execute fora da sessão Xfce, em um TTY. |
| `--packages` | `uninstall --restore` | Remove somente pacotes registrados como novos nas execuções de `aur` e `packages`. |
| `--profile NOME` | `doctor`, `packages`, `desktop` | Seleciona explicitamente um perfil de equipamento. `doctor` e as aplicações validam a compatibilidade; as prévias verificam apenas se o perfil existe. |
| `--verbose` | `doctor` | Acrescenta o diagnóstico detalhado à checagem resumida. Revise identificadores e UUIDs antes de compartilhar a saída. |
| `--help` ou `-h` | Comandos que aceitam opções | Mostra a ajuda do comando. Também é possível usar `archsetup help COMANDO`. |

Execute os comandos como usuário comum. `doctor` é sempre somente leitura; `sudo` é solicitado somente nas ações que exigem privilégio.

## Uso básico

```bash
archsetup doctor
archsetup doctor --verbose
archsetup aur --dry-run
archsetup aur
archsetup packages --dry-run
archsetup packages
```

O comando `packages` usa `base development desktop` quando nenhuma categoria é informada. Se os pacotes escolhidos exigirem um auxiliar AUR e não houver `paru` nem `yay`, o comando oferece executar `archsetup aur` antes de instalar. Para instalar só categorias sem AUR:

```bash
archsetup packages base development --dry-run
archsetup packages base development
```

Para instalar uma categoria mais tarde, execute apenas ela. Por exemplo:

```bash
archsetup packages gaming --dry-run
archsetup packages gaming
archsetup packages emulation
```

Reexecutar `packages` ignora pacotes já instalados com `--needed`, mas também pode atualizar os demais pacotes do sistema.

Antes de cada instalação, `aur` e `packages` salvam uma lista dos pacotes existentes. Depois, registram somente os pacotes que apareceram nessa execução, incluindo dependências. Os arquivos ficam em `~/.local/state/archlinuxsetup/packages-baseline` e `packages-installed` (ou sob `XDG_STATE_HOME`). Pacotes já existentes não entram na lista de remoção. O registro começa a valer quando esta versão dos scripts é usada; ele não consegue reconstruir instalações de versões anteriores nem distinguir uma instalação feita simultaneamente por outro processo.

Em uma sessão X11 do Xfce, aplique os ajustes do desktop:

```bash
archsetup desktop --dry-run
archsetup desktop
```

`desktop` requer Xfce em sessão X11, LightDM habilitado e Slick Greeter instalado. Se faltar um pacote da categoria `desktop` necessário aos ajustes, o comando oferece executar `archsetup packages desktop` antes de alterar configurações. Isso inclui o `xfce4-kbdleds-plugin` da AUR. Xfce, LightDM e Slick Greeter devem estar instalados e ativos no sistema; a categoria de pacotes não os instala.

## Categorias de pacotes

| Categoria | Conteúdo |
| --- | --- |
| `base` | Ferramentas de terminal e desenvolvimento básicas, como Git, curl, Neovim, `base-devel`, jq e ripgrep. |
| `development` | GCC, Clang, CMake, depuradores, rustup, Docker Engine e Compose, DBeaver e Kleopatra. |
| `desktop` | Utilitários X11, fontes Inter e Fira Code, temas Nordic, ícones e cursores Nordzy, aplicativos de comunicação, Brave, Spotify, ChatGPT Desktop, `ksuperkey` e o plugin Keyboard LEDs. |
| `gaming` | Steam, GameMode e MangoHud. |
| `emulation` | RetroArch, Dolphin, PCSX2 e RPCS3. |

As listas ficam em `packages/official/` e `packages/aur/`. O script elimina nomes repetidos e usa `pacman -Syu --needed --noconfirm` e `paru -S --needed --noconfirm` (ou `yay`). Sem `--dry-run`, as confirmações de instalação são aceitas automaticamente; erros e conflitos que não puderem ser resolvidos interrompem o comando. `gaming` exige `[multilib]` habilitado. A prévia mostra quais pacotes já estão instalados quando `pacman` está disponível; ela não calcula dependências nem versões futuras.

`aur` instala o auxiliar `paru`. Para isso, instala `git` e `base-devel` e, se não houver Rust instalado, instala `rustup` e configura a toolchain estável antes de compilar. Se `rust` já estiver instalado, usa essa ferramenta existente para a compilação. O comando mostra o PKGBUILD, lista os demais arquivos recebidos e executa `makepkg -si --noconfirm` como usuário comum. `packages` instala pacotes AUR sem perguntas de confirmação; confira os PKGBUILDs antes de executá-lo. Consulte a [ArchWiki sobre a AUR](https://wiki.archlinux.org/title/Arch_User_Repository).

Os comandos que precisam de privilégios validam o `sudo` uma vez antes de alterar o sistema. O próprio `sudo` mantém uma autorização temporária no terminal; o projeto não lê nem salva sua senha. Se essa autorização expirar durante uma instalação longa, o `sudo` pode pedir a senha novamente.

Na categoria `development`, o script usa `rustup` e instala a toolchain estável se não houver uma padrão. Se o pacote `rust` já estiver instalado, usa essa instalação e retira `rustup` da seleção, sem remover pacotes. Para migrar depois para o gerenciador de versões, confira os dependentes com `pacman -Qi rust`, remova `rust` com `sudo pacman -Rns rust` somente se a remoção for segura e execute `archsetup packages development` novamente. Docker Desktop não é instalado. O script adiciona o usuário atual ao grupo `docker` se necessário, inicia `docker.service` e o habilita para iniciar no boot. Saia e entre novamente para ativar a associação ao grupo; membros do grupo `docker` têm privilégios equivalentes aos de root.

## Perfis

Um perfil reúne pacotes e ajustes específicos de um equipamento. Sem `--profile`, os comandos usam apenas as listas compartilhadas. Nenhum perfil é escolhido automaticamente. Para usar um perfil existente:

```bash
archsetup doctor --profile NOME
archsetup packages --profile NOME --dry-run
archsetup packages --profile NOME
```

Substitua `NOME` pelo identificador de uma pasta em `profiles/`. Antes de instalar com um perfil, o ArchSetup valida a compatibilidade do equipamento. Consulte o README do perfil para suas opções e limitações.

## Ajustes do desktop

`desktop` aplica Nordic às janelas e aplicativos, Nordzy-dark aos ícones, Nordzy-cursors ao cursor e Inter à interface. Configura um painel superior único de 48 px com menu do Arch, busca, tarefas agrupadas, espaços de trabalho numerados, relógio e o `xfce4-kbdleds-plugin` para Caps, Num e Scroll Lock. Também configura `Alt+F1` e `ksuperkey` para abrir o menu com a tecla Super. Salva a configuração anterior em `~/.local/state/archlinuxsetup/backups/` antes de alterar o Xfce. Ao migrar da versão com Genmon, remove o indicador antigo do painel e preserva o script anterior no backup. Repetir o comando não duplica painéis ou plugins.

O mesmo comando aplica tema, ícones, cursor e fonte ao Slick Greeter, com fundo escuro e logo do Arch. Faz backup dos arquivos alterados em `/etc/lightdm/`, preserva as demais opções e não encerra a sessão atual. A nova aparência aparece no próximo login.

Também cria uma regra Xorg restrita a touchpads para rolagem natural. Alguns perfis acrescentam ajustes específicos, após validação do equipamento. O script faz backup antes de substituir suas regras; as mudanças valem na próxima sessão X11.

## Limites e recuperação

ArchSetup não configura boot, criptografia, partições, drivers de GPU, energia ou snapshots. Antes de instalar muitos pacotes ou aplicar ajustes, crie um snapshot ou backup com a ferramenta de recuperação do seu sistema. `archsetup doctor` informa o que encontrou sobre Btrfs, Timeshift e GRUB, mas não garante que uma restauração esteja pronta; teste esse processo no equipamento em que será usado.

Para remover apenas o comando, use `archsetup uninstall`. A linha do PATH só é retirada quando existe registro de que foi adicionada pelo ArchSetup. Para recuperar configurações anteriores, saia da sessão gráfica do Xfce e execute em um TTY:

```bash
archsetup uninstall --restore --dry-run
archsetup uninstall --restore
```

O comando restaura o primeiro backup do Xfce e os backups originais de LightDM e touchpad, além do estado anterior registrado para Docker e dos arquivos `mimeapps.list` capturados antes de mudar o navegador padrão. Arquivos sem backup ou sem prova de que foram criados pelo ArchSetup são preservados. A restauração guarda uma cópia dos arquivos atuais em `~/.local/state/archlinuxsetup/backups/` e junto aos arquivos de sistema antes de substituí-los. Ajustes feitos com versões anteriores podem não ter registro suficiente para desfazer tudo automaticamente.

Para incluir pacotes instalados pelos scripts, revise a lista primeiro com `archsetup uninstall --restore --packages --dry-run` e depois execute `archsetup uninstall --restore --packages`. Essa opção só usa `packages-installed` e não remove pacotes que já existiam antes. A lista pode incluir dependências que você passou a usar em outros projetos; revise os nomes antes de executar. O `pacman` recusa a remoção se algum pacote ainda for necessário por outro. Pacotes atualizados que já existiam antes não voltam à versão antiga; para desfazer uma atualização do sistema, use um snapshot ou backup verificado.

## Estrutura

| Caminho | Conteúdo |
| --- | --- |
| `archsetup` | Interface de comandos e instalação do link local. |
| `modules/` | Implementação dos comandos. |
| `packages/official/`, `packages/aur/` | Listas compartilhadas, um pacote por linha. |
| `profiles/<identificador>/` | Dados e pacotes específicos de um equipamento. |
