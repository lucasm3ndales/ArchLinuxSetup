# Instruções para agentes

Este repositório automatiza parte da pós-instalação do Arch Linux. Mantenha os comandos legíveis, revisáveis e seguros para repetir.

## Interface e escopo

- `archsetup` é a única interface pública. Os comandos são `link`, `check`, `doctor`, `aur`, `packages` e `desktop`. Use `--dry-run` para prévias sem alterações; não reintroduza `plan/apply`.
- `modules/packages.sh`, `modules/aur.sh` e `modules/doctor.sh` tratam pacotes, AUR e diagnóstico. `modules/desktop.sh` reúne os ajustes Xfce, LightDM/Slick Greeter e touchpad X11.
- O HD secundário será configurado manualmente. Não adicione formatação ou desbloqueio ao CLI sem um novo pedido explícito. O SSD é instalado pelo archinstall. Boot, driver NVIDIA, energia e recuperação de snapshots não estão implementados.
- Não crie diretórios vazios como marcadores de funcionalidades futuras.

## Pacotes e perfis

- `packages/official/<categoria>.txt` e `packages/aur/<categoria>.txt` são listas comuns. `profiles/<identificador>/` contém dados e pacotes próprios da máquina. Hoje existe apenas `acer-aspire-vx5-591g-54pg`.
- Sem `--profile`, use somente as listas comuns. Nunca escolha um perfil automaticamente.
- Classifique pacotes pela função. Coloque dependências de GPU, CPU ou outro dispositivo no perfil. Prefira pacotes oficiais; use nomes exatos do Arch. Cada linha contém um pacote, sem argumentos de shell; linhas vazias e comentários iniciados por `#` são aceitos.
- Não há configuração salva do archinstall no repositório. Antes de adicionar um pacote, confira no sistema alvo se o archinstall já o instala diretamente, por um grupo ou como dependência. `xfce4-verve-plugin` vem de `xfce4-goodies`; o grupo de pacotes `xfce` depende dele. `xfce4-kbdleds-plugin` fica na lista AUR `desktop`.
- Atualize a tabela de categorias do `README.md` ao mudar uma lista.
- `--dry-run` não deve alterar arquivos, pacotes ou configurações. Antes da primeira alteração, valide os pré-requisitos. Rode os scripts como usuário comum e use `sudo` somente quando necessário.
- Sem `--dry-run`, `aur` e `packages` instalam com `--noconfirm`, sem frases de confirmação adicionais. `aur` mostra o PKGBUILD e lista os demais arquivos antes de executar `makepkg`; nunca execute `makepkg` como root. `packages` prefere `paru`, mas aceita `yay` existente. Se nenhum auxiliar estiver disponível e houver pacotes AUR selecionados, informe a dependência e ofereça executar `archsetup aur` antes de instalar. Não leia nem armazene a senha do usuário; use o cache temporário do próprio `sudo`.
- `aur` instala `rustup` antes da compilação apenas se não houver `rust` ou `rustup`. `packages development` usa `rust` instalado e omite `rustup` da seleção nesse caso; nunca remova `rust` automaticamente. Inicialize a toolchain estável apenas quando `rustup` for o provedor escolhido. Verifique a seleção também em `--dry-run`.
- `packages development` instala Docker Engine e Compose, adiciona o usuário atual ao grupo `docker` somente se necessário, inicia `docker.service` e o habilita para iniciar no boot. Faça essas ações de modo idempotente. Avise que a nova associação ao grupo exige novo login.

## Hardware e dados

