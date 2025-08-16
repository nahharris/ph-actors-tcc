Messages consist of data structures transmitted between actors, with each actor having the capacity to process only certain types of messages. It is important to note that the behavior in response to a message is characterized as non-deterministic due to various factors:

- Actors can modify their internal behavior when processing a message
- The sequence of message reception has no ordering guarantee
- Message transmission occurs asynchronously, without guarantees regarding processing time