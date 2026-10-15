# Instruções para agentes

Este repositório automatiza parte da pós-instalação do Arch Linux. Mantenha os comandos legíveis, revisáveis e seguros para repetir.

## Interface e escopo

- `archsetup` é a interface pública da CLI. O único shell script de entrada é `install.sh`, usado para instalar os pré-requisitos, compilar o binário Rust e executar `link`; não reintroduza módulos ou comandos Bash. `link` aponta `~/.local/bin/archsetup` para `target/release/archsetup`. `src/cli.rs` interpreta os comandos, `src/commands/` os executa, e `src/project.rs` localiza o repositório. Todos os comandos públicos da CLI têm implementação Rust. Use `--dry-run` para prévias sem alterações; não reintroduza `plan/apply` nem os nomes antigos `install`, `uninstall` ou `packages`.
- Mensagens exibidas pela CLI, pelo instalador e pelas descrições de perfis devem estar em inglês. Atualize os testes de saída ao mudar esses textos.
- O Rust usa uma arquitetura hexagonal enxuta em um crate. A CLI e os adaptadores dependem dos casos de uso, nunca o contrário. Defina portas pequenas por caso de uso para leitura ou alteração externa; mantenha regras e decisões em `src/application/`. Não crie crates ou traits sem necessidade. `doctor` resumido e o planejamento de `link` já usam essa divisão; o diagnóstico detalhado ainda está em migração.
- `src/adapters/desktop/` aplica os ajustes Xfce, Ly e touchpad X11. `restore` preserva o comando instalado; `unlink` remove somente o link e o PATH registrado. O código Bash legado foi removido; um link antigo para o launcher pode ser atualizado com `./target/release/archsetup link` após compilar.
- O ArchSetup não configura discos, partições, boot, energia nem recuperação de snapshots. O perfil Acer pode selecionar o driver legado NVIDIA na categoria opcional `gaming`, após conferir conflitos e headers; não remova drivers automaticamente. Não adicione formatação ou desbloqueio ao CLI sem um novo pedido explícito.
- Não crie diretórios vazios como marcadores de funcionalidades futuras.

## Pacotes e perfis

- `packages/official/<categoria>.txt` e `packages/aur/<categoria>.txt` são listas comuns. As categorias são `base`, `development`, `apps`, `theme`, `desktop`, `gaming` e `emulation`; o padrão sem categoria é `base development apps theme desktop`. Cada `profiles/<identificador>/profile.toml` contém metadados, pacotes extras e ajustes suportados. Existem `generic` e `acer-aspire-vx5-591g`.
- Sem `--profile`, use somente as listas comuns. `generic` é a escolha explícita equivalente, sem extras de hardware. `doctor` pode sugerir perfis por fabricante e modelo DMI, mas nunca os escolhe automaticamente. Perfis específicos exigem `match_vendor` e `match_dmi` do modelo base.
- Mantenha o esquema TOML documentado no README. Recuse campos desconhecidos; não execute comandos arbitrários de um perfil. Headers entram em `gaming` quando o perfil seleciona `nvidia-580xx-dkms`, conforme os kernels instalados; `base` não instala headers sem DKMS. Não reinstale utilitários de disco escolhidos no archinstall como parte de `base`.
- Classifique pacotes pela função. Coloque dependências de GPU, CPU ou outro dispositivo no perfil. Prefira pacotes oficiais; use nomes exatos do Arch. Cada linha contém um pacote, sem argumentos de shell; linhas vazias e comentários iniciados por `#` são aceitos.
- Não há configuração salva do archinstall no repositório. Antes de adicionar um pacote, confira no sistema alvo se o archinstall já o instala diretamente, por um grupo ou como dependência. `xfce4-verve-plugin` vem de `xfce4-goodies`; o grupo de pacotes `xfce` depende dele. `xfce4-kbdleds-plugin` fica na lista AUR `desktop`.
- Atualize a tabela de categorias do `README.md` ao mudar uma lista.
- `--dry-run` não deve alterar arquivos, pacotes ou configurações. Antes da primeira alteração, valide os pré-requisitos. Rode os comandos como usuário comum e use `sudo` somente quando necessário.
- Sem `--dry-run`, `aur` e `pkg` instalam com `--noconfirm`, sem frases de confirmação adicionais. `aur` mostra o PKGBUILD e lista os demais arquivos antes de executar `makepkg`; nunca execute `makepkg` como root. `pkg` prefere `paru`, mas aceita `yay` existente. Se nenhum auxiliar estiver disponível e houver pacotes AUR selecionados, informe a dependência e ofereça executar `archsetup aur` antes de instalar. Não leia nem armazene a senha do usuário; use o cache temporário do próprio `sudo`.
- `aur` instala `rustup` antes da compilação apenas se não houver `rust` ou `rustup`. `pkg development` usa `rust` instalado e omite `rustup` da seleção nesse caso; nunca remova `rust` automaticamente. Inicialize a toolchain estável apenas quando `rustup` for o provedor escolhido. Verifique a seleção também em `--dry-run`.
- `pkg development` instala Docker Engine e Compose, adiciona o usuário atual ao grupo `docker` somente se necessário, inicia `docker.service` e o habilita para iniciar no boot. Faça essas ações de modo idempotente. Avise que a nova associação ao grupo exige novo login.
- `aur` e `pkg` registram o inventário de pacotes antes de instalar e os pacotes novos após cada execução, inclusive em falhas parciais. `restore --packages` remove somente os novos registrados, após validar dependências com `pacman -R --print`; não use `-Rns` nem infira pacotes a partir das listas do repositório. `restore` sem essa opção não remove pacotes; `unlink` nunca remove pacotes.

