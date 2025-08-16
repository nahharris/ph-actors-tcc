```rust
mod echo {
    use tokio::sync::{mpsc, oneshot};

    pub fn init() -> mpsc::Sender<(oneshot::Sender<Result<String, ()>>, String)> {
        let (tx, mut rx) = mpsc::channel(8); // Creates a channel

		// Spawns the actor in a Tokio task
        tokio::spawn(async move {
	        // Wait for messages
            while let Some((sender, msg)) = rx.recv().await {
	            let response = format!("Echoing: {}", msg);
	            // Echoes a response back
                sender.send(Ok(response)).await;
            }
        });

        return tx;
    }
}

use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
	// Initializes the actor
    let addr = echo::init();

	// Create a channel to receive the response
    let (tx, mut rx) = oneshot::channel();
    addr.send((tx, String::from("Hello world"))).await;

	// Wait for the response
    let response: Result<String, ()> = rx.recv().await.expect("The channel should be opened");
    println!("{}", response.unwrap());
}
```
> `"Hello World"` is not `String` in Rust, it's actually `&'static str` (a sequence of characters that is known at compile time). This is a design choise made by Rust for sake of optimization (the core concern of the language). That's why we need to use `String::from("Hello World")`

The program above works identically to the Elixir example. But some things were done differently to better fit Rust's philosophy.

First, Rust doesn't have PIDs like Elixir, instead we use channels to allow communication. Although this adds complexity to the code, the Rust approach allows greater control. We create an `mpsc` (Multiple Producer, Single Consumer) channel to allow multiple messages to arrive at the actor and configure the maximum size of the received message queue. Additionally, we create `oneshot` (single-use) channels on demand to return responses.

Note also that the `init` function is the one that creates the _task_ and the communication channel. Due to the nature of the `tokio::spawn` function, it's easier for the function that will be placed in the _task_ to be responsible for creating it and return a `Sender` to allow interaction with the _task_.

Unlike Elixir, a dynamically typed language, Rust is strongly statically typed. The type `mpsc::Sender<(oneshot::Sender<Result<String, ()>>, String)>` is somewhat complex but simple to understand. `mpsc::Sender` indicates that the function returns the message sending end of the channel. `(oneshot::Sender<Result<String, ()>>, String)` is the type of messages the actor will handle: a tuple where the first element is the sender's address and the second element is the message text itself.

The sender's address here is the sending end of the `oneshot` type that is used to return responses of type `Result<String, ()>`. The `Result` type here is used only to parallel the tuple `{:ok, "Response"}` used in Elixir. In our example, the `Result` can be an `Ok(String::from("Response"))` or an `Err(())` (we don't care about errors here).

Finally, analyzing `main`, we see that it needs to create the channel to receive its responses and that we need to explicitly use `.await` in asynchronous method calls (sending and receiving messages).