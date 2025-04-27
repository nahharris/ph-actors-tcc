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

Mensagens consistem em estruturas de dados transmitidas entre atores, sendo que cada ator possui capacidade para processar apenas determinados tipos de mensagens. É importante ressaltar que o comportamento em resposta a uma mensagem caracteriza-se como não determinístico por diversos fatores:

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

A implementação mais famosa do Modelo de Atores é a que se baseia na BEAM (Bogdan's Erlang Abstract Machine), a máquina virtual da linguagem Erlang, também utilizada pela linguagem Elixir. Os principais fatores para sua popularidade estão na simplicidade de se implementar sistemas concorrentes e distribuídos através de mecanismos como _green-threads_ e comunicação entre processos em máquinas remotas. Por causa disso, o Modelo de Atores é a abordagem padrão para a criação de sistemas.

Os atores são representados por processos que são criados através da função `spawn`. Essa função recebe como parâmetros: um módulo, o nome de uma função desse módulo e a lista de argumentos a serem passados para a função. A função `spawn` vai invocar a função indicada em um novo processo e devolver o PID desse processo. Para permitir a comunicação entre processos, existem os métodos `send` e `receive` que permitem a troca de mensagens.

### Exemplo

```elixir
defmodule Echo do
    def init do
        # Aguarda uma mensagem
        receive do
            # Obtém o enviador e o conteúdo da mensagem
            {sender, msg} -> 
                send(sender, {:ok, "Echoing: #{msg}"})
                init()
        end
    end
end

defmodule Example do
    def main do
        # Inicia o ator
        addr = spawn(Echo, :init, []) 
        # Envia uma mensagem
        send(addr, {self(), "Hello world"}) 
        
        # Aguarda pela resposta
        receive do 
            {:ok, response} -> 
                # Exibe a resposta
                IO.puts(response) 
        end
    end
end
```

> `:ok` e `:init` são átomos. Um tipo especial de literal Elixir. Um átomo só é igual a si mesmo. São mais rápidos de serem comparados do que strings.

No exemplo acima, definimos um simples ator de eco. Ele recebe uma mensagem que deve conter o remetente e um conteúdo. Em seguida envia de volta ao remetente uma mensagem com conteúdo `:ok` e a mensagem recebida prefixada de `Echoing: `. Por fim, fazemos uma chamada recursiva para que o ator possa lidar com mais mensagens (Elixir não possui loops).

No módulo exemplo nós usamos a função `spawn` para iniciar o ator através da função `init` do módulo `Echo` sem nenhum parâmetro. Essa função vai devolver o PID do processo criado e esse é o endereço que será usado para comunicar com o ator. A função `send` envia para o ator criado uma tupla onde o primeiro elemento é o PID do processo atual e o segundo é a string `"Hello world"`.

Nesse momento, a chamada a `receive` na função `init` entra em ação. Ela vai receber a mensagem e devolver ao remetente a uma nova tupla onde o primeiro elemento é `:ok` e o segundo a string `"Echoing: Hello world"`. Por causa da chamada recursiva esse ator está pronto para receber mais mensagens.

A chamada a `receive` na função `main` agora vai lidar com a resposta do ator. Vai verificar que a resposta de fato contém um `:ok` e exibir a string `Echoing: Hello world` no terminal.

## Abordagem Rust

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

### Compartilhamento de Dados

O foco deste trabalho concentra-se no desenvolvimento de aplicações centralizadas utilizando uma arquitetura baseada no modelo de atores. Esta abordagem proporciona certas flexibilidades, incluindo a possibilidade de compartilhamento de espaços de memória visando a otimização do desempenho do software.

Embora o isolamento de memória constitua uma característica positiva na abordagem Elixir do Modelo de Atores, Rust oferece mecanismos que permitem o compartilhamento de memória sem incorrer em problemas clássicos de sistemas concorrentes, como condições de corrida.

Para este propósito, pode-se utilizar o tipo `Arc` (_atomic reference counter_) para viabilizar o compartilhamento de dados em modo somente leitura em programas concorrentes. Será priorizada a minimização do uso de memória compartilhada para operações de escrita; contudo, quando imprescindível, recorrer-se-á ao tipo `Mutex`, que permite o controle rigoroso de acesso a uma região de memória de forma mutuamente exclusiva.

Mediante esta estratégia, uma aplicação desenvolvida com a arquitetura proposta apresentará vantagens de desempenho em comparação a sistemas distribuídos. Considere-se o cenário em que um ator transmite uma mensagem contendo uma _string_ para outro ator: em sistemas distribuídos, a única alternativa viável consiste na cópia da _string_ caractere por caractere (implicando em complexidade temporal linear em relação ao tamanho da _string_). Na arquitetura proposta, é possível compartilhar a área de memória da _string_ (resultando em complexidade temporal constante).

### Exemplo

```rust
mod echo {
    use tokio::sync::{mpsc, oneshot};

    pub fn init() -> mpsc::Sender<(oneshot::Sender<Result<String, ()>>, String)> {
        let (tx, mut rx) = mpsc::channel(1);

        tokio::spawn(async move {
            while let Some((sender, msg)) = rx.recv().await {
                sender.send(Ok(format!("Echoing: {}", msg))).await;
            }
        });

        return tx;
    }
}

use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    let addr = echo::init();

    let (tx, mut rx) = oneshot::channel();
    addr.send((tx, String::from("Hello world"))).await;

    let response = rx.recv().await.expect("The channel should be opened");
    println!("{}", response.unwrap());
}
```

O programa acima funciona de forma identica ao exemplo Elixir. Mas algumas coisas foram feitas de forma diferente para se ajustarem melhor à filosofia do Rust.

Primeiramente, os canais de comunicação precisam de ser criados explicitamente, enquanto em Elixir eles são implicitos. Embora isso adicione complexidade ao código, a abordagem Rust permite maior controle. Criamos um canal `mpsc` para permitir multiplas mensagens de chegarem ao ator e configuramos o tamanho máximo da fila de mensagens recebidas. Além disso, criamos canais `oneshot` sob demanda para devolver as respostas.

Note também que a função `init` é quem cria a _task_ e o canal de comunicação. Devido à natureza da função `tokio::spawn`, é mais fácil que a função que vá ser colocada na _task_ seja a responsável por criá-la e retorne um `Sender` ou `JoinHandle` para permitir a interação com a _task_.

Diferentemente de Elixir, uma linguagem de tipagem dinâmica, Rust é de tipagem estática forte. O tipo `mpsc::Sender<(oneshot::Sender<Result<String, ()>>, String)>` é um pouco complicado mas é simples de ser entendido. `mpsc::Sender` indica que a função devolve a ponta de envio de mensagens. `(oneshot::Sender<Result<String, ()>>, String)` é o tipo das mensagens que o ator vai lidar: uma tupla onde o primeiro elemento é o endereço do remetente e o segundo elemento é o texto da mensagem em si. 

O endereço do remetente aqui é a ponta de envio do tipo `oneshot` que é usado para devolver respostas do tipo `Result<String, ()>`. O tipo `Result` aqui é usado apenas para fazer o paralelo com a tupla `{:ok, "Resposta"}` usada em Elixir. No nosso exemplo, o `Result` pode ser um `Ok("Resposta")` ou um `Err(())` (não nos importamos com os erros aqui).

Por fim, analisando a `main`, vemos que ela necessita criar o canal para receber as suas respostas e que necessitamos de usar explicitamente `.await` em chamadas de métodos assincronos (envio e recebimento de mensagens).
