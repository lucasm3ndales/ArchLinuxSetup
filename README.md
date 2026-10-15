# ArchSetup

ArchSetup automatiza parte da pós-instalação do Arch Linux: instala pacotes por categoria e configura Xfce, a tela de login e o touchpad. Xfce é o ambiente padrão e o único suportado atualmente. Use `--dry-run` para conferir cada ação antes de executá-la. O projeto não instala o sistema operacional nem configura partições ou boot.

## Instalação

No Arch Linux, instale Git caso ainda não esteja disponível, clone o repositório e execute o instalador como usuário comum:

```bash
sudo pacman -Syu --needed git
git clone https://github.com/lucasm3ndales/ArchLinuxSetup.git
cd ArchLinuxSetup
./install.sh
```

Se o repositório já estiver clonado, execute apenas `./install.sh` na raiz. O instalador atualiza os pacotes do sistema, instala Git, `base-devel` e Rust/Cargo se necessário, compila o projeto e executa `archsetup link`. Ele respeita uma instalação existente de `rust` ou `rustup`, sem instalar os dois pacotes conflitantes. O link fica em `~/.local/bin/archsetup`; se necessário, o comando `link` adiciona esse diretório ao PATH do Bash ou Zsh. Abra um novo terminal e confira com `archsetup doctor`. Mantenha o repositório no mesmo lugar para preservar o link.

Se preferir fazer essas etapas manualmente, use Rust 1.85 ou mais recente e Cargo, depois execute `cargo build --release --locked` e `./target/release/archsetup link` como usuário comum.

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
| `archsetup doctor [--profile NOME] [--verbose]` | Confere Arch, usuário, perfil, Xfce/sessão e sinais do layout Btrfs, Snapper e boot. Com `--verbose`, também mostra hardware, discos, sessão e serviços. Não altera o sistema. |
| `archsetup aur [--dry-run]` | Instala o auxiliar `paru` para pacotes da AUR; se ele já existir, não o reinstala. |
| `archsetup pkg [CATEGORIA...] [--profile NOME] [--dry-run]` | Instala os pacotes oficiais e AUR das categorias escolhidas. Sem categoria, usa `base development apps theme desktop`. |
| `archsetup desktop [--profile NOME] [--dry-run]` | Configura o desktop padrão, Xfce. É o único ambiente suportado atualmente e exige uma sessão X11 do Xfce. |
| `archsetup profiles` | Lista os perfis disponíveis neste repositório. |
| `archsetup all [--profile NOME] [--dry-run]` | Executa `aur`, `pkg` e `desktop` nessa ordem. Exige Xfce instalado e sessão X11 do Xfce antes da primeira etapa. |
| `archsetup help [COMANDO]` | Mostra a ajuda geral ou a sintaxe de um comando. `archsetup --help` também mostra a ajuda geral. |

| Opção | Onde usar | Efeito |
| --- | --- | --- |
| `--dry-run` | `link`, `unlink`, `restore`, `aur`, `pkg`, `desktop`, `all` | Mostra uma prévia sem instalar pacotes ou alterar arquivos. A prévia não garante que todos os pré-requisitos da aplicação estejam atendidos. |
| `--packages` | `restore` | Remove somente pacotes registrados como novos nas execuções de `aur` e `pkg`. |
| `--profile NOME` | `doctor`, `pkg`, `desktop`, `all` | Seleciona explicitamente um perfil. `doctor` e as aplicações validam a compatibilidade; as prévias leem e validam o arquivo do perfil. |
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

Para executar as três etapas num sistema novo **com Xfce já instalado**, use `all`. Ele verifica o Xfce e a sessão X11 antes de instalar qualquer pacote, executa `aur`, `pkg` e `desktop` nessa ordem e para no primeiro erro. Xfce é o padrão do comando. O perfil é repassado a `pkg` e `desktop`:

```bash
archsetup profiles
archsetup all --profile NOME --dry-run
archsetup all --profile NOME
```

O comando `pkg` usa `base development apps theme desktop` quando nenhuma categoria é informada. Se os pacotes escolhidos exigirem um auxiliar AUR e não houver `paru` nem `yay`, o comando oferece executar `archsetup aur` antes de instalar. Para instalar só categorias sem AUR:

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

`desktop` configura apenas Xfce. Antes de alterar arquivos, confere se `xfce4-session` e `xfce4-panel` estão instalados e se a sessão atual é X11 do Xfce. O `doctor` mostra essa condição como aviso; `all` exige a mesma condição antes de começar a instalar. O projeto pressupõe que Xfce tenha sido escolhido no archinstall ou instalado manualmente. A categoria `pkg desktop` instala utilitários e plugins para o ambiente, mas não instala o Xfce. Se LightDM estiver habilitado, o comando exige Slick Greeter e aplica Arc-Dark, Arc e Hackneyed à tela de login. Com Ly ou outro gerenciador, preserva a tela de login existente. Se faltarem pacotes dos ajustes, oferece executar `archsetup pkg desktop theme` antes de alterar configurações; isso inclui o `xfce4-kbdleds-plugin` e os temas da AUR.

