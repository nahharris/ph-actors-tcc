The Actor Model of computation was stablished by Carl Hewitt in 1973 grounded on previous work done by Peter Bishop and Richard Steiger. His objetives were to address the new challenges imposed by the massive concurrent and parallel systems that emerged with cloud computing and many-core architectures.

His objectives were achieved creating a computational model that is inherently concurrent. A big shift from previous models is that it's based on physical laws, instead of pure algebra. The most important hypotesis defended by Hewitt's work is:

> All physically possible computation can be directly implemented with Actors

The Actor Model represents a complete rethinking of concurrent computation, moving from global state machines to distributed, asynchronous message-passing systems that better reflect the physical reality of modern computing systems.
## What is an Actor?

Hewitt proposes this primitive called Actor that is identified by an address. Actors are entities that will communicate with each other through messages. Once a message is received, an Actor can:

- Send messages to Actors they know the address
- Spawn new Actors
- Modify it's own behaviour for handling future messages received

Messages are the unit of communication and passed asynchronously between actors. According to the principle of security and locality, when processing a received message, an Actor can only send messages to Actors whose addresses were:

- Known beforehand
- Part of the message received
- Created during the current processing

## Indeterminacy

Here comes one the main principles in the Actor Model and what sets it apart of other patterns. The arrival order of messages is not guaranteed and the Actor Model won't try to address this, but rather deal with it as a natural property of the domain. The Actor Model is designed for distributed systems, which means that two actors can be communication over long physical distances. 

This means that messages sent can that unbounded time to arrive, and even if they arrive at the exact same time, the hardware arbiters that will order them won't guarantee any sort of ordering. Considering the physical factors is what differentiates the Actor Model from purely algebraic concurrent system patterns.

The indeterminated message ordering, summed with the fact that messages might change the behaviour of an Actor leads to the fact that the same initial conditions might result in different results. Moreover, the system itself is said to have no global defined state. Since the communication is asynchronous, at any moment is possible to have messages traveling or Actors in transient state.

This is what ensures fairness
## References
- Actor Model of Computation: Scalable Robust Information Systems