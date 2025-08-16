Actors are autonomous computational units that manage their own internal state, operating without shared memory space. Consequently, interactions between actors are made exclusively through messages. Upon receiving a message, an actor can:

- Instantiate new actors
- Transmit messages to other known actors
- Define the behavior to be adopted in processing the next received message

Due to their autonomous nature, actors need not be bound to the same _thread_ or the same machine. Therefore, sufficiently robust abstractions are necessary to enable interactions between actors.