- Obtenha diagnóstico no equipamento antes de aplicar ajustes de hardware. Descrições no perfil são hipóteses até conferência local. Exija perfil explícito e valide sua compatibilidade antes de alterar driver, boot, energia, touchpad específico ou sistema de arquivos.
- A regra compartilhada de rolagem natural usa `MatchIsTouchpad` no Xorg e não exige perfil. O toque para clicar do Acer exige `--profile` e validação DMI; mantenha as duas regras restritas a touchpads X11.
- Faça backup antes de editar arquivos existentes. Repetir comandos não deve duplicar entradas ou plugins.
- Não grave senhas, tokens, números de série/SNID ou diagnósticos completos no repositório. O modelo Acer confirmado é `acer-aspire-vx5-591g-54pg`.
- O SSD NVMe de 256 GB do Acer é alvo do archinstall. O HD de 1 TB será preparado manualmente para dados/jogos com LUKS2 e ext4. Nunca infira `/dev/sdX` ou escolha disco apenas pela capacidade. Valide o dispositivo por `/dev/disk/by-id/...`, modelo, partições e montagens no notebook.
- Valide as opções do archinstall no primeiro boot; a seleção de LUKS no instalador não prova que a instalação ficou criptografada.
- Para a nova instalação do Acer, o plano é LUKS, Timeshift em Btrfs com GRUB2, `@` e `@home`. Uma instalação anterior entrou em loop de boot sem a combinação UKI e **Install to removable location**; não se sabe qual opção foi decisiva. Recomende manter **Install to removable location** ativado; UKI desativado simplifica GRUB e snapshots, mas ainda precisa ser testado no notebook. O projeto não configura boot ou snapshots, e o perfil não instala Snapper, `snap-pac` nem Btrfs Assistant. Confira a montagem EFI antes de instalar: com `/boot` na partição EFI, os arquivos de boot ficam fora dos snapshots de `@`; uma restauração após atualização de kernel requer verificação adicional. GRUB comum não lê LUKS2 com Argon2 em um `/boot` criptografado. Não descreva rollback completo como validado antes de testá-lo no notebook.

## Xfce

- Temas, fontes e layout são comuns a todas as máquinas. `desktop` roda como usuário comum numa sessão X11 do Xfce e configura Xfce, LightDM/Slick Greeter e touchpad. Se faltarem pacotes da categoria `desktop` necessários ao comando, ofereça executar `archsetup packages desktop` antes de alterar arquivos; não instale ou substitua Xfce ou o gerenciador de login automaticamente.
- Faça backup antes de mudar Xfconf ou painéis. Repetir o comando não deve duplicar painéis ou plugins.
- O painel superior usa tarefas nativas agrupadas por aplicativo e limitadas ao espaço atual, busca Verve, espaços de trabalho, relógio com dia/mês e hora e `xfce4-kbdleds-plugin`. Reutilize o relógio existente e não recrie a dock inferior. Ao migrar, remova apenas o Genmon identificado como indicador antigo do projeto e preserve o script anterior no backup. O menu usa `/usr/share/pixmaps/archlinux-logo.svg`, fornecido por `filesystem`.
- Reutilize o Whisker Menu existente com o logo do Arch; se ele não existir, use o menu de aplicativos. Não deixe dois menus no painel.
- A configuração ativa usa Nordic escuro para GTK/xfwm4, Nordzy-dark para ícones, Nordzy-cursors para cursor e Inter para a interface. Fira Code é a única fonte extra para código. As variantes Nordic escura/claras ficam instaladas, mas não são aplicadas automaticamente.
- A configuração de login em `desktop` exige LightDM com Slick Greeter instalado e LightDM habilitado. Use Nordic, Nordzy-dark, Nordzy-cursors e Inter também na tela de login. Preserve as demais opções de `/etc/lightdm/slick-greeter.conf`, faça backup antes de editar e não reinicie a sessão durante a aplicação.
- `ksuperkey` pertence à categoria AUR `desktop`; `desktop` configura `Alt+F1` para o menu de aplicativos e inicia o daemon uma vez, com autostart no Xfce. Preserve atalhos existentes que conflitem e faça backup antes de alterar Xfconf ou o autostart.

## Verificação

- Execute `bash -n` nos scripts Bash alterados.
- Para listas de pacotes, confira `archsetup packages --dry-run` com e sem o perfil Acer e teste as categorias alteradas isoladamente. Perfis e categorias inválidos devem ser recusados.
- Confira `archsetup aur --dry-run`, `archsetup desktop --dry-run` e `archsetup link --dry-run`, inclusive a rejeição de argumentos inválidos. Confirme que as prévias não alteram arquivos. Teste os fluxos de dependência ausente, aceita e recusada com comandos simulados; nunca instale pacotes na máquina de desenvolvimento.
- Confira `archsetup check` e `archsetup check --profile acer-aspire-vx5-591g-54pg`. Em máquina incompatível, o perfil deve falhar sem alterações; só use `packages` com perfil e sem `--dry-run` no Acer real.
- Não execute `packages`, `aur` ou `desktop` sem `--dry-run` na máquina de desenvolvimento. Teste a aplicação num Arch alvo; para `desktop`, use uma sessão X11 real do Xfce com LightDM/Slick Greeter.