## Categorias de pacotes

| Categoria | Conteúdo |
| --- | --- |
| `base` | Ferramentas de terminal básicas, como Git, curl, Neovim, `base-devel`, jq e ripgrep. Não instala headers sem necessidade nem utilitários de disco escolhidos na instalação do sistema. |
| `development` | GCC, Clang, CMake, depuradores, rustup, Docker Engine e Compose, DBeaver, Kleopatra e VS Code (`visual-studio-code-bin`, AUR). |
| `apps` | Discord, Telegram, qBittorrent, Brave, Spotify e ChatGPT Desktop. |
| `theme` | Fontes Inter e Fira Code, tema GTK/Xfwm4 Arc-Dark (`arc-gtk-theme`), ícones Arc (`arc-icon-theme`) e cursor Hackneyed (`xcursor-hackneyed-light`). Os três temas vêm da AUR. |
| `desktop` | Utilitários X11, incluindo `xorg-xauth` para sessões iniciadas pelo Ly, áudio, portal GTK, `ksuperkey` e o plugin Keyboard LEDs. Pressupõe Xfce já instalado. O perfil Acer também inclui Blueman. |
| `gaming` | Steam, GameMode e MangoHud. |
| `emulation` | RetroArch, Dolphin, PCSX2 e RPCS3. |

As listas compartilhadas ficam em `packages/official/` e `packages/aur/`, com um pacote por linha em arquivos `.txt`. Perfis usam TOML porque também declaram identificação DMI e ajustes. O comando elimina nomes repetidos e usa `pacman -Syu --needed --noconfirm` e `paru -S --needed --noconfirm` (ou `yay`). Sem `--dry-run`, as confirmações de instalação são aceitas automaticamente; erros e conflitos que não puderem ser resolvidos interrompem o comando. `gaming` exige `[multilib]` habilitado. A prévia mostra quais pacotes já estão instalados quando `pacman` está disponível; ela não calcula dependências nem versões futuras.

