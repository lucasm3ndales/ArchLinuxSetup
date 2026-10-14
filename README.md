# ArchSetup

ArchSetup automatiza parte da pós-instalação do Arch Linux: instala pacotes por categoria e aplica ajustes opcionais ao Xfce, à tela de login e ao touchpad. Use `--dry-run` para conferir cada ação antes de executá-la. O projeto não instala o sistema operacional nem configura partições ou boot.

## Disponibilizar o comando

Tenha Rust 1.85 ou mais recente e Cargo disponíveis (via `rust` ou `rustup`). Como usuário comum, compile o projeto no repositório clonado e crie o comando:

```bash
cargo build --release
./target/release/archsetup link
```

Isso cria `~/.local/bin/archsetup` como link para o executável compilado e, se necessário, adiciona esse diretório ao PATH do Bash ou Zsh. Abra um novo terminal e confira com `archsetup help`. Mantenha o repositório no mesmo lugar para preservar o link.

## CLI em Rust

A CLI implementa `link`, `unlink`, `restore`, `doctor`, `profiles`, `aur`, `pkg`, `desktop` e `all` em Rust. Após um `git pull`, recompile para aplicar mudanças no código:

```bash
cargo build --release
archsetup help
```

O link continua válido após a recompilação, pois aponta para `target/release/archsetup`. Se a instalação anterior apontava para o antigo launcher Bash, execute `./target/release/archsetup link` uma vez para atualizar o link. O executável usa as listas de `packages/` e `profiles/` no repositório; não mova apenas o binário.

### Organização do código Rust

O projeto usa uma arquitetura hexagonal enxuta em um único crate. `src/commands/` recebe argumentos e apresenta resultados; `src/application/` concentra as decisões dos casos de uso; `src/adapters/` implementa o acesso ao Linux. Portas (`traits`) são usadas quando ajudam a separar uma regra do acesso ao sistema. A checagem resumida do `doctor` recebe um inspetor do sistema e devolve um relatório. `link` e `unlink` preparam planos antes de gravar arquivos; as prévias detectam links que pertencem a outro projeto. O layout do painel e o conteúdo do greeter são planejados em `src/application/desktop.rs`; os adaptadores executam os comandos do Xfce e gravam os arquivos do sistema. `restore` valida registros e decide quais arquivos pertencem ao ArchSetup antes de restaurá-los. O diagnóstico `doctor --verbose` ainda usa funções de `src/system.rs`.

```text
CLI (commands) -> casos de uso (application) -> portas (traits)
                                             <- adaptadores Linux (adapters)
```

O objetivo é manter as regras testáveis sem exigir vários crates ou criar uma interface para cada função. A aplicação não deve importar adaptadores nem executar `pacman`, `sudo` ou ler arquivos diretamente.

## Comandos e opções

| Comando | O que faz |
| --- | --- |
| `archsetup link [--dry-run]` | Cria ou atualiza o link para o binário Rust em `~/.local/bin` e ajusta o PATH do Bash ou Zsh; não instala o Arch Linux nem os pacotes das categorias. |
| `archsetup unlink [--dry-run]` | Remove apenas o link do comando e a linha PATH registrada pelo ArchSetup. |
| `archsetup restore [--packages] [--dry-run]` | Recupera configurações anteriores com backup. `--packages` também remove pacotes novos registrados pelo ArchSetup. Preserva o comando instalado. |
| `archsetup doctor [--profile NOME] [--verbose]` | Confere Arch, usuário, perfil, Xfce/sessão e sinais do layout Btrfs, Timeshift e GRUB. Com `--verbose`, também mostra hardware, discos, sessão e serviços. Não altera o sistema. |
| `archsetup aur [--dry-run]` | Instala o auxiliar `paru` para pacotes da AUR; se ele já existir, não o reinstala. |
| `archsetup pkg [CATEGORIA...] [--profile NOME] [--dry-run]` | Instala os pacotes oficiais e AUR das categorias escolhidas. Sem categoria, usa `base development apps theme desktop`. |
| `archsetup desktop [--profile NOME] [--dry-run]` | Configura o desktop; atualmente só Xfce. Exige Xfce instalado e uma sessão X11 do Xfce. |
| `archsetup profiles` | Lista os perfis disponíveis neste repositório. |
| `archsetup all [--profile NOME] [--dry-run]` | Executa `aur`, `pkg` e `desktop` nessa ordem. Exige Xfce instalado e sessão X11 do Xfce antes da primeira etapa. |
| `archsetup help [COMANDO]` | Mostra a ajuda geral ou a sintaxe de um comando. `archsetup --help` também mostra a ajuda geral. |

