Functional Programming (FP) is a paradigm that uses functions as its main abstraction and is derived from principles of *lambda calculus*. Some of the core features of functional programming languages (FPL) are:
- **Functions as values**: functions can be stored in variables as regular values
- **Higher order functions**: functions that take other functions as arguments or return other functions as its result
- **Pure functions**: functions that do not have side-effects, in simple terms, they don't modify or depends on mutable values out of its scope, also don't do any IO operations. It means that for the same input they'll always produce the same output
- **Immutable Data**: data the is not suposed to mutabe is great for concurrent programs since race conditions cannot happen
And those 3 features are enough to provide some key aspects of what this work is looking for:
- **Decoupling**: high-order functions are great for this, since they decouple a part of the function execution logic to be defined by the caller of the function
- **Testability**: since a pure function has no side-effects, testing it is trivial since they are completely decoupled from the state of the program, and just depends on its input values.

Those are definitely some characteristics from FPL that will be used in this work.
# Pure Functions
One of the most important concepts of FP. Pure functions are a special kind of function whose output depends only on the value of its input and do not produce any side-effects.
## Why care about the function purity?
They are excellent for a lot of applications:
- **Easy parallelism**: since they won't write state, nor depend on reading mutable state, they can be safely run in parallel
- **Testability**: also testing pure functions is trivial since a test just need to define the input arguments and check if the output matches the expectation
- **Predictable behaviour**: pure functions won't affect any other aspect of the program execution
> State a theoreme about the composition of pure functions being also a pure function
## Impure Functions
In simple terms an impure function is a function that do at least one of the following:
- Do IO operations
- Mutate values out of the function scope
- Depends of mutable values out of the function scope
- Calling it with the same arguments won't always produce the same result
It means that a function that does any amount of pure operations but do a single impure operation is classified as an impure function.
Those functions are not evil in any sense and one probably need those in order to create a useful program. But reducing impurity to just a few spots might be good. This way the amount of pure functions is maxed which is a good thing.
### Extracting purity
From the above, we know that an impure function needs to do at least of one those said impure operations. But if it does a non negligible amount of pure operations, those could be extracted into a new pure function. This way we can increase the overall testability of the function.

```rust
fn greet() {
	let mut buf = String::new();
	std::io::stdin().read_line(&mut buf).expect("Failed to read a line");
	
	println!("Hello, {}", buf.to_uppercase());
}
```

The above function does 2 impure operations by reading content from `stdin` and calling `println`. But the transformation on the `name` string and the formatting are pure operations. So those could be extracted into a separate pure function.

```rust
fn make_greeting(name &str) -> String {
	let name = name.to_uppercase();
	return format!("Hello, {}", name);
}

fn greet() {
	let mut buf = String::new();
	std::io::stdin().read_line(&mut buf).expect("Failed to read a line");
	
	let greeting = make_greeting(&buf);
	
	println!("{}", greeting);
}
```

The behavior is still the same, but now we extracted the pure part of the previous implementation to a separated pure function that is easy to test. 

Also let's check the readability of both implementations of `greet`:
1. Create a string
2. Read content from `stdin` and store it into the string
4. Print "Hello, " and then the uppercased version of the input

1. Create a string
2. Read content from `stdin` and store it into the string
3. Make a greeting out of the received input
4. Print the greeting

In the first case the word "greet" is not even used when talking about the steps of the `greet` function. In the second case, we actually read **what** the function is doing: print a greeting.
### Purifying functions
Some impure functions might be purified if it's not impure at its core. One way to achieve this is to delegate the execution of the impure code to another function by treating actions as values.
This action will be stored and late executed by some where in the code, this way the impure execution happens in a single spot in your program. This concept is known as Monad.