`aur` instala o auxiliar `paru`. Para isso, instala `git` e `base-devel` e, se não houver Rust instalado, instala `rustup` e configura a toolchain estável antes de compilar. Se `rust` já estiver instalado, usa essa ferramenta existente para a compilação. O comando mostra o PKGBUILD, lista os demais arquivos recebidos e executa `makepkg -si --noconfirm` como usuário comum. `pkg` instala pacotes AUR sem perguntas de confirmação; confira os PKGBUILDs antes de executá-lo. Consulte a [ArchWiki sobre a AUR](https://wiki.archlinux.org/title/Arch_User_Repository). O instalador e as categorias não trocam NetworkManager, PipeWire, Ly, power-profiles-daemon, UFW, serviço de impressão, Snapper ou systemd-boot. Pacotes já instalados são ignorados por `--needed`.

Os comandos que precisam de privilégios validam o `sudo` uma vez antes de alterar o sistema. O próprio `sudo` mantém uma autorização temporária no terminal; o projeto não lê nem salva sua senha. Se essa autorização expirar durante uma instalação longa, o `sudo` pode pedir a senha novamente.

Na categoria `development`, o comando usa `rustup` e instala a toolchain estável se não houver uma padrão. Se o pacote `rust` já estiver instalado, usa essa instalação e retira `rustup` da seleção, sem remover pacotes. Para migrar depois para o gerenciador de versões, confira os dependentes com `pacman -Qi rust`, remova `rust` com `sudo pacman -Rns rust` somente se a remoção for segura e execute `archsetup pkg development` novamente. Docker Desktop não é instalado. O comando adiciona o usuário atual ao grupo `docker` se necessário, inicia `docker.service` e o habilita para iniciar no boot. Saia e entre novamente para ativar a associação ao grupo; membros do grupo `docker` têm privilégios equivalentes aos de root.

Se UFW estiver ativo, confira as portas publicadas pelos containers: o Docker pode encaminhar esse tráfego antes das regras do UFW. O comando avisa sobre isso ao configurar o Docker. Consulte a [documentação do Docker sobre firewalls](https://docs.docker.com/engine/network/packet-filtering-firewalls/).

## Perfis

Um perfil reúne pacotes e ajustes específicos de um equipamento. Os perfis disponíveis são `generic` e `acer-aspire-vx5-591g`. `generic` é a escolha explícita para usar apenas as listas compartilhadas. Sem `--profile`, os comandos também usam apenas essas listas. `doctor` lê fabricante e modelo DMI e sugere um perfil compatível; ele não aplica o perfil automaticamente. Para usar o perfil Acer:

```bash
archsetup doctor --profile acer-aspire-vx5-591g
archsetup pkg --profile acer-aspire-vx5-591g --dry-run
archsetup pkg --profile acer-aspire-vx5-591g
```

Para outros perfis, substitua o valor de `--profile` pelo identificador de uma pasta em `profiles/`. Antes de instalar com um perfil específico, o ArchSetup valida a compatibilidade do equipamento. O perfil Acer usa o modelo base VX5-591G, sem sufixo regional. Consulte o README do perfil para suas opções e limitações.

No perfil Acer, `gaming` é opcional e não faz parte de `all`: ele inclui o driver legado `nvidia-580xx-dkms` da AUR para variantes com GTX 10xx, as bibliotecas gráficas Intel/NVIDIA de 32 bits e os headers correspondentes aos kernels instalados. `base` sozinho não instala headers. Confira a GPU e os pacotes NVIDIA existentes antes de executar `gaming`. Se houver driver oficial conflitante, o comando recusa a instalação e não remove nada automaticamente. A [ArchWiki sobre NVIDIA](https://wiki.archlinux.org/title/NVIDIA) identifica o ramo 580xx para Pascal.

Para contribuir com outro modelo, crie `profiles/fabricante-modelo/profile.toml`:

```toml
name = "fabricante-modelo"
description = "Fabricante Modelo"
match_vendor = "Fabricante"
match_dmi = "Modelo"

[packages.official]
desktop = ["blueman"]
gaming = ["lib32-mesa"]

[packages.aur]
# desktop = ["nome-do-pacote"]

[desktop]
touchpad_tapping = true
```

`name` deve ser igual ao nome da pasta. `match_vendor` e `match_dmi` são obrigatórios para perfis de equipamento e fazem comparação parcial sem diferenciar maiúsculas de minúsculas. As listas aceitam as mesmas categorias de `pkg`; seções e campos desconhecidos são recusados. O formato atual aceita strings simples, listas em uma linha e `touchpad_tapping` como booleano. Outros ajustes e serviços ainda não fazem parte do esquema. Confira os pacotes e o ajuste no equipamento antes de enviar um PR.

## Ajustes do desktop

`desktop` aplica Arc-Dark às janelas e aplicativos, Arc aos ícones, Hackneyed ao cursor e Inter à interface. Ajusta o cursor também em `~/.Xresources` e `~/.icons/default/index.theme`, preservando as outras opções desses arquivos. Configura um painel superior único de 48 px com menu do Arch, busca, tarefas agrupadas, espaços de trabalho numerados, relógio e o `xfce4-kbdleds-plugin` para Caps, Num e Scroll Lock. Também configura `Alt+F1` e `ksuperkey` para abrir o menu com a tecla Super. Salva a configuração anterior em `~/.local/state/archlinuxsetup/backups/` antes de alterar o Xfce. `restore` recupera os arquivos de cursor originais; remove um arquivo criado pelo ArchSetup somente se ele ainda tiver o conteúdo gerado. Ao migrar da versão com Genmon, remove o indicador antigo do painel e preserva o script anterior no backup. Repetir o comando não duplica painéis ou plugins.

Se LightDM estiver habilitado, o mesmo comando aplica tema, ícones, cursor e fonte ao Slick Greeter, com fundo escuro e logo do Arch. Faz backup dos arquivos alterados em `/etc/lightdm/`, preserva as demais opções e não encerra a sessão atual. A nova aparência aparece no próximo login. Com Ly, o comando deixa o greeter como está.

Também cria uma regra Xorg restrita a touchpads para rolagem natural. Alguns perfis acrescentam ajustes específicos, após validação do equipamento. O comando faz backup antes de substituir suas regras; as mudanças valem na próxima sessão X11.

## Limites e recuperação

ArchSetup não configura boot, criptografia, partições, energia ou snapshots. O perfil Acer inclui o driver NVIDIA legado apenas na categoria opcional `gaming`; ele não altera o boot nem remove drivers conflitantes. Antes de instalar muitos pacotes ou aplicar ajustes, crie um snapshot ou backup com a ferramenta de recuperação do seu sistema. `archsetup doctor` informa o que encontrou sobre Btrfs, Snapper e a partição EFI, mas não garante que uma restauração esteja pronta.

**Snapper com systemd-boot e UKI:** Snapper cria snapshots Btrfs, mas systemd-boot não cria entradas para eles automaticamente. UKIs ficam na partição EFI ou XBOOTLDR, fora do snapshot da raiz. Se a linha de boot fixa `rootflags=subvol=@`, apenas trocar o subvolume padrão com `snapper rollback` pode continuar iniciando `@`, sem aplicar a restauração. Para restaurar o sistema após uma atualização de kernel, pode ser necessário iniciar por uma ISO, restaurar o subvolume raiz e reconstruir a UKI correspondente; teste o procedimento no equipamento antes de depender dele. Se `/home` é outro subvolume, seu conteúdo não volta junto com a raiz. Confira `cat /proc/cmdline`, `findmnt /`, `findmnt /boot` e `bootctl status` antes de planejar o retorno. Consulte a [ArchWiki: Snapper](https://wiki.archlinux.org/title/Snapper), [Btrfs](https://wiki.archlinux.org/title/Btrfs), [UKI](https://wiki.archlinux.org/title/Unified_kernel_image) e [partição EFI](https://wiki.archlinux.org/title/EFI_system_partition).

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
| `profiles/<identificador>/profile.toml` | Metadados, pacotes extras e ajustes específicos de um equipamento. `generic` não exige hardware. |
