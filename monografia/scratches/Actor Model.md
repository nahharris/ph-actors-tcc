# Actor Model

The actor model is a model for concurrent computation where a program is though
in terms of **actors** that work independently of each other and communicate via
messages.

There are 3 main parts of a system that uses the Actor Model:

- **Actor**: a computation unit
- **Message**: data send to an actor
- **Address**: identifies an actor, used to identify the receiver of a message

When an actor receives a message, it can do a lot of things, like:

- Define the behaviour of next messages
- Create new actors
- Send messages to other actors

It's also important to put emphasys in the fact that the message ordering is not guaranteed

The actor model can be think of as distributed OOP. Where the actors act like objects but they may execute code concurrently or in parallel (in different threads or even in different machines).

## Elixir Approach

Some concepts from Elixir will be used to design the approach that will be used for the actor model in Rust

In general you will have at least 2 elements to implement an actor in elixir. An (_Erlang_) process and a high level interface to interact with it.

The process works like a loop, receiving messages with the `receive` method and then uses pattern matching to handle the message.

The interface will abstract the communication details since to send a message to a process in elixir you need to know it's pid and use `send`. The idea is to create methods that will hide this low level details from anyone who needs to use the actor. In Elixir this is particularly useful because you won't need to write the messages manually (elixir is dynamically typed, so more error prone).

So the Elixir approach maps to the Actor theory as: **Actor**s are _Erlang_ processes, the **Address** is the PID and the **Message** is the data sent with `send` and received with `receive`.

## Rust Approach

We'll be mapping the theory into Rust structures as:

- **Actor**: a _Tokio_ task
- **Address**: a _Tokio_ _mspc_ sender
- **Message**: an `enum` specific to each actor

To spawn a new actor, we'll first need some data structure to manage the state of the actor. This structure will be initialized with either a `new` or `build` static method (`build` for faillible initialization). Then a `spawn` instance method will consume the data structure and move it into a brand new task that will be running in the background. This task is the actor, and it will be listening for messages. The `spawn` method must return the address of the actor.

The address will be a bit more complex than the PID that is used in the Elixir approach. It will wrap a `tokio::mspc::Sender` and provide a high-level interface to send messages to the actor.

Messages will bne strongly typed, and specific for each actor. They will be enums where each variant is a message that carry all the data the actor need in-order to perform a given action. Also, messages that need a response will carry a `tokio::oneshoot::Sender` so the actor can send a response back.

With this protocol, the usage of actors across Rust code will be similar to regular async code.

### Copy, Clone, References and Mutability 

Here are some policies to how we'll deal will data passing across actors:

- `Copy`: is data is `Copy` it will be passed by copy
- `Clone`: for clone data, we'll rather store it using shared references and pass it by reference

In regular Rust programs, regular `&` and `&mut` are used. But regular references cannot be shared across tasks. This way, `Arc` references will be used.

Also we'll avoid mutability at all costs. It means that we'll replace some types:

- `String` -> `Arc<str>`
- `PathBuf` -> `Arc<Path>`

For those types that must be mutable, _Tokio_ mutexes will be needed

- `std::fs::File` -> `Arc<Mutex<tokio::fs::File>>`

These policies are attempts to keep the program safe and performatic using the strongs of Rust type system.

## References

- Programming Elixir
- Actor Model of Computation: Scalable Robust Information Systems


## The Actor Model of Computation: Core Concepts

> Made by Claude 3.7 Sonnet by summarizing Hewitt Paper

This paper by Carl Hewitt presents the Actor Model as a mathematical theory that treats "Actors" as universal primitives of digital computation. The central hypothesis is that all physically possible computation can be directly implemented using Actors, positioning it as more fundamental than previous computational models.

### Fundamental Principles

The Actor Model defines computation as distributed entities (Actors) that communicate through asynchronous message passing. When an Actor receives a message, it can concurrently:

1. Send messages to addresses of other Actors that it knows
2. Create new Actors
3. Designate how to handle the next message it receives

Unlike previous computational models that relied on shared memory or synchronous communication, the Actor Model embraces true concurrency. The sender of a message is not intrinsic to the semantics of communication, allowing for decoupling of components.

### Key Characteristics

- **Locality and Security**: An Actor can only communicate with Actors whose addresses it possesses, and can only obtain addresses through creation, message reception, or by creating new Actors
- **Asynchronous Communication**: Messages are sent without waiting for a response, and there's no guarantee of delivery timing
- **Indeterminacy**: The reception order of messages is not predetermined, allowing for modeling real-world concurrency
- **Unbounded Nondeterminism**: Unlike Turing machines, Actor systems can exhibit unbounded nondeterminism, which better represents real distributed systems

### Information System Principles

The Actor Model supports robust information integration through:

- **Persistence**: Information is collected and indexed
- **Concurrency**: Work proceeds interactively and in parallel
- **Quasi-commutativity**: Operations can be performed in different orders when order doesn't matter
- **Pluralism**: Systems can handle heterogeneous, overlapping, and inconsistent information
- **Provenance**: The origin of information is tracked and recorded

### Implementation Techniques

The paper describes several implementation approaches:

- **Swiss Cheese Pattern**: For controlling access to shared resources by defining "holes" where concurrent execution is allowed
- **Futures**: For parallel execution and handling of return values
- **Organizational Programming**: Using hierarchical organizational structures for scalability

### Relationship to Other Models

The Actor Model differs from previous models in significant ways:

- Unlike the lambda calculus, it supports true concurrency and unbounded nondeterminism
- Unlike process calculi (e.g., π-calculus), it's based on physics rather than algebra
- Unlike classical object models, it focuses on message passing rather than method invocation
- Unlike CSP and other models, it doesn't require synchronous communication

### Practical Applications

The Actor Model has been implemented in languages and systems like:
- Erlang (for telecom systems)
- Orleans (for cloud computing)
- Various specialized Actor languages (Act1, Act2, Ani, Cantor)
- Modern features in JavaScript, C#, and Java

The model provides a foundation for building robust, scalable systems in distributed environments, cloud computing, many-core architectures, and systems requiring fault tolerance.

This computational paradigm represents a fundamental shift from sequential, centralized processing to distributed, message-based computation that better reflects the physics of real-world systems.
