One of the core results of this work is proposal of a framework for centralized applications written in Rust using the Actor Model. Once the theory about the Actor Model and the known implementations have been discussed, it's time to adapt its abstractions to the specific needs and traits of centralized Rust applications.
## Why Rust?

The Actor Model is widely known for its dynamicity and flexibility, with most of its famous implementations being in flexible runtimes like BEAM or JVM. Rust is quite the opposite of that, a modern language with a strong type system and a compiler powerful enough to validate almost everything at compile time. Moreover, the same surveys that put Elixir as the second most loved programming language chose Rust as the most loved programming language since 2016 (it's worth mentioning that Rust 1.0 was released in May, 2015, and the surveys happen in January).

Might sound contradictory to use a rigid language with a flexible pattern, but it's exactly the purpose of this work: experiment with the Actor Model outside of its comfort zone. This will enable the experimentation of the limits of both the Actor Model and the Rust programming language.
## Quick Introduction To Rust

Rust is known because of its security mechanisms that forces compile time checking of many things and making invalid state not possible to be even written in certain cases.

The most important concepts here are:
### Memory Safety

A memory space has one, and only one, owner (a variable), this owner determines the lifecycle of the memory space. Once the owner is destroyed, the memory space is freed. The owner also determines if the region is mutable or not.

```rust
let a = 10;
let mut b = 20;
a += 1; // forbidden: a is not mutable
b += 1; // alright
// since the code ends here, a and b are freed from memory
```

A memory space can be borrowed with the usage of references, but the compiler will make sure that references to a value are not being used after the owner is destroyed. There might be infinite references (read-only `&`) to a value or one single mutable reference (writable `&mut`), but one can't have both.

```rust
let numbers = vec![1, 2, 3, 4];
let nref = &numbers;
nref[1]; // -> 2
drop(numbers); // this will destroy numbers

// from this point and beyond, the compiler 
// will forbid the use of both `numbers` and `nref`
```

```rust
let mut numbers = vec![1, 2, 3];
let nref = &mut numbers;
nref.push(4); // A mutable operation can be done using a &mut
let nref2 = &mut numbers; // forbidden: there's already a &mut to numbers
```
### Type System

The strong and powerful type system is a key characteristic of Rust. It does not have inheritance, but it compensates that with tuples, structs, enums and traits:

#### `tuple`

A tuple is simply a product type, it has a fixed size and each field (named after their index) might have a different type. It's not a collection!
```rust
let value: (i32, f32) = (20, 3.14);
value.0; // -> 20
```

#### `struct`
A struct is similar to a tuple, but it has named fields and might have methods with the use of an `impl` block:

```rust
struct Point {
	timestamp: i32,
	value: f32
}

impl Point {
	// This is a method that will be able to mutate the value of self
	fn add(&mut self, value: f32) {
		self.value += value;
	}
}

let mut p = Point { timestamp: 20, value: 2.14 };
p.add(1.0);
```

There is also a construct called tuple-struct which is a struct but the fields are named after their indexes, like a tuple. It also supports methods.
```rust
struct Coordinate(i32, i32);

let o = Coordinate(0, 0);
```

Notice that there is no "constructor" in a struct, in the sense that Rust doesn't have constructors similar to what Java, Python or C++ provide. But often, developers provide static methods named `new` that serve as constructors, but `new` itself has no special meaning.

```rust
impl Point {
	fn new(timestamp: i32, value: f32) -> Self {
		Self { timestamp, value } // Short-hand syntax 
	}
}

let p = Point::new(10, 5.5);
```

Notice that a method is static if it take no `self`, `&self`, or `&mut self` as the first argument and the operator `::` is used instead of `.` when accessing static attributes. Also there's the `Self` type, that is an alias for the type being implemented.
#### `enum`

Unlike most languages, enums in Rust might have payloads since each variant can be treated as a struct

```rust
enum Coordinate {
	Polar { 
		theta: f32, 
		radius: f32
	},
	Cartesian(f32, f32),
	Origin,
}

let o: Coordinate = Coordinate::Origin;
let p1: Coordinate = Coordinate::Cartesian(1, 1);
let p2: Coordinate = Coordinate::Polar { theta: 45.0, radius: 2 };
```

#### `trait`

Traits is the Rust equivalent of a Java interface, it defines methods that must be implemented by a type so it has a given trait. They are useful when bounding generic types or to overload operators

```rust
// Display is a trait that is used for string formatting with `{}`
impl Display for Coordinate {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
		match self {
			Coordinate::Origin => write!(f, "Origin"),
			Coordinate::Cartesian(x, y) => write!(f, "({}, {})", x, y),
			Coordinate::Polar { theta, radius } => 
				write!(f, "({} ∠ {}°)", radius, theta)
		}
	}
}

let c = Coordinate::Polar { radius: 10.0, theta: 75.0 };
println!("The coordinate is {}", c);
```
## Concurrency and Parallelism

This proposal englobbes the use of green-threads which have no native support in Rust. This is because Rust, unlike Elixir, Golang, or Java, has a very minimal runtime, similar o C's runtime. The support for green-threads is provisioned by the Tokio library and they are called _tasks_.

Tasks are spawned and executed with isolation either concurently or in parallel depending on the amount operating system threads that are being used by Tokio.

## Memory Sharing

Memory isolation is desirable and a native trait of the Actor Model since actors are designed to run in isolated environments (for instance different physical machines). This increases security but degrades perfomance since data between actors is shared by copy, not by reference.

But that's not the case for that proposal, since the application isn't distributed. Moreover, the issues with memory sharing with concurrent systems are related to race conditions and undesired mutability. Rust provide efficient native mechanisms to share memory spaces safely.

The `Arc<T>` type will be used, it permits a immutable memory space to be shared with atomic reference counting (once no one is referencing it, it's freed).

## Message Passing and Addresses

Both Rust and Tokio have a native message passing abstraction called `channel`. Unlike Elixir's `send` method, channels are strongly typed (which permit static checking of messages) and with defined lifecycles. The 2 principal channels types for the scope of this work are:

### `mpsc`

A multiple producer, single consumer channel is the abstraction for an actor address. It has 2 ends: `Sender` and `Receiver`. The `Sender` can be shared between all the actors that want to communicate with a particular actor. The `Receiver` end will be held by the actor instance so it can listen to messages until the channel is closed (no more `Sender`s exist).

It's important to notice that Rust ensures, at compile time, that the receiver of a channel is unique. So it's guaranteed that the messages are arriving to the right destination. In addition, the channel has a queue with a configurable size, and it's guaranteed that the message will arrive if the queue is not full.

### `oneshot`

A channel that is destroyed after a message is sent. Since channels are unidirectional, if a sent message needs a response other than the information that it arrived an oneshot channel will be created on demand for the response.

Is also worth to point out that "use after free" of an oneshot channel is checked at compile time.

## Typed Messages

Just like everything else in Rust, the messages sent through channels must be typed, in particular since an actor might accept different kinds of messages, an `enum` will be used. 

```rust
enum Message {
	SetThing { value: i32 }, 
	GetThing { tx: Sender<i32> }
}
```

The biggest advantage of typing messages, is that sending and handling is verified at compile time. This creates a source of truth for the kind of messages that can be involved in a given communication and the compiler will enforce that:

- The receiver will provide explicit handling for all possible messages
- The sender will never send an invalid message

The above gurantees do not hold for Elixir implementation of the Actor Model.

## The Actor Representation

Now it's time to define what an actor will look like in terms of Rust code. First of all, there will be a module for each actor, and if an actor is subordinated to the existance of another actor by the bussiness logic, it can be created as a submodule. 

In this work, the standard for modules will be having a file and a folder with the same name (except by the `.rs` extension). For instance, for an actor called App, there must be a `app.rs` and a `app/` in the same directory.

So far, 3 main parts of an actor were presented:

- The address that enable the communication between actors
- The messages that will be sent to the actor
- The handler that deals with the messages received

Those 3 parts will be translated to separate parts of Rust code as:

- A public struct to wrap the `mspc` channel that serves as the address with sending methods
- A private enum to define the types of the messages
- A private struct to manage the actor state and define how to handle each message

For the example App actor there's gonna be:

- A struct`App` at `app.rs`to serve as the public interface for dependents of this actor
- An enum `Message` at `app/message.rs` defining the type for the messages
- A struct `Core` at `app/core.rs` that holds the actor logic

Notice that the convention is the struct that serves as the public interface for the actor will be named after the actor for improved readability. This so called public interface must provide methods that abstract away the message building and the channel utilization logic.

### Public Interface

The public interface for an actor is an abstraction of the address, and it's the object that will be shared between dependants of that actor. The main purpose of it is to abstract away all the comunication details (building the message, preparing oneshot response channels, sending and waiting for response, etc) by providing methods (ideally, one per message kind).

Suppose that the App actor has a message called GetTime that returns the total execution time of the application. This means that:

- There must be a `GetTime` variant for the `Message` enum
- The payload for that message must contain a oneshot channel for the response
- The `Core` must provide means to handle that message
- The `App` must have a `get_time` method that deals with the communication

A simplified example of what this method could look like:

```rust
impl App {
	pub async fn get_time(&self) -> Duration {
		// Creates the response channel
		let (mut rx, tx) = oneshot::channel(); 
		
		// Builds the message
		let msg = Message::GetTime { tx };
		
		// Send the message using the async `send` method
		self.addr.send(msg).await.expect("The App actor should not be dead");
		
		// Receives the response using `recv` and returns it
		rx.recv().await.expect("The App actor should not be dead")
	}
}
```

For a dependent of the App actor, the communication would be just:

```rust
async fn do_task(&self, app: App) { // Inject the dependency
	...
	let time = app.get_time().await; // Just use it
	...
}
```

#### Mocks

Mocking is a common technique for software testing. It consists of creating fake implementations of parts of the system that are simple and controlable enough so they cannot be a source of bugs, at the same time, other parts of the system never know if they are interacting with the actual implementation or with a mock. This way, testing an isolated specific portion of the code becomes a simple task.

In a actor modeled system, isolation is native since actors, by design, touch very specific domains. The dependencies on other actors are injected on specific points by using public interfaces. So simply by providing mock implementations of actors' public interfaces, testing isolation is easy. 

Often, programming languages create mocks by inheritance. Since Rust doesn't support that, the public interface will be an enum with 2 variants: `Actual` and `Mock`. The former will wrap the address to an instance of an actor and actually process the message. The latter will wrap a struct that will contain mocked logic that will be used when unit testing a dependant actor.

### Message

Since the public interface is there to abstract away the communication details, the message enum is something internal to the module. It's actually supposed to be really simple and straight forward. Below are some coventions that will be adopted:
1. They will all be called just `Message` since they are private to the module
2. The variants should be named like verb + noun phrase, unless either of them is clear by the context
3. In a getter-setter pair of messages, the getter might ommit the `Get` verb (general Rust convention)
4. A variant should contain a `tx` field if the message needs a response
### Core
The core is the abstraction over the behaviour of the actor. It's responsible for dependency injection, dealing with low-level details of Tokio for spawning new tasks, instantiating the public interface, and having the methods to handle each message that arrives in a loop. 

#### `Core::new` or `Core::build`
In some scenarios, creating the actor core is as simple as receiving a couple parameters and putting them into a struct. On the other hand, some cases might require more operations that are fallible. For the former, the constructor will be called `new` and return `Self`. For the latter, it will be called `build` and return a `Result` where the `Ok` variant holds `Self`.

#### `Core::spawn`
Once the core is built, the next step is to spawn the actor. The `spawn` method will consume the actor struct, prepare the `mspc` channel for it, and spawn a Tokio task that will contain the logic to route the messages that arrive in the channel until it's closed. After that, the method will return the public interface (by wrapping the channel sender) and a join handle. The join handle is an object that permits to await until the task it's associated to ends.

```rust
 pub fn spawn(mut self) -> (PublicInterface, tokio::task::JoinHandle<()>) {
        let (tx, mut rx) = mpsc::channel(BUFFER_SIZE);

		// Spawns a new separate task to handle the logic
        let handle = tokio::spawn(async move {
	        // This task will process all messages that arrive until the channel is closed
            while let Some(msg) = rx.recv().await {
                match msg {
	                // Describes the behaviour for each kind of message
                    Message::DoAction { tx, arg } => {
	                    // Invokes the corresponding method to handle the request
	                    let response = self.do_action(arg).await;
	                    // Send the response back
	                    let _ = tx.send(response);
	                }
	                // ... more branches
                }
            }
        });

		// Return a tuple with the public interface instance and the join handle
        (PublicInterface::Actual(tx), handle)
    }
```
