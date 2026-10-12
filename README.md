# ArchLinuxSetup

Configurador de pós-instalação do Arch Linux. O projeto reúne pacotes de uso pessoal, separa ajustes comuns dos específicos de cada computador e permite repetir a configuração em novas instalações.

## O que funciona hoje

- `diagnose` mostra informações do sistema sem alterá-lo.
- `packages plan` mostra a seleção de pacotes, sem consultar quais já estão instalados.
- `packages apply` atualiza o Arch e instala os pacotes selecionados.
- `aur plan/apply` prepara o `paru`, auxiliar da AUR escolhido para instalações novas.
- `xfce plan` mostra o visual proposto; `xfce apply` configura tema, fontes, dock com menu do Arch e indicador das teclas Lock.
- `system plan/apply` prepara Ly, zram com zstd e rolagem natural do touchpad.
- `storage plan/apply/unlock` inspeciona, prepara e desbloqueia o HD secundário do Acer.

O SSD do sistema continua sob responsabilidade do archinstall. Há um comando separado para preparar o HD secundário, com confirmação explícita. O projeto ainda não automatiza boot, drivers NVIDIA, outros ajustes do touchpad, energia nem a criação de configurações Snapper.

## Início rápido

Requisitos para instalar pacotes: Arch Linux já instalado e usuário comum com acesso a `sudo`. Prepare o `paru` com o comando `aur apply` antes das categorias que usam AUR. Se já houver `paru` ou `yay`, `packages apply` pode usar o auxiliar existente; a preferência é pelo `paru`.

```bash
./install.sh help
./install.sh diagnose
./install.sh aur plan
./install.sh aur apply
./install.sh packages plan --profile acer-aspire-vx5-591g-54pg
./install.sh packages apply --profile acer-aspire-vx5-591g-54pg
./install.sh xfce plan
./install.sh xfce apply
./install.sh system plan
./install.sh system apply
./install.sh storage plan
```

Execute o diagnóstico como usuário comum. Na sessão X11 do Xfce, ele também consulta as opções do touchpad. Revise a saída antes de compartilhá-la: ela contém nomes de dispositivos, partições e pontos de montagem. O diagnóstico não confirma automaticamente qual perfil usar.

## Estrutura do repositório

| Caminho | Responsabilidade |
| --- | --- |
| `install.sh` | Entrada única e seleção explícita de comandos. |
| `modules/` | Etapas independentes: diagnóstico, AUR, pacotes, visual do Xfce, sistema e HD secundário. |
| `packages/official/` e `packages/aur/` | Pacotes compartilhados por categoria, válidos para qualquer computador. |
| `profiles/<nome>/` | Ajustes e pacotes específicos de cada computador. Há somente o perfil do Acer neste momento. |

As categorias compartilhadas ficam fora dos perfis. Cada máquina futura ganhará sua própria pasta em `profiles/`; nenhum comando seleciona o perfil Acer automaticamente. Sem `--profile`, só entram os pacotes compartilhados. O perfil adiciona pacotes às categorias selecionadas, sem substituir as listas comuns.

## Pacotes por categoria

| Categoria | Conteúdo |
| --- | --- |
| `base` | Git, Vim, Neovim, fastfetch, firmware Linux, fonte Terminus para console, zram-generator e utilitários. O perfil Acer acrescenta headers para os kernels `linux` e `linux-zen`, Btrfs, Snapper e Btrfs Assistant. |
| `development` | Compiladores C/C++, CMake, depuradores, fnm, Rust/Cargo, .NET 10, Podman, DBeaver, Zed, Kleopatra para certificados e Bruno (AUR). |
| `desktop` | Xfce, Ly, libinput, Genmon para Caps/Num/Scroll Lock, rede, áudio PipeWire, temas, ícones, cursores, fontes Inter/Fira Code/Hack, Discord, Telegram, qBittorrent e, pela AUR, Brave, VSCodium e Spotify. O perfil Acer acrescenta Bluetooth. |
| `gaming` | Steam, GameMode, MangoHud e fontes; opcional. |
| `emulation` | RetroArch e Dolphin; PCSX2 e RPCS3 pela AUR; opcional. |

Sem categorias explícitas, o padrão é `base development desktop`. Ao informar categorias, **somente elas** entram no plano. Por exemplo, `packages plan development` não instala `base` nem `desktop`.

```bash
# Jogos e emuladores são selecionados separadamente:
./install.sh packages plan --profile acer-aspire-vx5-591g-54pg gaming emulation
./install.sh packages apply --profile acer-aspire-vx5-591g-54pg gaming emulation
```