## Hardware e dados

- Obtenha diagnóstico no equipamento antes de aplicar ajustes de hardware. Descrições no perfil são hipóteses até conferência local. Exija perfil explícito e valide sua compatibilidade antes de alterar driver, boot, energia, touchpad específico ou sistema de arquivos.
- A regra compartilhada de rolagem natural usa `MatchIsTouchpad` no Xorg e não exige perfil. O toque para clicar exige `touchpad_tapping = true` no perfil e validação DMI; mantenha as duas regras restritas a touchpads X11.
- Faça backup antes de editar arquivos existentes. Repetir comandos não deve duplicar entradas ou plugins.
- Não grave senhas, tokens, números de série/SNID ou diagnósticos completos no repositório. `acer-aspire-vx5-591g` representa o modelo base; não presuma que todas as variantes tenham os mesmos componentes.
- Nunca infira um disco por `/dev/sdX` ou capacidade. Qualquer documentação sobre discos ou restauração deve exigir verificação no equipamento e não prometer um rollback completo sem teste.

## Xfce

- Temas, fontes e layout são comuns a todas as máquinas. Xfce é o padrão atual e o único ambiente suportado por `desktop` e `all`; ambos rodam como usuário comum numa sessão X11 do Xfce. Exija `xfce4-session` e `xfce4-panel` instalados e a sessão Xfce ativa antes de qualquer alteração; `all` exige isso antes de executar `aur` ou `pkg`, e `doctor` informa a condição como aviso. Se faltarem pacotes das categorias `desktop` ou `theme` necessários ao comando, ofereça executar `archsetup pkg desktop theme` antes de alterar arquivos. Xfce é pré-requisito instalado pelo archinstall ou manualmente. Preserve Ly e outros gerenciadores de login; não os substitua automaticamente.
- Faça backup antes de mudar Xfconf ou painéis. Repetir o comando não deve duplicar painéis ou plugins.
- `restore` deve usar o primeiro backup anterior às alterações, preservar arquivos sem origem comprovada e restaurar os XMLs do Xfce somente fora da sessão gráfica.
- O painel superior usa tarefas nativas agrupadas por aplicativo e limitadas ao espaço atual, busca Verve, espaços de trabalho, relógio com dia/mês e hora e `xfce4-kbdleds-plugin`. Reutilize o relógio existente e não recrie a dock inferior. Ao migrar, remova apenas o Genmon identificado como indicador antigo do projeto e preserve o script anterior no backup. O menu usa `/usr/share/pixmaps/archlinux-logo.svg`, fornecido por `filesystem`.
- Reutilize o Whisker Menu existente com o logo do Arch; se ele não existir, use o menu de aplicativos. Não deixe dois menus no painel.
- A configuração ativa usa Nordic para GTK/xfwm4, Zafiro Dark (`zafiro-dark`) para ícones, Hackneyed para cursor e Inter para a interface. Fira Code é a fonte extra para código. Configure também `~/.Xresources` e `~/.icons/default/index.theme`, preservando as demais opções e fazendo backup antes de editar.
- Com Ly habilitado, `desktop` ativa `animation = matrix` e `clock = %d/%m/%Y %H:%M` em `/etc/ly/config.ini`. Preserve as demais opções, faça backup antes de editar e não reinicie a sessão durante a aplicação. Outros gerenciadores permanecem sem alterações.
- `ksuperkey` pertence à categoria AUR `desktop`; `desktop` configura `Alt+F1` para o menu de aplicativos e inicia o daemon uma vez, com autostart no Xfce. Preserve atalhos existentes que conflitem e faça backup antes de alterar Xfconf ou o autostart.

## Verificação

- Ao alterar Rust, execute `cargo fmt --check`, `cargo test` e `cargo build --release`. Teste o binário com um repositório e HOME temporários; nenhum teste deve instalar pacotes na máquina de desenvolvimento.
- Ao alterar `install.sh`, confira `bash -n install.sh` e sua saída em ambiente sem Arch; não execute a instalação real na máquina de desenvolvimento.
- Após alterar o código, recompile o binário Rust; não restaure o launcher nem os módulos Bash.
- Para listas de pacotes, confira `archsetup pkg --dry-run` com e sem o perfil Acer e teste as categorias alteradas isoladamente. Perfis e categorias inválidos devem ser recusados.
- Confira `archsetup aur --dry-run`, `archsetup desktop --dry-run` e `archsetup link --dry-run`, inclusive a rejeição de argumentos inválidos. Confirme que as prévias não alteram arquivos. Teste os fluxos de dependência ausente, aceita e recusada com comandos simulados; nunca instale pacotes na máquina de desenvolvimento.
- Confira `archsetup unlink --dry-run` e `archsetup restore --packages --dry-run`; teste a criação e remoção do link com um HOME temporário. Nunca execute a restauração real na máquina de desenvolvimento.
- Confira `archsetup doctor`, `archsetup doctor --verbose` e `archsetup doctor --profile acer-aspire-vx5-591g`. Em máquina incompatível, o perfil deve falhar sem alterações; só use `pkg` com perfil e sem `--dry-run` num equipamento compatível e conferido.
- Não execute `pkg`, `aur` ou `desktop` sem `--dry-run` na máquina de desenvolvimento. Teste a aplicação num Arch alvo; para `desktop`, use uma sessão X11 real do Xfce com o gerenciador de login configurado.
