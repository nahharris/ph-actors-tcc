# Fundamentação Teórica

## O que é o Modelo de Atores

> De acordo com Hewitt em Actor Model of Computation: Scalable Robust Information Systems

O Modelo de Atores é uma arquitetura pensada para computação concorrente distribuída. Programas são descritos em termos de _atores_ que existem de forma independente e se comunicam trocando mensagens. Existem 3 componentes principais num sistema modelado com atores:

### Atores

Atores são unidades de computação independentes que gerenciam seu próprio estado, ou seja, não compartilham espaço de memória. Portanto, as interações entre os atores são feitas através de mensagens. Ao receber uma mensagem, um ator pode:

- Criar novos atores
- Enviar mensagens para outros atores que ele conheça
- Determinar o comportamento da próxima mensagem recebida

Por serem unidades independentes, os atores não precisam estar na mesma thread ou na mesma máquina. Logo, deve haver abstrações poderosas o suficiente para permitir as interações entre os atores. 

### Mensagens

Mensagens são os dados enviados entre atores e, evidentemente, atores só sabem lidar com alguns tipos de mensagens. Todavia, o comportamento resposta a uma mensagem é não determinístico e isso se dá por uma série de fatores:

- Os atores podem modificar o seu comportamento ao receber uma mensagem
- A ordem de chegada das mensagens não pode ser garantida
- As mensagens são enviadas de forma assíncrona, ou seja, sem garantia de tempo de processamento

### Endereços

Endereços são utilizados para identificar os atores e é através de um endereço que um ator consegue se comunicar com outros. Um ator pode ter acesso ao endereço de outros atores por alguns meios:

- Durante sua própria criação (recebendo através de algum tipo de construtor)
- Como conteúdo de uma mensagem recebida
- Através da criação de um outro ator

Em geral, o endereço de um ator é fornecido pelo método usado para criar ele.

## Abordagem Elixir

> TODO: Expandir esse texto

A aplicação mais famosa do Modelo de Atores é a BEAM (Bogdan's Erlang Abstract Machine) que é a "máquina virtual" da linguagem Erlang que também é usada pela linguagem Elixir. Em Elixir, o Modelo de Atores é a abordagem padrão para se criar sistemas e a BEAM funciona como uma facilitadora permitindo a criação de atores em processos Erlang diferentes (criando sistemas concorrentes) que podem estar em máquinas diferentes (criando sistemas distribuídos).

## Abordagem Rust

Esse trabalho visa a implementação do Modelo de Atores para a criação de uma aplicação em Rust. Desse modo, vamos propor uma abordagem que faça sentido para a linguagem Rust, permitindo explorar ao máximo os recursos de performance e segurança oferecidos pela linguagem.

### Ambiente de Execução

Primeiramente, Rust é uma linguagem compilada que possui gerenciamento automático de memória sem coletor de lixo e, portanto, não possui nenhuma _runtime_, diferentemente de linguagens como Golang. O modelo de atores se apoia fortemente no uso de programação assíncrona e, por isso, precisamos de suporte a isso em Rust.

A famosa biblioteca Tokio fornece uma _runtime_ com suporte a programação assíncrona e _green threads_ (chamadas de _tasks_). O suporte a _green threads_ é importante do ponto de vista performático pois permite a execução concorrente de código sem o custo extra do uso de _threads_ a nível de sistema operacional.

### Atores

Os atores serão traduzidos em _tasks_ Tokio que ficarão aguardando pela chegada de mensagens em um loop. Cada ator terá liberdade para decidir como irá lidar com a fila de mensagens que receber, no sentido que pode decidir: processar a mensagem seguinte apenas quando terminar de processar a mensagem atual ou iniciar o processamento de uma mensagem imediatamente e descarregar ela para uma _task_ própria.

### Mensagens

Para tirar máximo proveito do poderoso sistema de tipos da linguagem Rust, as mensagens são fortemente tipadas usando tipos algébricos. Portanto, cada ator irá definir uma enumeração Rust para as mensagens que pode lidar, onde cada variante dessa enumeração é um tipo (potencialmente composto) com os dados necessários para o processamento dessa mensagem.

O uso de mensagens tipadas vai permitir a checagem em tempo de compilação da corretude das mensagens no sentido que irá impedir:

- Envio de mensagens inválidas
- Comportamento não definido ao receber uma mensagem

### Endereços

A comunicação entre os atores será feita usando canais. Quando um ator for criado, ele irá criar um canal MPSC (_multiple producer, single consumer_) e manterá consigo a ponta de recebimento de mensagem (`Receiver`). Já a ponta de envio (`Sender`) será devolvida a quem solicitou a criação do ator de modo que possa ser compartilhada com outros atores.

Para mensagens que necessitem de resposta, um canal secundário em sentido oposto deve ser criado. Esse canal deve ser do tipo _oneshot_, que é especializado em comunicações onde uma única mensagem é enviada. A ponta de envio desse canal deve ser enviada junto com a mensagem e será usada pelo ator para retornar a resposta através de uma _future_.

Por fim, usaremos uma abstração em cima de um `Sender` MPSC para criar uma API de alto nível que remova a necessidade de invocadores usarem métodos genéricos como `send` e tenham que cuidar da criação manual da mensagem.

### Compartilhamento de Dados

O foco desse trabalho está em criar aplicações centralizadas usando uma arquitetura baseada no modelo de atores. Isso nos dá algumas liberdades como a possibilidade de compartilhar espaços de memória para melhorar a performance do software.

Embora isolamento de memória seja um ponto positivo da abordagem Elixir do Modelo de Atores, Rust nos fornece meios para permitir o compartilhamento de memória sem causar problemas clássicos de sistemas concorrentes como condições de corrida.

Para isso, é possível usar o tipo `Arc` (_atomic reference counter_) para permitir o compartilhamento de dados em modo apenas leitura em programas concorrentes. Tentaremos ao máximo evitar o uso de memória compartilhada para escrita, mas onde não for possível, será usado o tipo `Mutex` que permite o controle rigoroso de acesso a uma região de memória de modo mutualmente excludente.

Com isso, um programa usando a arquitetura proposta teria uma vantagem performática em relação a sistemas distribuídos. Imagine que um ator vai enviar uma mensagem com uma _string_ para outro ator. Em sistemas distribuídos, não há outra opção além de copiar a _string_ caracter por caracter (consumo de tempo linear no tamanho da _string_). Já na arquitetura proposta, podemos compartilhar a área de memória da _string_ (consumo de tempo constante).