`plan` funciona sem acesso de administrador e não altera nada; ele não resolve dependências nem calcula o que falta instalar. `apply` só roda no Arch Linux e como usuário comum. Antes de instalar, verifica a presença do auxiliar AUR quando necessário e de `[multilib]` quando `gaming` foi selecionado. Usa `sudo pacman -Syu --needed` para os pacotes oficiais e `paru` (ou `yay`, se for o único instalado) para a AUR. O processo é interativo: revise os PKGBUILDs e as transações apresentadas. Uma falha após o início pode deixar parte dos pacotes instalada; nesse caso, confira a saída antes de repetir `apply`.

### Preparar a AUR

`./install.sh aur apply` atualiza o Arch, instala `git` e `base-devel`, clona o [PKGBUILD do paru](https://aur.archlinux.org/packages/paru) em um diretório temporário e o mostra para revisão. O script só executa `makepkg -si` depois que você digitar `INSTALAR PARU`; o `makepkg` roda como usuário comum e pode instalar dependências oficiais, como Rust. Se o `paru` já estiver disponível, o comando termina sem reinstalá-lo. Não execute o configurador inteiro com `sudo`.

A AUR é mantida pela comunidade; os arquivos de construção precisam ser revisados em cada instalação ou atualização. O [guia da ArchWiki](https://wiki.archlinux.org/title/Arch_User_Repository) descreve esse processo. `aur plan` não altera o sistema.

Steam exige o repositório `[multilib]` habilitado e bibliotecas gráficas de 32 bits. O perfil Acer inclui as bibliotecas Intel; a variante NVIDIA 580xx será tratada junto do módulo de driver. O Proton é escolhido e gerenciado pela própria Steam; ele não é instalado como pacote separado aqui. Emulação de Xbox 360 permanece fora da lista automática até escolhermos uma implementação Linux adequada e testá-la neste hardware.

Ao aplicar `desktop`, o script tenta definir o Brave como navegador padrão na sessão gráfica. Ao aplicar `development`, instala a versão LTS do Node.js pelo fnm e a define como padrão; em Bash ou Zsh, cria uma cópia de segurança do arquivo de configuração antes de acrescentar a inicialização do fnm. A instalação do pacote `rust` inclui `rustc` e `cargo`. O .NET 10 inclui SDK e runtimes .NET e ASP.NET. As dependências declaradas pelos pacotes são resolvidas pelo gerenciador de pacotes.

## Visual leve do Xfce

A categoria `desktop` instala o grupo básico `xfce4` com Xorg e Ly, sem o grupo amplo `xfce4-goodies`. Inclui Docklike e Genmon no painel, o tema Materia-dark (GTK e bordas), ícones Papirus-Dark, cursores Capitaine, fonte Inter para a interface e Fira Code/Hack para programação. Esses pacotes são compartilhados por todos os computadores; resolução, detecção do touchpad e GPU continuam nos perfis de hardware.

Depois de instalar os pacotes, inicie uma sessão **X11 do Xfce** e execute `./install.sh xfce plan` seguido de `./install.sh xfce apply` como usuário comum. O comando faz backup das configurações atuais em `~/.local/state/archlinuxsetup/backups/`, aplica o tema e transforma o painel inferior em dock com ocultação inteligente. O **logo do Arch Linux** fica à esquerda da dock e abre o menu de aplicativos do Xfce; o ícone vem do pacote básico `filesystem` do Arch. Acrescenta ao painel superior um indicador discreto: **CAPS**, **NUM** e **SCR** ficam destacados quando ativos; passe o mouse para ver o estado completo. Se não existir painel superior, o comando cria um. O indicador usa o Genmon oficial e consulta `xset` uma vez por segundo. Aplicar novamente não cria outra dock, menu ou indicador. Os aplicativos favoritos podem ser fixados pelo menu de contexto do Docklike.

Depois de instalar os pacotes, execute `./install.sh system plan` e `./install.sh system apply`. O comando exige a sessão Xfce X11 instalada, copia `/etc/ly/config.ini` antes de editá-lo, oculta as entradas de shell, xinitrc e Wayland no Ly, habilita `ly@tty1.service` para o próximo boot e deixa tty2 livre para recuperação. Sessões X11 adicionais instaladas no futuro também aparecerão. O Ly pede o nome de usuário no login; a seleção de "perfil" é a seleção de sessão gráfica. Se outro gerenciador de login estiver habilitado, `system apply` para antes de fazer alterações e pede que você o desabilite.

O mesmo comando configura zram com zstd para metade da RAM e grava uma regra Xorg/libinput para ativar **rolagem natural em touchpads**, preservando a direção da roda de mouses externos. Com 8 GB instalados, o dispositivo lógico zram terá cerca de 4 GB; a RAM física consumida dependerá do conteúdo comprimido. Reinicie e confira com `zramctl` e `swapon --show`. Isso não cria swap em disco nem fornece hibernação. A regra do touchpad entra em vigor na próxima sessão X11; confirme em *Configurações → Mouse e touchpad* e com `xinput list-props` no dispositivo real. Se o archinstall já gerou configuração zram, revise possíveis arquivos adicionais em `/etc/systemd/zram-generator.conf.d/` antes de aplicar.

Para uma instalação nova, habilite `NetworkManager` após instalar `desktop`: `sudo systemctl enable --now NetworkManager`. No Acer, após confirmar o adaptador Bluetooth pelo diagnóstico, habilite `sudo systemctl enable --now bluetooth`. A instalação dos pacotes não troca automaticamente serviços de rede/áudio existentes.

## Adicionar outro computador

Crie uma pasta em `profiles/<identificador>/` com um `README.md` descrevendo o hardware conhecido e o que já foi validado nele. Adicione apenas os pacotes específicos da máquina em `packages/official/<categoria>.txt` ou `packages/aur/<categoria>.txt` dentro desse perfil. Use o mesmo nome das categorias compartilhadas. Cada arquivo contém **um nome de pacote por linha**; linhas vazias e comentários iniciados por `#` são aceitos.

```text
profiles/
  acer-aspire-vx5-591g-54pg/
    README.md
    packages/official/base.txt
    packages/official/gaming.txt
  outro-computador/
    README.md
    packages/official/base.txt
```

Para revisar um novo perfil, execute `./install.sh packages plan --profile outro-computador`. A seleção de perfil é manual; confirme o hardware real antes de criar módulos de driver, boot ou energia para ele.

## Prioridades para o Acer

1. **Diagnóstico real:** confirmar as GPUs, placa Wi-Fi/Bluetooth, áudio, driver carregado, dispositivo do touchpad, tipo de sessão e layout Btrfs. O proprietário confirmou o modelo VX5-591G-54PG com i5-7300HQ de 7ª geração, SSD NVMe de 256 GB adicionado e HD de 1 TB presente mas sem uso. Confirmar os caminhos e UUIDs no próprio notebook.
2. **Touchpad no Xfce:** a rolagem natural já é configurada como padrão pelo módulo `system`; confirmar no aparelho que libinput detecta o touchpad e testar toque para clicar, rolagem com dois dedos e rejeição da palma.
3. **Gráficos híbridos:** manter a GPU Intel para o desktop e usar a NVIDIA sob demanda com PRIME. Para a GTX 1050 (Pascal), verificar a série proprietária legada `nvidia-580xx-dkms`; o driver NVIDIA principal 590+ não oferece suporte a Pascal. Evitar automatizar a troca antes de confirmar GPU, kernel e pacotes instalados.
4. **Btrfs e Snapper:** verificar subvolumes, montagem de `/.snapshots`, configuração `root`, política de retenção e procedimento de recuperação. O **Btrfs Assistant**, já incluído no perfil Acer, oferece interface gráfica para criar, consultar e restaurar snapshots do Snapper; abra-o pelo menu de aplicativos do Xfce. `snap-pac` cria snapshots em transações do pacman quando a configuração Snapper existe. A integração para iniciar snapshots pelo Limine ainda precisa de configuração separada. Snapshot local não substitui backup externo.
5. **Notebook e programas:** configurar Wi-Fi/Bluetooth, áudio, teclas de função, suspensão, bateria e os aplicativos realmente usados. Escolher um único gerenciador de energia após medir o comportamento do aparelho.

O [perfil Acer](profiles/acer-aspire-vx5-591g-54pg/README.md) registra os pacotes de hardware já selecionados e os candidatos que dependem do diagnóstico. Não é necessário baixar drivers Windows da Acer para o Arch; funções específicas da marca podem usar o módulo `acer_wmi` incluído no kernel Linux.

## Instalação base planejada no archinstall

No Acer, o Arch será instalado no **SSD NVMe de 256 GB** com LUKS, Btrfs e compressão `zstd`, Snapper e Limine. Selecione **os dois kernels `linux` e `linux-zen` no archinstall**: o instalador prepara as entradas de boot para os kernels escolhidos. Mantenha `linux` como alternativa de boot e teste o Zen nos jogos; o ganho de desempenho depende do jogo e do hardware. Não selecione o HD de 1 TB como alvo no archinstall; ele será preparado depois, também com LUKS. A compressão Btrfs `zstd` e a swap zram com `zstd` são ajustes distintos.

Preferências da instalação: hostname `dev-machine`, usuários `root` e `dev` (este no grupo com acesso a `sudo`), espelhos brasileiros, timezone `America/Sao_Paulo`, locale `en_US.UTF-8`, teclado ABNT2 (escolha o mapa de console disponível no instalador), fonte `ter-v16n` do pacote `terminus-font`, PipeWire e NetworkManager. O Ly será habilitado pelo módulo `system` após instalar `desktop`; não é preciso selecionar outro gerenciador de login no instalador. Confira as opções oferecidas pela versão do archinstall antes de confirmar.

Após o primeiro boot, execute `./install.sh diagnose`. Confira que `/` está no Btrfs sobre LUKS, que `/.snapshots` está montado como esperado, que `snapper list-configs` mostra `root` e que o Limine oferece entradas funcionais para `linux` e `linux-zen`. O `pacman` atualiza os dois kernels; este projeto não instala um gerenciador gráfico de kernels. Os pacotes `linux-headers` e `linux-zen-headers` do perfil Acer preparam a compilação de módulos DKMS para ambos, necessária quando configurarmos a NVIDIA proprietária. `system apply` não altera Limine, desbloqueio LUKS do SSD ou subvolumes Btrfs.

## HD de 1 TB para dados e jogos

O HD secundário será reparticionado como **uma partição LUKS2 com ext4 dentro**. A formatação apaga todos os arquivos existentes; ela não faz apagamento seguro dos blocos antigos. Para evitar atingir o SSD, `storage apply` exige o perfil Acer e um caminho `/dev/disk/by-id/...`, confirma o modelo pelo firmware, confere que é um disco rotativo de cerca de 1 TB, recusa dispositivos montados e pede para digitar `APAGAR /dev/...` antes de qualquer alteração. Execute somente no Acer e confira a saída do plano no próprio notebook:

```bash
./install.sh storage plan
ls -l /dev/disk/by-id/
./install.sh storage apply --profile acer-aspire-vx5-591g-54pg --disk /dev/disk/by-id/ID_REAL_DO_HD
```

Substitua `ID_REAL_DO_HD` pelo identificador observado; não copie literalmente o exemplo. O `cryptsetup` pedirá uma senha nova do HD, separada da senha do SSD. O comando grava UUIDs em `/etc/crypttab` e `/etc/fstab`, com cópia de segurança desses arquivos, monta o volume em `/mnt/dados` e cria `Dados` e `Jogos` para `dev`. Em boots seguintes, o HD não será desbloqueado automaticamente: execute `./install.sh storage unlock --profile acer-aspire-vx5-591g-54pg` e digite a senha do HD. Depois, adicione `/mnt/dados/Jogos` como biblioteca na Steam. O comando de preparação recusa repetir a operação se já houver entradas para esse volume.

## Referências

- [ArchWiki: NVIDIA e famílias suportadas](https://wiki.archlinux.org/title/NVIDIA)
- [Arch Linux: fim do suporte Pascal no driver NVIDIA 590](https://archlinux.org/news/nvidia-590-driver-drops-pascal-support-main-packages-switch-to-open-kernel-modules/)
- [ArchWiki: NVIDIA Optimus](https://wiki.archlinux.org/title/NVIDIA_Optimus)
- [ArchWiki: Intel graphics](https://wiki.archlinux.org/title/Intel_graphics)
- [ArchWiki: libinput](https://wiki.archlinux.org/title/Libinput)
- [Xfce: plugin Genmon](https://docs.xfce.org/panel-plugins/xfce4-genmon-plugin/start)
- [Arch Linux: logo fornecido pelo pacote filesystem](https://archlinux.org/packages/core/any/filesystem/files/)
- [ArchWiki: Snapper](https://wiki.archlinux.org/title/Snapper)
- [ArchWiki: Ly](https://wiki.archlinux.org/title/Ly)
- [ArchWiki: zram](https://wiki.archlinux.org/title/Zram)
- [Arch Linux: Btrfs Assistant](https://archlinux.org/packages/extra/x86_64/btrfs-assistant/)
- [Arch Linux: Kleopatra](https://archlinux.org/packages/extra/x86_64/kleopatra/)
- [Arch: crypttab e opção noauto](https://man.archlinux.org/man/crypttab.5)
