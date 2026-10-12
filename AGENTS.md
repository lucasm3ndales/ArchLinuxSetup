# Instruções para agentes

Este repositório automatiza a pós-instalação do Arch Linux. Mantenha os comandos reutilizáveis, revisáveis por `plan` e seguros para repetir.

## Escopo

- `install.sh` expõe `diagnose`, `aur plan/apply`, `packages plan/apply`, `system plan/apply`, `xfce plan/apply` e `storage plan/apply/unlock`. Os módulos ficam em `modules/`.
- `system apply` configura apenas a rolagem natural de touchpads X11. Ly e zram ficam sob responsabilidade do archinstall. `xfce apply` configura o visual e o indicador das teclas Lock. `storage apply` prepara apenas o HD secundário do Acer após seleção explícita.
- O SSD é instalado pelo archinstall. Não há módulos prontos para boot, driver NVIDIA, outros ajustes do touchpad, energia ou recuperação/configuração Snapper. Não os descreva como implementados.
- Não crie diretórios vazios para funcionalidades futuras.

## Pacotes e perfis

- `packages/official/<categoria>.txt` e `packages/aur/<categoria>.txt` são listas comuns. `profiles/<identificador>/` contém somente dados e pacotes próprios da máquina. Hoje existe apenas `acer-aspire-vx5-591g-54pg`.
- Sem `--profile`, use somente as listas comuns. Nunca escolha um perfil automaticamente.
- Classifique pacotes pela função. Coloque dependências de GPU, CPU ou outro dispositivo no perfil. Prefira pacotes oficiais; use nomes exatos do Arch. Cada linha contém um pacote, sem argumentos de shell; linhas vazias e comentários iniciados por `#` são aceitos.
- As listas atuais partem das escolhas em `archinstall/user_configuration.json`. Antes de adicionar um pacote, confira se o archinstall já o instala diretamente, por um grupo selecionado ou como dependência. `xfce4-genmon-plugin` vem do grupo `xfce4-goodies` selecionado pelo perfil Xfce; `xfce apply` depende dele.
- Atualize a tabela de categorias do `README.md` ao mudar o conteúdo visível de uma lista.
- `plan` deve continuar sem alterações. Antes da primeira alteração, `apply` valida os pré-requisitos. Rode os scripts como usuário comum e use `sudo` somente nas ações privilegiadas.
- Preserve as perguntas do `pacman` e do auxiliar AUR. `aur apply` instala `paru` após revisão do PKGBUILD e nunca executa `makepkg` como root. `packages apply` prefere `paru`, mas aceita `yay` existente.

## Hardware e dados

- Obtenha diagnóstico no equipamento antes de aplicar ajustes de hardware. Descrições no perfil são hipóteses até a conferência local. Exija perfil explícito e valide sua compatibilidade antes de alterar driver, boot, energia, touchpad específico ou sistema de arquivos.
- A regra compartilhada de rolagem natural pode dispensar perfil porque usa `MatchIsTouchpad` no Xorg. Mantenha-a restrita a touchpads X11.
- Ofereça um plano legível, evite entradas duplicadas em execuções repetidas e faça backup antes de editar arquivos existentes. Separe partições, boot e recuperação Btrfs/Snapper da instalação de aplicativos.
- Não grave senhas, tokens, números de série/SNID ou diagnósticos completos no repositório. O modelo Acer confirmado é `acer-aspire-vx5-591g-54pg`.
- O SSD NVMe de 256 GB do Acer é alvo do archinstall. O HD de 1 TB é destinado a dados/jogos com LUKS2 e ext4. Para formatá-lo, exija `/dev/disk/by-id/...`, confirme modelo, partições e montagens no notebook e peça confirmação digitada. Nunca escolha `/dev/sdX` ou um disco apenas pela capacidade.
- Valide as opções do archinstall no primeiro boot; o JSON salvo não comprova que o SSD foi criptografado. As preferências estão no README do perfil.

## Xfce

- Temas, fontes e layout são comuns a todas as máquinas. `xfce apply` roda como usuário comum numa sessão X11 do Xfce; `xfce plan` permanece sem alterações.
- Faça backup antes de mudar Xfconf ou painéis. Repetir `apply` não deve duplicar painéis ou plugins.
- O indicador do painel superior usa `xfce4-genmon-plugin` e `modules/keylocks.sh`. O menu da dock inferior usa `/usr/share/pixmaps/archlinux-logo.svg`, fornecido por `filesystem`.

## Verificação

- Scripts Bash alterados: execute `bash -n`.
- Listas de pacotes: confira `./install.sh packages plan` com e sem o perfil Acer e teste a categoria alterada sozinha. Perfis e categorias inválidos devem ser recusados.
- Confira `xfce plan`, `system plan`, `storage plan` e `aur plan`, inclusive a rejeição de argumentos inválidos. Confirme que `plan` não altera o sistema.
- Não execute `packages apply`, `aur apply`, `system apply` ou `storage apply` na máquina de desenvolvimento. Teste a aplicação num Arch alvo. Para `xfce apply`, use uma sessão Arch/Xfce X11 real; nesta máquina, valide apenas a sintaxe e o plano.
