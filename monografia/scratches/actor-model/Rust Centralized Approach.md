One of the core results of this work is a framework for centralized applications written in Rust using the Actor Model. Once the theory about the Actor Model and the known implementations have been discussed, it's time to adapt its abstractions to the specific needs and traits of centralized Rust applications.

## Concurrency and Parallelism

This proposal englobbes the use of green-threads which have no native support in Rust. This is because Rust, unlike Elixir, Golang, or Java, has a very minimal runtime, similar o C's runtime. The support for green-threads is provisioned by the Tokio library and they are called _tasks_.

Tasks are spawned and executed with isolation either concurently or in parallel depending on the amount operating system threads that are being used by Tokio.

## Memory Sharing

Memory isolation is desirable and a native trait of the Actor Model since actors are designed to run in isolated environments (for instance different physical machines). This increases security but degrades perfomance since data between actors is shared by copy, not by reference.

But that's not the case for that proposal since the application isn't distributed. Moreover, the issues with memory sharing with concurrent systems are related with race conditions and undesired mutability. Rust provide native efficient mechanisms to share memory spaces safely.

The abstraction that will be used here is the `Arc<T>` type, which permits a memory space to be shared in read-only mode with atomic reference counting (once no one is referencing it, it's freed).

## Message Passing and Addresses

Both Rust and Tokio have a native message passing abstraction called `channel`. Unlike Elixir's `send` method, channels are strongly typed (which permit static checking of messages) and with defined lifecycles. The 2 principal channels types for the scope of this work are:

### `mpsc`

A multiple producer, single consumer channel is the abstraction for an actor address. It has 2 ends: `Sender` and `Receiver`. The `Sender` can be shared between all the actors that want to communicate with a particular actor. The `Receiver` end will be held by the actor instance so it can listen to messages until the channel is closed (no more `Sender`s exist).

It´s important to notice that Rust ensures, at compile time, that the receiver of a channel is unique. So it's guaranteed that the messages are arriving to the right destination. In addition, the channel has a queue with a configurable size, and it's guaranteed that the message will arrive if the queue is not full.

### `oneshot`

A channel that is single use and destroyed after a message is sent. Since channels are unidirectional, if a sent message needs a response other than the information that it arrived a oneshot channel will be created on demand for the sending of the response.

Is also worth to point out that "use after free" of an oneshot channel is checked at compile time.