# Fundamentação Teórica

## O que é o Modelo de Atores

> De acordo com Hewitt em Actor Model of Computation: Scalable Robust Information Systems

O Modelo de Atores constitui uma arquitetura concebida para computação concorrente distribuída. Sistemas são especificados em termos de _atores_ que coexistem de forma autônoma e interagem mediante troca de mensagens. Um sistema modelado com atores apresenta três componentes fundamentais:

### Atores

Atores são unidades computacionais autônomas que gerenciam seu próprio estado interno, operando sem compartilhamento de espaço de memória. Consequentemente, as interações entre atores são feitas exclusivamente através de mensagens. Ao receber uma mensagem, um ator pode:

- Instanciar novos atores
- Transmitir mensagens para outros atores conhecidos
- Definir o comportamento a ser adotado no processamento da próxima mensagem recebida

Devido à sua natureza autônoma, os atores não necessitam estar vinculados à mesma _thread_ ou à mesma máquina. Portanto, são necessárias abstrações suficientemente robustas para viabilizar as interações entre os atores.

### Mensagens

Mensagens consistem em estruturas de dados transmitidas entre atores, sendo que cada ator possui capacidade para processar apenas determinados tipos de mensagens. É importante ressaltar que o comportamento em resposta a uma mensagem caracteriza-se como não determinístico, em decorrência de diversos fatores:

- Os atores podem modificar seu comportamento interno ao processar uma mensagem
- A sequência de recebimento das mensagens não possui garantia de ordenação
- A transmissão das mensagens ocorre de forma assíncrona, sem garantias quanto ao tempo de processamento

### Endereços

Endereços são utilizados como identificadores únicos dos atores, constituindo o meio pelo qual um ator estabelece comunicação com outros. Um ator pode ter acesso ao endereço de outros atores através dos seguintes meios:

- Durante sua inicialização (recebendo através de parâmetros em seu construtor)
- Como parte do conteúdo de uma mensagem recebida
- Mediante a criação de um novo ator

Geralmente, o endereço de um ator é fornecido pelo mecanismo utilizado para sua instanciação.

## Abordagem Elixir

> TODO: Expandir esse texto

A implementação mais proeminente do Modelo de Atores é representada pela BEAM (Bogdan's Erlang Abstract Machine), a máquina virtual da linguagem Erlang, também utilizada pela linguagem Elixir. Em Elixir, o Modelo de Atores constitui a abordagem padrão para o desenvolvimento de sistemas, onde a BEAM atua como facilitadora, possibilitando a criação de atores em processos Erlang distintos (viabilizando sistemas concorrentes) que podem estar distribuídos em máquinas diferentes (permitindo a implementação de sistemas distribuídos).

## Abordagem Rust

Este trabalho objetiva a implementação do Modelo de Atores para o desenvolvimento de uma aplicação em Rust. Propõe-se, portanto, uma abordagem adequada às características da linguagem Rust, visando explorar plenamente os recursos de desempenho e segurança proporcionados por esta linguagem.

### Ambiente de Execução

Primeiramente, Rust caracteriza-se como uma linguagem compilada que incorpora gerenciamento automático de memória sem coletor de lixo e, consequentemente, não dispõe de _runtime_ intrínseca, diferenciando-se de linguagens como Golang. O modelo de atores fundamenta-se significativamente na programação assíncrona, demandando suporte adequado em Rust.

A biblioteca Tokio, amplamente reconhecida, proporciona uma _runtime_ com suporte à programação assíncrona e _green threads_ (denominadas _tasks_). O suporte a _green threads_ reveste-se de importância sob a perspectiva de desempenho, pois viabiliza a execução concorrente de código sem o custo adicional associado à utilização de _threads_ gerenciadas pelo sistema operacional.

### Atores

Os atores serão implementados como _tasks_ Tokio que permanecerão em estado de espera pela recepção de mensagens em um ciclo de execução. Cada ator disporá de autonomia para determinar a estratégia de processamento da fila de mensagens recebidas, podendo optar por: processar a mensagem subsequente apenas após a conclusão do processamento da mensagem atual, ou iniciar o processamento de uma mensagem imediatamente, delegando sua execução a uma _task_ específica.

### Mensagens

Para maximizar a utilização do sistema de tipos robusto da linguagem Rust, as mensagens serão fortemente tipadas mediante o emprego de tipos algébricos. Cada ator definirá uma enumeração em Rust para as mensagens que está apto a processar, sendo que cada variante dessa enumeração constituirá um tipo (potencialmente composto) contendo os dados necessários para o processamento da respectiva mensagem.

A implementação de mensagens tipadas permitirá a verificação em tempo de compilação da correção das mensagens, prevenindo:

- Transmissão de mensagens inválidas
- Comportamentos indefinidos durante o processamento de mensagens recebidas

### Endereços

A comunicação interatores será implementada mediante o uso de canais. Durante sua criação, um ator estabelecerá um canal MPSC (_multiple producer, single consumer_), mantendo consigo o terminal receptor de mensagens (`Receiver`). O terminal emissor (`Sender`) será disponibilizado à entidade que solicitou a criação do ator, possibilitando seu compartilhamento com outros atores.

Para mensagens que demandem resposta, será necessário estabelecer um canal secundário em direção oposta. Este canal deverá ser do tipo _oneshot_, especializado em comunicações onde uma única mensagem é transmitida. O terminal emissor desse canal será enviado juntamente com a mensagem e utilizado pelo ator para retornar a resposta através de uma _future_.

Adicionalmente, será implementada uma abstração sobre o `Sender` MPSC para criar uma API de alto nível, eliminando a necessidade de invocadores utilizarem métodos genéricos como `send` e gerenciarem manualmente a criação de mensagens.

### Compartilhamento de Dados

O foco deste trabalho concentra-se no desenvolvimento de aplicações centralizadas utilizando uma arquitetura baseada no modelo de atores. Esta abordagem proporciona certas flexibilidades, incluindo a possibilidade de compartilhamento de espaços de memória visando a otimização do desempenho do software.

Embora o isolamento de memória constitua uma característica positiva na abordagem Elixir do Modelo de Atores, Rust oferece mecanismos que permitem o compartilhamento de memória sem incorrer em problemas clássicos de sistemas concorrentes, como condições de corrida.

Para este propósito, pode-se utilizar o tipo `Arc` (_atomic reference counter_) para viabilizar o compartilhamento de dados em modo somente leitura em programas concorrentes. Será priorizada a minimização do uso de memória compartilhada para operações de escrita; contudo, quando imprescindível, recorrer-se-á ao tipo `Mutex`, que permite o controle rigoroso de acesso a uma região de memória de forma mutuamente exclusiva.

Mediante esta estratégia, uma aplicação desenvolvida com a arquitetura proposta apresentará vantagens de desempenho em comparação a sistemas distribuídos. Considere-se o cenário em que um ator transmite uma mensagem contendo uma _string_ para outro ator: em sistemas distribuídos, a única alternativa viável consiste na cópia da _string_ caractere por caractere (implicando em complexidade temporal linear em relação ao tamanho da _string_). Na arquitetura proposta, é possível compartilhar a área de memória da _string_ (resultando em complexidade temporal constante).