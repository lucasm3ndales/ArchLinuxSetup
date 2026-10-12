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
| `archsetup paru` | Prepara o auxiliar AUR após revisão do PKGBUILD. |
| `archsetup packages [categorias]` | Instala pacotes oficiais e AUR das categorias escolhidas. |
| `archsetup xfce` | Aplica tema, fontes, painel e atalhos a uma sessão X11 do Xfce. |
| `archsetup greeter` | Aplica o visual Nordic ao LightDM com Slick Greeter. |
| `archsetup touchpad` | Ativa rolagem natural em touchpads X11. |

Use `archsetup help <comando>` para ver as opções. `link`, `paru`, `packages`, `xfce`, `greeter` e `touchpad` aceitam `--dry-run`; a prévia não instala pacotes nem altera arquivos. `check` e `doctor` são sempre somente leitura.

## Uso básico

```bash
archsetup check
archsetup doctor
archsetup paru --dry-run
archsetup paru
archsetup packages --dry-run
archsetup packages
```

O comando `packages` usa `base development desktop` quando nenhuma categoria é informada. Como `desktop` inclui pacotes AUR, a sequência padrão exige `paru` ou um `yay` já instalado. Para instalar só categorias sem AUR, você pode omitir essa etapa:

```bash
archsetup packages base development --dry-run
archsetup packages base development
```

Depois de instalar os pacotes, aplique apenas os ajustes que correspondem ao seu ambiente:

```bash
archsetup xfce --dry-run
archsetup xfce
archsetup greeter --dry-run
archsetup greeter
archsetup touchpad --dry-run
archsetup touchpad
```

`xfce` requer uma sessão X11 do Xfce. `greeter` requer LightDM habilitado e Slick Greeter instalado. Esses componentes não são instalados pelas categorias de pacotes do ArchSetup.

## Categorias de pacotes

| Categoria | Conteúdo |
| --- | --- |
| `base` | Ferramentas de terminal e desenvolvimento básicas, como Git, curl, Neovim, `base-devel`, jq e ripgrep. |
| `development` | GCC, Clang, CMake, depuradores, rustup, Docker Engine e Compose, DBeaver e Kleopatra. |
| `desktop` | Utilitários X11, fontes Inter e Fira Code, temas Nordic, ícones e cursores Nordzy, aplicativos de comunicação, Brave, Spotify, ChatGPT Desktop e `ksuperkey`. |
| `gaming` | Steam, GameMode e MangoHud. |
| `emulation` | RetroArch, Dolphin, PCSX2 e RPCS3. |

As listas ficam em `packages/official/` e `packages/aur/`. O script elimina nomes repetidos e usa `pacman -Syu --needed` e `paru -S --needed` (ou `yay`), sem suprimir as perguntas dessas ferramentas. Reexecutar uma categoria não duplica pacotes, mas pode atualizar o sistema. `gaming` exige `[multilib]` habilitado. A prévia mostra quais pacotes já estão instalados quando `pacman` está disponível; ela não calcula dependências nem versões futuras.

`paru` instala `git` e `base-devel`, mostra o PKGBUILD e só executa `makepkg -si` após confirmação digitada. Revise também os PKGBUILDs apresentados pelo auxiliar AUR ao instalar outros pacotes. Consulte a [ArchWiki sobre a AUR](https://wiki.archlinux.org/title/Arch_User_Repository).

Na categoria `development`, o script instala a toolchain Rust estável com rustup somente se não houver uma padrão. Se `rust` já estiver instalado, a aplicação para antes de instalar pacotes: confira os dependentes antes de trocá-lo por `rustup`. Docker Desktop não é instalado. O script adiciona o usuário atual ao grupo `docker` se necessário, inicia `docker.service` e o habilita para iniciar no boot. Saia e entre novamente para ativar a associação ao grupo; membros do grupo `docker` têm privilégios equivalentes aos de root.

## Perfis

Um perfil reúne pacotes e ajustes específicos de um equipamento. Sem `--profile`, os comandos usam apenas as listas compartilhadas. Nenhum perfil é escolhido automaticamente. Para usar um perfil existente:

```bash
archsetup check --profile NOME
archsetup packages --profile NOME --dry-run
archsetup packages --profile NOME
```

Substitua `NOME` pelo identificador de uma pasta em `profiles/`. Antes de instalar com um perfil, o ArchSetup valida a compatibilidade do equipamento. Consulte o README do perfil para suas opções e limitações.

## Ajustes do desktop

`xfce` aplica Nordic às janelas e aplicativos, Nordzy-dark aos ícones, Nordzy-cursors ao cursor e Inter à interface. Configura um painel superior único de 48 px com menu do Arch, busca, tarefas agrupadas, espaços de trabalho numerados, relógio e indicadores das teclas Lock. Também configura `Alt+F1` e `ksuperkey` para abrir o menu com a tecla Super. Salva a configuração anterior em `~/.local/state/archlinuxsetup/backups/` antes de alterar o Xfce. Repetir o comando não duplica painéis ou plugins.

`greeter` aplica os mesmos tema, ícones, cursor e fonte ao Slick Greeter, com fundo escuro e logo do Arch. Faz backup dos arquivos alterados em `/etc/lightdm/`, preserva as demais opções e não encerra a sessão atual. A nova aparência aparece no próximo login.

`touchpad` cria uma regra Xorg restrita a touchpads para rolagem natural. Alguns perfis acrescentam ajustes específicos, após validação do equipamento. O script faz backup antes de substituir suas regras; as mudanças valem na próxima sessão X11.

## Limites e recuperação

ArchSetup não configura boot, criptografia, partições, drivers de GPU, energia ou snapshots. Antes de instalar muitos pacotes ou aplicar ajustes, crie um snapshot ou backup com a ferramenta de recuperação do seu sistema. `archsetup check` informa o que encontrou sobre Btrfs, Timeshift e GRUB, mas não garante que uma restauração esteja pronta; teste esse processo no equipamento em que será usado.

## Estrutura

| Caminho | Conteúdo |
| --- | --- |
| `archsetup` | Interface de comandos e instalação do link local. |
| `modules/` | Implementação dos comandos. |
| `packages/official/`, `packages/aur/` | Listas compartilhadas, um pacote por linha. |
| `profiles/<identificador>/` | Dados e pacotes específicos de um equipamento. |
