# ArchSetup

ArchSetup automatiza parte da pós-instalação do Arch Linux: instala pacotes por categoria e aplica ajustes opcionais ao Xfce, à tela de login e ao touchpad. Use `--dry-run` para conferir cada ação antes de executá-la. O projeto não instala o sistema operacional nem configura partições ou boot.

## Instalação do comando

Execute como usuário comum com acesso a `sudo`, a partir do repositório clonado:

```bash
./archsetup link
```

Isso cria `~/.local/bin/archsetup` como link para o script do repositório e, se necessário, adiciona esse diretório ao PATH do Bash ou Zsh. Abra um novo terminal e confira com `archsetup help`. Mantenha o repositório no mesmo lugar para preservar o link.

## Comandos

| Comando | Ação |
| --- | --- |
| `archsetup check` | Confere pré-requisitos e informa o layout de recuperação detectado, sem alterar o sistema. |
| `archsetup doctor` | Mostra um diagnóstico somente leitura. Revise identificadores antes de compartilhar a saída. |
| `archsetup aur` | Exibe o PKGBUILD e instala o auxiliar paru para a AUR. |
| `archsetup packages [categorias]` | Instala pacotes oficiais e AUR das categorias escolhidas. |
| `archsetup desktop` | Configura Xfce, LightDM/Slick Greeter e touchpad X11. |

Use `archsetup help <comando>` para ver as opções. `link`, `aur`, `packages` e `desktop` aceitam `--dry-run`; a prévia não instala pacotes nem altera arquivos. `check` e `doctor` são sempre somente leitura.

## Uso básico

```bash
archsetup check
archsetup doctor
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

Em uma sessão X11 do Xfce, aplique os ajustes do desktop:

```bash
archsetup desktop --dry-run
archsetup desktop
```

`desktop` requer Xfce em sessão X11, LightDM habilitado e Slick Greeter instalado. Se faltar um pacote da categoria `desktop` necessário aos ajustes, o comando oferece executar `archsetup packages desktop` antes de alterar configurações. Xfce, LightDM e Slick Greeter devem estar instalados e ativos no sistema; a categoria de pacotes não os instala.

## Categorias de pacotes

| Categoria | Conteúdo |
| --- | --- |
| `base` | Ferramentas de terminal e desenvolvimento básicas, como Git, curl, Neovim, `base-devel`, jq e ripgrep. |
| `development` | GCC, Clang, CMake, depuradores, rustup, Docker Engine e Compose, DBeaver e Kleopatra. |
| `desktop` | Utilitários X11, fontes Inter e Fira Code, temas Nordic, ícones e cursores Nordzy, aplicativos de comunicação, Brave, Spotify, ChatGPT Desktop e `ksuperkey`. |
| `gaming` | Steam, GameMode e MangoHud. |
| `emulation` | RetroArch, Dolphin, PCSX2 e RPCS3. |

As listas ficam em `packages/official/` e `packages/aur/`. O script elimina nomes repetidos e usa `pacman -Syu --needed --noconfirm` e `paru -S --needed --noconfirm` (ou `yay`). Sem `--dry-run`, as confirmações de instalação são aceitas automaticamente; erros e conflitos que não puderem ser resolvidos interrompem o comando. Reexecutar uma categoria não duplica pacotes, mas pode atualizar o sistema. `gaming` exige `[multilib]` habilitado. A prévia mostra quais pacotes já estão instalados quando `pacman` está disponível; ela não calcula dependências nem versões futuras.

`aur` instala o auxiliar `paru`. Para isso, instala `git` e `base-devel` e, se não houver Rust instalado, instala `rustup` e configura a toolchain estável antes de compilar. Se `rust` já estiver instalado, usa essa ferramenta existente para a compilação. O comando mostra o PKGBUILD, lista os demais arquivos recebidos e executa `makepkg -si --noconfirm` como usuário comum. `packages` instala pacotes AUR sem perguntas de confirmação; confira os PKGBUILDs antes de executá-lo. Consulte a [ArchWiki sobre a AUR](https://wiki.archlinux.org/title/Arch_User_Repository).

Os comandos que precisam de privilégios validam o `sudo` uma vez antes de alterar o sistema. O próprio `sudo` mantém uma autorização temporária no terminal; o projeto não lê nem salva sua senha. Se essa autorização expirar durante uma instalação longa, o `sudo` pode pedir a senha novamente.

Na categoria `development`, o script usa `rustup` e instala a toolchain estável se não houver uma padrão. Se o pacote `rust` já estiver instalado, usa essa instalação e retira `rustup` da seleção, sem remover pacotes. Para migrar depois para o gerenciador de versões, confira os dependentes com `pacman -Qi rust`, remova `rust` com `sudo pacman -Rns rust` somente se a remoção for segura e execute `archsetup packages development` novamente. Docker Desktop não é instalado. O script adiciona o usuário atual ao grupo `docker` se necessário, inicia `docker.service` e o habilita para iniciar no boot. Saia e entre novamente para ativar a associação ao grupo; membros do grupo `docker` têm privilégios equivalentes aos de root.

## Perfis

Um perfil reúne pacotes e ajustes específicos de um equipamento. Sem `--profile`, os comandos usam apenas as listas compartilhadas. Nenhum perfil é escolhido automaticamente. Para usar um perfil existente:

```bash
archsetup check --profile NOME
archsetup packages --profile NOME --dry-run
archsetup packages --profile NOME
```

Substitua `NOME` pelo identificador de uma pasta em `profiles/`. Antes de instalar com um perfil, o ArchSetup valida a compatibilidade do equipamento. Consulte o README do perfil para suas opções e limitações.

## Ajustes do desktop

`desktop` aplica Nordic às janelas e aplicativos, Nordzy-dark aos ícones, Nordzy-cursors ao cursor e Inter à interface. Configura um painel superior único de 48 px com menu do Arch, busca, tarefas agrupadas, espaços de trabalho numerados, relógio e indicadores das teclas Lock. Também configura `Alt+F1` e `ksuperkey` para abrir o menu com a tecla Super. Salva a configuração anterior em `~/.local/state/archlinuxsetup/backups/` antes de alterar o Xfce. Repetir o comando não duplica painéis ou plugins.

O mesmo comando aplica tema, ícones, cursor e fonte ao Slick Greeter, com fundo escuro e logo do Arch. Faz backup dos arquivos alterados em `/etc/lightdm/`, preserva as demais opções e não encerra a sessão atual. A nova aparência aparece no próximo login.

Também cria uma regra Xorg restrita a touchpads para rolagem natural. Alguns perfis acrescentam ajustes específicos, após validação do equipamento. O script faz backup antes de substituir suas regras; as mudanças valem na próxima sessão X11.

## Limites e recuperação

ArchSetup não configura boot, criptografia, partições, drivers de GPU, energia ou snapshots. Antes de instalar muitos pacotes ou aplicar ajustes, crie um snapshot ou backup com a ferramenta de recuperação do seu sistema. `archsetup check` informa o que encontrou sobre Btrfs, Timeshift e GRUB, mas não garante que uma restauração esteja pronta; teste esse processo no equipamento em que será usado.

## Estrutura

| Caminho | Conteúdo |
| --- | --- |
| `archsetup` | Interface de comandos e instalação do link local. |
| `modules/` | Implementação dos comandos. |
| `packages/official/`, `packages/aur/` | Listas compartilhadas, um pacote por linha. |
| `profiles/<identificador>/` | Dados e pacotes específicos de um equipamento. |