| Opção | Onde usar | Efeito |
| --- | --- | --- |
| `--dry-run` | `link`, `unlink`, `restore`, `aur`, `pkg`, `desktop`, `all` | Mostra uma prévia sem instalar pacotes ou alterar arquivos. A prévia não garante que todos os pré-requisitos da aplicação estejam atendidos. |
| `--packages` | `restore` | Remove somente pacotes registrados como novos nas execuções de `aur` e `pkg`. |
| `--profile NOME` | `doctor`, `pkg`, `desktop`, `all` | Seleciona explicitamente um perfil de equipamento. `doctor` e as aplicações validam a compatibilidade; as prévias verificam apenas se o perfil existe. |
| `--verbose` | `doctor` | Acrescenta o diagnóstico detalhado à checagem resumida. Revise identificadores e UUIDs antes de compartilhar a saída. |
| `--help` ou `-h` | Comandos que aceitam opções | Mostra a ajuda do comando. Também é possível usar `archsetup help COMANDO`. |

Execute os comandos como usuário comum. `doctor` é sempre somente leitura; `sudo` é solicitado somente nas ações que exigem privilégio.

## Uso básico

```bash
archsetup doctor
archsetup doctor --verbose
archsetup aur --dry-run
archsetup aur
archsetup pkg --dry-run
archsetup pkg
```

Para executar as três etapas num sistema novo **com Xfce já instalado**, use `all`. Ele verifica o Xfce e a sessão X11 antes de instalar qualquer pacote, executa `aur`, `pkg` e `desktop` nessa ordem e para no primeiro erro. O perfil é repassado a `pkg` e `desktop`:

```bash
archsetup profiles
archsetup all --profile NOME --dry-run
archsetup all --profile NOME
```

O comando `pkg` usa `base development apps theme desktop` quando nenhuma categoria é informada. Esse padrão instala o mesmo conjunto de pacotes anterior à divisão de `desktop`. Se os pacotes escolhidos exigirem um auxiliar AUR e não houver `paru` nem `yay`, o comando oferece executar `archsetup aur` antes de instalar. Para instalar só categorias sem AUR:

```bash
archsetup pkg base development --dry-run
archsetup pkg base development
```

Para instalar uma categoria mais tarde, execute apenas ela. Por exemplo:

```bash
archsetup pkg gaming --dry-run
archsetup pkg gaming
archsetup pkg emulation
```

Reexecutar `pkg` ignora pacotes já instalados com `--needed`, mas também pode atualizar os demais pacotes do sistema.

Antes de cada instalação, `aur` e `pkg` salvam uma lista dos pacotes existentes. Depois, registram somente os pacotes que apareceram nessa execução, incluindo dependências. Os arquivos ficam em `~/.local/state/archlinuxsetup/packages-baseline` e `packages-installed` (ou sob `XDG_STATE_HOME`). Pacotes já existentes não entram na lista de remoção. O registro começa a valer quando esta versão do ArchSetup é usada; ele não consegue reconstruir instalações de versões anteriores nem distinguir uma instalação feita simultaneamente por outro processo. Se a consulta final de pacotes falhar, a lista anterior permanece em `packages-before-*` para conferência manual.

Em uma sessão X11 do Xfce, aplique os ajustes do desktop:

```bash
archsetup desktop --dry-run
archsetup desktop
```

