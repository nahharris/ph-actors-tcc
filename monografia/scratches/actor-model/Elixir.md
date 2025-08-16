The most famous implementation of the Actor Model is the one based on BEAM (Bogdan's Erlang Abstract Machine), the virtual machine of the Erlang language, also used by the Elixir language. The main factors for its popularity lie in the simplicity of implementing concurrent and distributed systems through mechanisms such as _green-threads_ and inter-process communication on remote machines. Because of this, the Actor Model is the standard approach for creating systems in the language.

Actors are represented by Erlang processes (hereafter referred to only as processes) that are created through the `spawn` function. This function receives as parameters: a module, the name of a function from that module, and the list of arguments to be passed to the function. The `spawn` function will invoke the indicated function in a new process and return the PID of that process. To enable communication between processes, there are the `send` and `receive` methods that allow message exchange.

### Example

```elixir
defmodule Echo do
    def init do
        # Waits for a message
        receive do
            # Gets the sender and message content
            {sender, msg} -> 
                # Echoes the response with success (`:ok`)
                send(sender, {:ok, "Echoing: #{msg}"})
                # Loop
                init()
        end
    end
end

defmodule Example do
    def main do
        # Starts the actor
        addr = spawn(Echo, :init, []) 
        # Sends a message
        send(addr, {self(), "Hello world"}) 
        
        # Waits for the response
        receive do 
            {:ok, response} -> 
                # Displays the response
                IO.puts(response) 
        end
    end
end
```

> `:ok` and `:init` are atoms. A special type of Elixir literal. An atom is only equal to itself. They are faster to compare than strings.

> In Elixir there are no loops, so recursion with tail call optimization is used

> `self()` returns the PID of the current process

In the example above, we define a simple echo actor. It receives a message that must contain the sender and content. It then sends back to the sender a message with content `:ok` and the received message prefixed with `Echoing: `. Finally, we make a recursive call so the actor can handle more messages (Elixir has no loops).

In the example module we use the `spawn` function to start the actor through the `init` function of the `Echo` module without any parameters. This function will return the PID of the created process and this is the address that will be used to communicate with the actor. The `send` function sends to the created actor a tuple where the first element is the PID of the current process and the second is the string `"Hello world"`.

At this moment, the call to `receive` in the `init` function comes into action. It will receive the message and return to the sender a new tuple where the first element is `:ok` and the second is the string `"Echoing: Hello world"`. Because of the recursive call this actor is ready to receive more messages.

The call to `receive` in the `main` function will now handle the actor's response. It will verify that the response indeed contains an `:ok` and display the string `Echoing: Hello world` in the terminal.
## References

- Programming Elixir
