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


## References
- Actor Model of Computation: Scalable Robust Information Systems