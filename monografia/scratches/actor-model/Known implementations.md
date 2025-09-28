Lots of prior art was done regarding implementation of the Actor Model for creation of distributed systems. There are really big libraries and frameworks such as Akka (Java) and Orleans (C#) that bring support to many languages. But the most famous implementation is the Elixir programming language.

In the 80s, the Swedish company Ericsson was experimenting with functional programming languages to develop solutions for telecommunication when they came up with Erlang. The focus of Erlang was concurrency, distribution and fault tolerance by leveraging the Actor Model at language level. BEAM, the abstract machine (by the time the term "virtual machine" was not applied to that domain) that was the runtime for Erlang programs was the biggest acomplishment of the Ericsson. On the otherhand, Erlang's syntax was considered complicated which became a barrier to enable larger adoption of the language.

In the early 2010s, José Valim was working on creating solutions for distributed systems. The exact same problems Ericsson solved 30 years ago. Once he found out about the work done with Erlang and BEAM noticed good solutions, but that wera just too hard to use. The Elixir programming language was then born, combining the power and flexibility of BEAM and the Actor Model with a much more simple and approachable Ruby-like syntax.

Unlike library implementations that bring support to the Actor Model, Elixir has the Actor Model as the default approach for developing software. It does that by providing native support to features such as green-threads, fault tolerance and remote inter-process communication. These were the differentials that made Elixir the main platform for the Actor Model. 

This young dynamic language is gaining more attention year over year, and it is already considered the second most admired and loved programming language in the world for the past 3 years according to Stack Overflow Developer Survey. Because of that, Elixir will be used in this work as the reference implementation and state of art for the Actor Model.

## The Elixir Programming Language

The Elixir programming language depends heavily on Erlang libraries and BEAM. That why those two will be mentioned a lot when elaborating on Elixir features.

The actors are represented by Erlang processes (from now on, referred to only as processes) that are created through the `spawn` function. This function receives as parameters: a module, the name of a function from that module, and the list of arguments to be passed to the function. The `spawn` function will invoke the indicated function in a new process and return the PID of that process that can be used as it's address. The functions `send` and `receive` will permit message exchange between different actors.

### Elixir Basics

Elixir is a dynamically typed programming language, it means that types are checked only at runtime. Newer versions of the language are experimenting with optional type annotations but still early stages.

There are many different native data types in Elixir:

```
i = 123 
f = 3.14 
s = "héllo" # UTF-8 
c = 'hello' # Charlists (list of ints)
b = true
a = :atom
```

The special syntax `:somename` is used to declare atoms, an Erlang concept. They are constants named by themselves and used in many scenarios. They are used to represent status codes, function names, tags or even represent primitive data values:

```
true == :true
false == :false
nil == :nil
```

When it comes to compound data, elixir has 2 main types: tuples and lists. The former are fixed size and quickly indexable. The later are single linked lists with dynamic length. Tuples are heavly used as simple structured data objects. Data as a whole in Elixir is immutable, it means that you cannot change data, but only used it to produce new data.

```
response = {:ok, "Some message"}
status = elem(response, 0) # :ok
response = put_elem(response, 1, "Another message") # {:ok, "Another message"}

data = [1, 2, 3, 4]
data = ["0" | data] # ["0", 1, 2, 3, 4]
data = data ++ [5] # ["0", 1, 2, 3, 4, 5]
```

The `=` is called the bind operator, because it does not simply do attribution, but can be used for pattern matching.

```
x = 1
1 = x # That succeeds
2 = x # That will crash

{status, message} = {:fail, "the system panicked"}

IO.puts "It's a #{status}. Because #{message}"
# It's a fail. Because the system panicked
```

More broadly, pattern matching can be done with the `->` syntax:

```
response = {:ok, "Content"}

case response do
	{:ok, content} -> IO.puts "Success with content: #{content}"
	{:err, cause} -> IO.puts "Ugh, it failed due to: #{cause}"
end
```

In Elixir, there are some different forms of declaring functions, and they can be namespaced with the usage of modules:

```
defmodule Math do
  # Public function
  def add(a, b) do
    a + b
  end

  # Shorthand single-line
  def mul(a, b), do: a * b

  # Private function (only callable inside module)
  defp square(x), do: x * x

  # Multiple function heads with pattern matching
  def abs(n) when is_number(n) and n < 0, do: -n
  def abs(n) when is_number(n), do: n
end

IO.puts(Math.add(2, 3))   # 5
IO.puts(Math.mul(4, 5))   # 20
```

### Example

Now that you are introduced to some elixir syntax

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

> `:ok` and `:init` are atoms. A special type of literal value in Erlang. An atom is only equal to atoms with the same name. They are faster to compare than strings.

> In Elixir there are no loops, so recursion with tail call optimization is used

In the example above, it's defined a simple echo actor. It receives a message that must contain the address of the sender and some message. It then sends back to the sender a message with content `:ok` and the received message prefixed with `Echoing: `. Finally, there's a recursive call to `init` so the actor can handle more messages.

In the `Example` module, the call to `spawn` will start the actor through the `init` function of the `Echo` module without any parameters. This function will return the PID of the created process that serves as the actor's address. The `send` function sends to the newly created actor a tuple where the first element is the PID of the current process and the second is the string `"Hello world"`.

At this moment, the call to `receive` in the `init` function comes into action. It will receive the message and return to the sender a new tuple where the first element is `:ok` and the second is the string `"Echoing: Hello world"`. Because of the recursive call this actor is ready to receive more messages.

The call to `receive` in the `main` function will now handle the actor's response. It patiently waited for a response and can now verify that the response indeed contains an `:ok` and display the string `Echoing: Hello world` in the terminal.
## References

- Programming Elixir


## References
- https://erlang.org/course/history.html
- https://www.erlang.org/faq/academic
- Programming Elixir 1.6
- https://survey.stackoverflow.co/