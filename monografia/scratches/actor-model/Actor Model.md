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

---

The Actor Model of computation was stablished by Carl Hewitt in 1973 grounded on previous work done by Peter Bishop and Richard Steiger. His objetives were to address the new challenges imposed by the massive concurrent and parallel systems that emerged with cloud computing and many-core architectures, but founded on physical laws, unlike most other previous work based purely on algebra. The most important hypotesis defended by Hewitt's work is:

> All physically possible computation can be directly implemented with Actors

## What is an Actor?

Hewitt proposes this primitive called Actor that is identified by an address. Actors are entities that will communicate with each other through messages. Once a message is received, an Actor can:

- Send messages to Actors they know the address
- Spawn new Actors
- Modify it's own behaviour for handling future messages received

Messages are the unit of communication and passed asynchronously between actors. According to the principle of security and locality, when processing a received message, an Actor can only send messages to Actors whose addresses were:

- Known beforehand
- Part of the message received
- Created during the current processing

The arrival order of messages is not guaranteed, which leads to the principle of indeterminacy. Since messages might change the actor behaviour, same initial conditions might end in different results since the message arrival order might differ across different executions of the same program.
## References
- Actor Model of Computation: Scalable Robust Information Systems