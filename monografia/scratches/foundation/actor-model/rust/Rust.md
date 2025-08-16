There are important considerations while planning regarding Rust philosophy:
- "Do not pay for what you don't use" means that all the costy operations are done explicitly;
- Rust has minimal runtime, so no native support to green-threads, async, etc;
- Memory safety is the core concern of the language, so a lot of abstractions are needed to use memory safely and efficiently;

*...continue...*

We'll be mapping the theory into Rust structures as:

- **Actor**: a _Tokio_ task
- **Address**: a _Tokio_ _mspc_ channel
- **Message**: an `enum` specific to each actor

To spawn a new actor, we'll first need some data structure to manage the state of the actor. This structure will be initialized with either a `new` or `build` static method (`build` for faillible initialization). Then a `spawn` instance method will consume the data structure and move it into a brand new task. This task is the actor, and it will be listening for messages. The `spawn` method must return the address of the actor.

The address will be a bit more complex than the PID that is used in the Elixir approach. It will wrap a `tokio::mspc::Sender` and provide a high-level interface to send messages to the actor.

Messages will be strongly typed, and specific for each actor. They will be enums where each variant is a message that carry all the data the actor need in-order to perform a given action. Also, messages that need a response will carry a `tokio::oneshoot::Sender` so the actor can send a response back.

With this protocol, the usage of actors across Rust code will be similar to regular async code.

### Copy, Clone, References and Mutability 

Here are some policies about how we'll deal will data passing across actors:

- `Copy`: if data is `Copy` it will be passed by copy
- `Clone`: for clone data, we'll rather store it using shared references and pass it by reference

In regular Rust programs, read references (`&`) and mutable references (`&mut`) are used. But regular references cannot be shared across tasks. This way, `Arc` references will be used.

Also we'll avoid mutability at all costs. It means that we'll replace some types:

- `String` -> `Arc<str>`
- `PathBuf` -> `Arc<Path>`

For those types that must be mutable, _Tokio_ mutexes will be needed

- `std::fs::File` -> `Arc<Mutex<tokio::fs::File>>`

These policies are attempts to keep the program safe and performatic using the strenghts of Rust type system.
Este trabalho objetiva a implementação do Modelo de Atores para o desenvolvimento de uma aplicação em Rust. Propõe-se, portanto, uma abordagem adequada às características da linguagem Rust, visando explorar plenamente os recursos de desempenho e segurança proporcionados por esta linguagem.

### Ambiente de Execução

Primeiramente, Rust caracteriza-se como uma linguagem compilada que incorpora gerenciamento automático de memória sem coletor de lixo e, consequentemente, não dispõe de _runtime_ intrínseca, diferenciando-se de linguagens como Golang e Elixir. O modelo de atores fundamenta-se significativamente na programação assíncrona, demandando suporte adequado em Rust.

A biblioteca Tokio, amplamente reconhecida, proporciona uma _runtime_ com suporte à programação assíncrona e _green threads_ (denominadas _tasks_). O suporte a _green threads_ é muito importante do ponto de vista de desempenho, pois viabiliza a execução concorrente de código sem o custo adicional associado à utilização de _threads_ gerenciadas pelo sistema operacional.

### Atores

Os atores serão implementados como _tasks_ Tokio criadas pela função `tokio::spawn` que permanecerão em estado de espera pela recepção de mensagens em um ciclo de execução. Cada ator disporá de autonomia para determinar a estratégia de processamento da fila de mensagens recebidas, podendo optar por: processar a mensagem seguinte apenas após a conclusão do processamento da mensagem atual, ou iniciar o processamento de uma mensagem imediatamente, delegando sua execução a uma _task_ específica. 

### Mensagens

Para maximizar a utilização do sistema de tipos robusto da linguagem Rust, as mensagens serão fortemente tipadas usando tipos algébricos. Cada ator definirá uma enumeração em Rust para as mensagens que está apto a processar, sendo que cada variante dessa enumeração constituirá um tipo (potencialmente composto) contendo os dados necessários para o processamento da respectiva mensagem.

A implementação de mensagens tipadas permitirá a verificação em tempo de compilação da correção das mensagens, prevenindo:

- Transmissão de mensagens inválidas
- Comportamentos indefinidos durante o processamento de mensagens recebidas

### Endereços

A comunicação entre atores será implementada com o uso de canais. Durante sua criação, um ator estabelecerá um canal MPSC (_multiple producer, single consumer_), mantendo consigo o terminal receptor de mensagens (`Receiver`). O terminal emissor (`Sender`) será disponibilizado à entidade que solicitou a criação do ator, possibilitando seu compartilhamento com outros atores.

Para mensagens que demandem resposta, será necessário estabelecer um canal secundário em direção oposta. Este canal deverá ser do tipo _oneshot_ (especializado em comunicações onde uma única mensagem é transmitida). O terminal emissor desse canal será enviado juntamente com a mensagem e utilizado pelo ator para retornar a resposta através de uma _future_.

Adicionalmente, será implementada uma abstração sobre o `Sender` MPSC para criar uma interface de comunicação de alto nível, eliminando a necessidade de invocadores utilizarem métodos genéricos como `Sender::send` e gerenciarem manualmente a criação de mensagens.
