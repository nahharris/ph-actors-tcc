# Actor Model

The actor model is a model for concurrent computation where a program is though
in terms of **actors** that work independently of each other and communicate via
messages.

When hit with a message an actor can do a lot of things, like:

- Define the behaviour of next messages
- Create new actors
- Send messages to other actors

The actor model can be think of as a concurrent safe OOP.

## Elixir Approach

I use some concepts that i've learn in Elixir to draw my approach for the actor model in Rust

In general you will have at least 2 elements to implement an actor in elixir. A (_Erlang_) process and a high level interface to interact with it.

The process works like a loop, receiving messages with the `receive` method and then uses pattern matching to handle the message.

The interface will abstract the communication details since to send a message to a process in elixir you need to know it's pid and use `send`. The idea is to create methods that will hide this low level details from anyone who needs to use the actor. In Elixir this is particularly useful because you won't need to write the messages manually (elixir is dynamically typed, so more error prone).

## Rust Approach

I tried to design a implementation of the actor based on the elixir approach but adapt to Rust (and Patch-Hub) needs.

First will describe what i want:

1. Actors running in parallel with green threads
2. Use pattern matching to handle messages
3. Have a high level interface to hide communication details
4. Swappable actors

For the first point, an async runtime is the solution. And `tokio` is the obvious choice in Rust. The second one implies in the need to have messages as enumerations for safe pattern matching. For the third and fourth one I think I'll have to use `trait`s to define the actors. Not a single `trait` for all actors, but a `trait` for each kind of actor.

The `actix` library provides support for implementing the actor model in Rust. But it is sort of restrictive and limiting and adds several layers for constructing an actor that are sort of overkill for Patch-Hub.

So my proposal is to implement an actor called `Foo`:

- A data structure called `Foo` responsible for initializing it
- A method `Foo::spawn` which produces both a concrete `FooActor` and a `JoinHandle` and spawn a tokio task to handle messages
- An enum `Commands` that describes the messages that can be sent to the running actor
- A trait `FooActor` the defines the interface that users of this actor will have.
- One or mode concrete `FooTx` that implement `FooActor` and might (or not) communicate with `Foo` via tokio channels

Some more in depth details

### `Foo::spawn`

This method that actually transforms the data structure into an actor by spawning a task.

The task will have an "infinite" loop that receives messages from a tokio `mspc` channel (Multiple Producers Single Consumer) and uses pattern matching to define what to do with each message. This function should return a concrete `FooActor` that will likely wrap a sender to the channel being read by the task. Also is interesting to return the spawned task join handle.

### `Commands`

The messages are defined by a `Commands` enum. Each variant contains the parameters relevent in order to complete the command. And with the commands we also define how the actor will be able to respond to the messages.

Rust channels are unidirectional. So the existing `mspc` channel can only be used to send messages to the actor, but cannot be used to send responses back. The solution is simple: for each command that needs a response, we create a `oneshoot` channel (A channel that can be used to send a single message) and send the sending half with the message payload.

> By a design choice, this enum should be private to the module where it is used. No one outside the scope of this module should bother with the details or how to create a message for the actor, that's why we have a `FooActor`

### The high-level interface

`FooActor` defines with methods we will use to communicate with the actor (at least one method for each variant of `Commands`). `FooTx` is a concrete implementation that needs to bother with the communication details like: building a `Commands` and calling `send`, creating a `oneshoot` channel and await for the actor response, simplifing the flexible interface provided by `FooActor` to the low-level needs of channel messaging.

## References

- Programming Elixir