`desktop` configura apenas Xfce. Antes de alterar arquivos, confere se `xfce4-session` e `xfce4-panel` estão instalados e se a sessão atual é X11 do Xfce. O `doctor` mostra essa condição como aviso; `all` exige a mesma condição antes de começar a instalar. O projeto pressupõe que Xfce, LightDM e Slick Greeter tenham sido escolhidos no archinstall ou instalados manualmente. A categoria `pkg desktop` instala utilitários e plugins para o ambiente, mas não instala o Xfce. O comando também exige LightDM habilitado e Slick Greeter instalado. Se faltarem pacotes dos ajustes, oferece executar `archsetup pkg desktop theme` antes de alterar configurações; isso inclui o `xfce4-kbdleds-plugin` da AUR e o tema Nordic.

## Categorias de pacotes

| Categoria | Conteúdo |
| --- | --- |
| `base` | Ferramentas de terminal e desenvolvimento básicas, como Git, curl, Neovim, `base-devel`, jq e ripgrep. |
| `development` | GCC, Clang, CMake, depuradores, rustup, Docker Engine e Compose, DBeaver e Kleopatra. |
| `apps` | Discord, Telegram, qBittorrent, Brave, Spotify e ChatGPT Desktop. |
| `theme` | Fontes Inter e Fira Code, temas Nordic, ícones e cursores Nordzy. |
| `desktop` | Utilitários X11, áudio, portal GTK, `ksuperkey` e o plugin Keyboard LEDs. Pressupõe Xfce já instalado. O perfil Acer também inclui Blueman. |
| `gaming` | Steam, GameMode e MangoHud. |
| `emulation` | RetroArch, Dolphin, PCSX2 e RPCS3. |

As listas ficam em `packages/official/` e `packages/aur/`. O comando elimina nomes repetidos e usa `pacman -Syu --needed --noconfirm` e `paru -S --needed --noconfirm` (ou `yay`). Sem `--dry-run`, as confirmações de instalação são aceitas automaticamente; erros e conflitos que não puderem ser resolvidos interrompem o comando. `gaming` exige `[multilib]` habilitado. A prévia mostra quais pacotes já estão instalados quando `pacman` está disponível; ela não calcula dependências nem versões futuras.

