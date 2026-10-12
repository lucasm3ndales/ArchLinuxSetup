# ArchLinuxSetup — instruções para agentes

## Objetivo e estado atual

Este repositório automatiza a **pós-instalação do Arch Linux** para diferentes computadores. O foco inicial é instalar aplicativos e evoluir, com base em diagnóstico real, para ajustes de hardware e de ambiente gráfico. O projeto deve ser reutilizável e seguro para executar mais de uma vez.

Hoje, `install.sh` oferece `diagnose`, `aur plan/apply`, `packages plan/apply`, `xfce plan/apply`, `system plan/apply` e `storage plan/apply/unlock`. `system apply` configura Ly, zram e rolagem natural do touchpad no Xorg; `xfce apply` também instala um indicador das teclas Lock no painel. `storage apply` prepara somente o HD secundário após seleção explícita. Não existe instalador do SSD, boot, driver NVIDIA, demais ajustes do touchpad, energia ou configuração Snapper. Não descreva essas etapas como prontas.

## Organização

- `install.sh` é a interface de comandos; `modules/` contém a implementação.
- `packages/official/<categoria>.txt` e `packages/aur/<categoria>.txt` contêm pacotes compartilhados por todos os computadores.
- `profiles/<identificador>/` contém somente dados e pacotes específicos de uma máquina. Atualmente há apenas `acer-aspire-vx5-591g-54pg`.
- A ausência de `--profile` significa usar somente as listas compartilhadas. Nenhum perfil de hardware é escolhido automaticamente.
- O perfil Acer confirmado é `acer-aspire-vx5-591g-54pg`; não grave SNID ou número de série no repositório ou em diagnósticos compartilhados.
- Crie novas pastas somente quando houver arquivos ou módulos que as utilizem; não mantenha diretórios vazios como marcadores de funcionalidades futuras.

## Ao modificar pacotes

- Escolha a categoria pela função do programa, não pelo computador em que ele será usado. Coloque pacotes dependentes de GPU, CPU ou outro dispositivo no perfil correspondente.
- Use os nomes exatos dos pacotes do Arch. Prefira repositórios oficiais quando disponíveis; registre pacotes AUR em `packages/aur/` ou em `profiles/<identificador>/packages/aur/`, conforme o alcance.
- Mantenha um pacote por linha, sem argumentos de shell. Linhas vazias e comentários que começam com `#` são aceitos pelo leitor.
- Atualize a tabela de categorias do `README.md` quando mudar o conteúdo visível de uma categoria.
- Preserve `plan` como operação sem alterações. `apply` deve validar pré-requisitos antes da primeira alteração, operar como usuário comum e pedir `sudo` apenas para ações que exigem privilégio.
- Não suprima as perguntas do `pacman` ou do auxiliar AUR. A instalação de AUR exige revisão dos PKGBUILDs pelo usuário.
- Para uma instalação nova, `aur apply` prepara o `paru` a partir do PKGBUILD da AUR. Preserve a etapa de revisão antes de `makepkg`; nunca rode `makepkg` como root. `packages apply` prefere `paru`, mas continua aceitando `yay` se já estiver instalado.

## Ao criar configurações de hardware

- Primeiro obtenha diagnóstico da máquina alvo. O hardware descrito em um perfil é informação inicial, a confirmar no equipamento real.
- Exija seleção explícita do perfil e valide sua compatibilidade antes de alterar driver, boot, energia, ajustes específicos do touchpad ou sistema de arquivos. A rolagem natural compartilhada pode ser aplicada sem perfil porque a regra Xorg se limita a dispositivos identificados como touchpad.
- Disponibilize um plano legível para revisão e permita repetir a operação sem duplicar entradas. Faça cópia de segurança antes de editar arquivos existentes.
- Trate partições, boot e recuperação Btrfs/Snapper em módulos próprios. Não misture essas mudanças com a instalação de aplicativos.
- Não grave senhas, tokens, identificadores pessoais ou saídas completas de diagnóstico no repositório.
- O SSD de 256 GB do Acer é alvo do archinstall; o HD de 1 TB presente será apagado separadamente para dados/jogos, com LUKS2 e ext4. Nunca inferir `/dev/sdX` ou formatar por capacidade apenas. Exigir caminho estável by-id, confirmar modelo, partições e montagens no equipamento real e solicitar confirmação digitada antes da formatação.
- As preferências declaradas para archinstall estão no README e no perfil Acer. Validar no primeiro boot em vez de presumir que o instalador configurou tudo corretamente.

## Ao modificar o visual do Xfce

- Deixe temas, fontes e layout independentes do hardware; use o perfil apenas para dispositivos e configurações específicas.
- `xfce plan` deve continuar sem alterações. `xfce apply` só deve operar em sessão X11 do Xfce e como usuário comum.
- Salve a configuração anterior antes de alterar canais Xfconf ou painéis. Evite criar painéis ou plugins duplicados ao repetir o comando.
- O indicador usa `xfce4-genmon-plugin` e `modules/keylocks.sh`; mantenha sua configuração no painel superior. O menu de aplicativos na dock inferior usa o logo `/usr/share/pixmaps/archlinux-logo.svg` do pacote `filesystem` do Arch. A regra de rolagem natural em `system.sh` deve continuar restrita a touchpads X11 via `MatchIsTouchpad`.
- Não presuma que o notebook já executa Arch nesta máquina de desenvolvimento; valide a sintaxe localmente e teste a aplicação em uma sessão Arch/Xfce real.

## Verificação mínima

- Para scripts Bash, execute `bash -n` nos arquivos alterados.
- Para listas de pacotes, confira `./install.sh packages plan` com e sem o perfil Acer e teste a categoria alterada isoladamente.
- Confirme que `plan` não altera o sistema e que perfis/categorias inválidos são recusados.
- Confira também `./install.sh xfce plan` e rejeição de argumentos inválidos.
- Confira `./install.sh system plan`, `./install.sh storage plan` e rejeição de argumentos inválidos. Nunca teste `system apply` ou `storage apply` na máquina de desenvolvimento.
- Confira `./install.sh aur plan` e rejeição de argumentos inválidos. Nunca execute `aur apply` na máquina de desenvolvimento.
- Não execute `packages apply` na máquina de desenvolvimento para testar mudanças de documentação ou listas; use um Arch alvo apropriado para testes de instalação.

