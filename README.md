# ArchSetup

ArchSetup é uma ferramenta de pós-instalação para Arch Linux. Ela instala pacotes por categoria, aplica uma configuração pronta para Xfce e permite acrescentar ajustes específicos de um equipamento por meio de perfis.

O ArchSetup parte de um sistema já instalado. Ele não instala o Arch Linux nem configura discos, criptografia, boot ou snapshots.

## Antes de começar

- Use o Arch Linux com acesso à internet e `sudo`.
- Execute os comandos como usuário comum, sem `sudo` na frente de `archsetup` ou `./install.sh`.
- Para usar `desktop` ou `all`, tenha o Xfce instalado e entre em uma sessão X11 do Xfce. O ArchSetup configura o ambiente, mas não instala o Xfce.

## Instalação

Instale o Git, clone o repositório e execute o instalador:

```bash
sudo pacman -Syu --needed git
git clone https://github.com/lucasm3ndales/ArchLinuxSetup.git
cd ArchLinuxSetup
./install.sh
```

O instalador prepara o que for necessário, compila a ferramenta e disponibiliza o comando `archsetup`. Abra um novo terminal e confira a instalação:

```bash
archsetup doctor
```

Mantenha a pasta do repositório no lugar: o comando instalado aponta para ela.

## Primeiro uso

Para conferir o sistema e ver o que uma configuração completa faria:

```bash
archsetup doctor
archsetup all --dry-run
```

Se quiser instalar as categorias padrão e configurar o Xfce, execute:

```bash
archsetup all
```

`all` executa `aur`, `pkg` e `desktop`, nessa ordem. Ele exige uma sessão X11 do Xfce desde o início. Para escolher somente algumas categorias, use `pkg`:

```bash
archsetup pkg base development --dry-run
archsetup pkg base development
archsetup pkg gaming
```

Você pode executar uma categoria depois das outras ou repetir um comando. Pacotes já instalados são ignorados; uma nova execução de `pkg` também pode atualizar pacotes do sistema.

## Comandos

| Comando | O que faz |
| --- | --- |
| `archsetup doctor` | Confere o sistema e mostra avisos, sem alterá-lo. |
| `archsetup profiles` | Lista os perfis disponíveis. |
| `archsetup aur` | Instala o auxiliar `paru`, usado para pacotes da AUR. |
| `archsetup pkg [CATEGORIA...]` | Instala pacotes oficiais e da AUR das categorias escolhidas. |
| `archsetup desktop` | Aplica a configuração do Xfce, do painel e do touchpad. |
| `archsetup all` | Executa `aur`, `pkg` e `desktop`. |
| `archsetup restore` | Recupera configurações anteriores com backup feito pelo ArchSetup. |
| `archsetup link` | Cria ou atualiza o comando `archsetup`; o instalador já faz isso. |
| `archsetup unlink` | Remove o comando instalado, sem desfazer as configurações. |
| `archsetup help [COMANDO]` | Mostra a ajuda geral ou de um comando. |

Se `pkg` precisar de pacotes da AUR e não encontrar um auxiliar, ele oferecerá executar `aur` antes de continuar. `desktop` também pode oferecer a instalação das categorias `desktop` e `theme` quando faltarem pacotes necessários.

## Opções

| Opção | Onde usar | Efeito |
| --- | --- | --- |
| `--dry-run` | `link`, `unlink`, `restore`, `aur`, `pkg`, `desktop`, `all` | Mostra uma prévia sem alterar o sistema. |
| `--profile NOME` | `doctor`, `pkg`, `desktop`, `all` | Seleciona um perfil explicitamente. |
| `--verbose` | `doctor` | Mostra um diagnóstico mais detalhado. |
| `--packages` | `restore` | Inclui a remoção de pacotes registrados como novos pelo ArchSetup. |
| `--help` ou `-h` | Comandos | Mostra a ajuda do comando. |

## Categorias de pacotes

`archsetup pkg` sem categoria usa `base development apps theme desktop`. `gaming` e `emulation` são opcionais e precisam ser solicitadas pelo nome. O comando `all` usa as mesmas categorias padrão.

| Categoria | Finalidade |
| --- | --- |
| `base` | Ferramentas gerais de terminal. |
| `development` | Ferramentas de desenvolvimento e containers. |
| `apps` | Aplicativos do dia a dia. |
| `theme` | Fontes, tema, ícones e cursor. |
| `desktop` | Utilitários necessários à configuração do Xfce. |
| `gaming` | Jogos e ferramentas relacionadas. |
| `emulation` | Emuladores. |

Cada categoria pode reunir pacotes dos repositórios [oficiais](packages/official/) e da [AUR](packages/aur/). Abra essas pastas para ver a lista exata de pacotes antes de instalar.

`pkg desktop` instala utilitários para o ambiente; `archsetup desktop` aplica a configuração. Nenhum dos dois instala o Xfce. Para a categoria `gaming`, habilite `[multilib]` no pacman antes de instalar.

## Perfis

Um perfil acrescenta pacotes ou ajustes para um equipamento específico. Sem `--profile`, o ArchSetup usa apenas as categorias compartilhadas. O perfil `generic` representa explicitamente essa mesma escolha; perfis de hardware são aplicados somente quando você os seleciona e o equipamento é compatível.

```bash
archsetup profiles
archsetup doctor
archsetup pkg --profile NOME --dry-run
archsetup pkg --profile NOME
```

`doctor` pode sugerir um perfil compatível, mas não o seleciona automaticamente. O mesmo `--profile NOME` também funciona com `desktop` e `all`. Veja os perfis disponíveis em [profiles](profiles/).

## Configuração do desktop

`archsetup desktop` configura o Xfce com Nordic, ícones Zafiro Dark, cursor Hackneyed e um painel superior com menu, busca e espaços de trabalho numerados. Também ajusta atalhos e o touchpad. Se LightDM com Slick Greeter estiver habilitado, aplica o visual à tela de login. Se Ly estiver habilitado, ativa a animação Matrix e mostra data e hora no login. Outros gerenciadores são preservados.

Execute o comando dentro de uma sessão X11 do Xfce:

```bash
archsetup desktop --dry-run
archsetup desktop
```

O ArchSetup guarda cópias das configurações que altera para permitir a restauração.

## Restaurar ou remover

Para conferir e recuperar configurações anteriores, saia da sessão gráfica do Xfce e use um TTY:

```bash
archsetup restore --dry-run
archsetup restore
```

Para incluir os pacotes, revise a prévia antes de executar a remoção:

```bash
archsetup restore --packages --dry-run
archsetup restore --packages
```

Essa opção considera somente os pacotes registrados como novos pelo ArchSetup. Ela não reverte atualizações de pacotes que já existiam nem restaura o sistema inteiro. Para remover apenas o comando e manter as configurações, use `archsetup unlink`.

## Atualizar o ArchSetup

Na pasta do repositório:

```bash
git pull
./install.sh
```

O instalador atualiza o sistema, recompila a ferramenta e mantém o comando apontando para a versão atual. Se quiser apenas consultar a sintaxe disponível, use `archsetup help` ou `archsetup help pkg`.