`aur` instala o auxiliar `paru`. Para isso, instala `git` e `base-devel` e, se não houver Rust instalado, instala `rustup` e configura a toolchain estável antes de compilar. Se `rust` já estiver instalado, usa essa ferramenta existente para a compilação. O comando mostra o PKGBUILD, lista os demais arquivos recebidos e executa `makepkg -si --noconfirm` como usuário comum. `pkg` instala pacotes AUR sem perguntas de confirmação; confira os PKGBUILDs antes de executá-lo. Consulte a [ArchWiki sobre a AUR](https://wiki.archlinux.org/title/Arch_User_Repository).

Os comandos que precisam de privilégios validam o `sudo` uma vez antes de alterar o sistema. O próprio `sudo` mantém uma autorização temporária no terminal; o projeto não lê nem salva sua senha. Se essa autorização expirar durante uma instalação longa, o `sudo` pode pedir a senha novamente.

Na categoria `development`, o comando usa `rustup` e instala a toolchain estável se não houver uma padrão. Se o pacote `rust` já estiver instalado, usa essa instalação e retira `rustup` da seleção, sem remover pacotes. Para migrar depois para o gerenciador de versões, confira os dependentes com `pacman -Qi rust`, remova `rust` com `sudo pacman -Rns rust` somente se a remoção for segura e execute `archsetup pkg development` novamente. Docker Desktop não é instalado. O comando adiciona o usuário atual ao grupo `docker` se necessário, inicia `docker.service` e o habilita para iniciar no boot. Saia e entre novamente para ativar a associação ao grupo; membros do grupo `docker` têm privilégios equivalentes aos de root.

## Perfis

Um perfil reúne pacotes e ajustes específicos de um equipamento. Sem `--profile`, os comandos usam apenas as listas compartilhadas. Nenhum perfil é escolhido automaticamente. Para usar um perfil existente:

```bash
archsetup doctor --profile NOME
archsetup pkg --profile NOME --dry-run
archsetup pkg --profile NOME
```

Substitua `NOME` pelo identificador de uma pasta em `profiles/`. Antes de instalar com um perfil, o ArchSetup valida a compatibilidade do equipamento. Consulte o README do perfil para suas opções e limitações.

## Ajustes do desktop

`desktop` aplica Nordic às janelas e aplicativos, Nordzy-dark aos ícones, Nordzy-cursors ao cursor e Inter à interface. Configura um painel superior único de 48 px com menu do Arch, busca, tarefas agrupadas, espaços de trabalho numerados, relógio e o `xfce4-kbdleds-plugin` para Caps, Num e Scroll Lock. Também configura `Alt+F1` e `ksuperkey` para abrir o menu com a tecla Super. Salva a configuração anterior em `~/.local/state/archlinuxsetup/backups/` antes de alterar o Xfce. Ao migrar da versão com Genmon, remove o indicador antigo do painel e preserva o script anterior no backup. Repetir o comando não duplica painéis ou plugins.

O mesmo comando aplica tema, ícones, cursor e fonte ao Slick Greeter, com fundo escuro e logo do Arch. Faz backup dos arquivos alterados em `/etc/lightdm/`, preserva as demais opções e não encerra a sessão atual. A nova aparência aparece no próximo login.

Também cria uma regra Xorg restrita a touchpads para rolagem natural. Alguns perfis acrescentam ajustes específicos, após validação do equipamento. O comando faz backup antes de substituir suas regras; as mudanças valem na próxima sessão X11.

## Limites e recuperação

ArchSetup não configura boot, criptografia, partições, drivers de GPU, energia ou snapshots. Antes de instalar muitos pacotes ou aplicar ajustes, crie um snapshot ou backup com a ferramenta de recuperação do seu sistema. `archsetup doctor` informa o que encontrou sobre Btrfs, Timeshift e GRUB, mas não garante que uma restauração esteja pronta; teste esse processo no equipamento em que será usado.

Para remover apenas o comando, use `archsetup unlink`. A linha do PATH só é retirada quando existe registro de que foi adicionada pelo ArchSetup. Para recuperar configurações anteriores sem remover o comando, saia da sessão gráfica do Xfce e execute em um TTY:

```bash
archsetup restore --dry-run
archsetup restore
```

O comando restaura o primeiro backup do Xfce e os backups originais de LightDM e touchpad, além do estado anterior registrado para Docker e dos arquivos `mimeapps.list` capturados antes de mudar o navegador padrão. Arquivos sem backup ou sem prova de que foram criados pelo ArchSetup são preservados. A restauração guarda uma cópia dos arquivos atuais em `~/.local/state/archlinuxsetup/backups/` e junto aos arquivos de sistema antes de substituí-los. Ajustes feitos com versões anteriores podem não ter registro suficiente para desfazer tudo automaticamente.

Para incluir pacotes instalados pelos comandos, revise a lista primeiro com `archsetup restore --packages --dry-run` e depois execute `archsetup restore --packages`. Essa opção só usa `packages-installed` e não remove pacotes que já existiam antes. A lista pode incluir dependências que você passou a usar em outros projetos; revise os nomes antes de executar. O `pacman` recusa a remoção se algum pacote ainda for necessário por outro. Pacotes atualizados que já existiam antes não voltam à versão antiga; para desfazer uma atualização do sistema, use um snapshot ou backup verificado.

## Estrutura

| Caminho | Conteúdo |
| --- | --- |
| `Cargo.toml` e `src/` | Projeto da CLI Rust. |
| `target/release/archsetup` | Executável gerado por `cargo build --release`. |
| `src/commands/` | Entrada da CLI e apresentação dos comandos. |
| `src/application/` | Casos de uso e portas de acesso ao sistema. |
| `src/adapters/` | Implementações das portas para Linux. |
| `src/system.rs` | Funções de sistema compartilhadas pelos comandos. |
| `packages/official/`, `packages/aur/` | Listas compartilhadas, um pacote por linha. |
| `profiles/<identificador>/` | Dados e pacotes específicos de um equipamento. |
