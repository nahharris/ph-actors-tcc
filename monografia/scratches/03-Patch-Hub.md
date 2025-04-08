# Patch-Hub

A prova de conceito da arquitetura proposta será a reimplementação de um programa de software livre chamado Patch-Hub. O Patch-Hub é uma aplicação de interface de texto (TUI) pensada para auxiliar desenvolvedores do Kernel Linux a gerenciarem revisões de contribuições (chamadas de _patches_). Alguns dos principais recursos do Patch-Hub que devem ser replicados na reimplementação são:

- Acessar diferentes listas de email (onde os patches são enviados)
- Acessar os detalhes dos patches
- Oferecer uma vizualização do conteúdo do patch
- Permitir que um patch seja aplicado a um repositório local
- Permitir que um patch seja marcado como "revisado"
- Ter configurações personalizáveis
- Um logger para registrar eventos e erros

Para dar suporte a tais recursos, é necessário implementar as seguintes capacidades:

- Interagir com o sistema de arquivos
- Interagir com as variáveis de ambiente
- Interagir com a rede
- Interagir com o terminal
- Invocar programas externos
- Gerenciar configurações

Podemos entender essas capacidades e recursos como atores e assim dar uma intuição inicial da arquitetura do projeto:

- **Model**: gerencia o estado global da aplicação
- **View**(-> **Terminal**): cuida de renderização das telas

    - `Render`: renderiza alguma tela

- **Controller**(-> **Model**): cuida de executar ações baseadas nas entradas do usuário 

    - `Chord`: um conjunto de teclas pressionado ao mesmo tempo

- **Sys**: ator responsável por interagir com o sistema (arquivos e variáveis de ambiente)

    - `[Set|Unset|Get]Env`: interage com variáveis de ambiente
    - `[Open|Close|Remove]File`: interage com arquivos
    - `ReadDir`: lê conteúdo de um diretório 

- **Terminal**: ator responsável por interações baixo-nível com o terminal

    - `Attach`/`Detach`: assume/restaura controle do terminal
    - `Draw`: desenha uma primitiva no terminal

- **Net**: ator responsável por interações baixo-nível com a rede

    - `Send`: envia uma request

- **Logger**: faz log dos eventos que acontecem durante a execução
  
    - `Debug`: log de debug
    - `Info`: log de informação
    - `Warn`: log de aviso
    - `Error`: log de erro

- **Config**: gerencia as opções de configurações

    - Setters e getters
    - `Save`: salva a configuração atual
    - `Load`: carrega a configuração a partir do arquivo
    - `Override`: sobreescreve valores usando variáveis de ambiente

- **Lore**(-> **Net**): responsável por chamadas à API do Lore

    - `Lists`: obtém as listas de email
    - `Page`: obtém uma página de patches de uma lista
    - `Details`: obtém os detalhes de um patch
