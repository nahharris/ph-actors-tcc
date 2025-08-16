The focus of this work concentrates on developing centralized applications using an architecture based on the actor model. This approach provides certain flexibilities, including the possibility of sharing memory spaces aimed at optimizing software performance.

Although memory isolation constitutes a positive characteristic in Elixir's approach to the Actor Model, Rust offers mechanisms that allow memory sharing without incurring classic concurrent system problems, such as race conditions.

For this purpose, one can use the `Arc` type (_atomic reference counter_) to enable data sharing in read-only mode in concurrent programs. Priority will be given to minimizing the use of shared memory for write operations; however, when essential, the `Mutex` type will be used, which allows rigorous access control to a memory region in a mutually exclusive manner.

```rust
// Create a vector protected by a Mutex
let data = Mutex::new(vec![1, 2, 3]);

// Locks the Mutex, that returns a reference to the data it holds
let mut lock = data.lock().unwrap();
// We can now modify the data
lock.push(4);

// No one else can get access to the data while the lock holds
println!("{:?}", data); // Mutex { data: <locked>, poisoned: false, .. }

// Releases the lock
drop(lock);

// Now you can use data again if needed
println!("{:?}", data); // Mutex { data: [1, 2, 3, 4], poisoned: false, .. }
```

Through this strategy, an application developed with the proposed architecture will present performance advantages compared to distributed systems. Consider the scenario where an actor transmits a message containing a _string_ to another actor: in distributed systems (separate machines), the only viable alternative consists of copying the _string_ character by character (implying linear time complexity relative to the _string_ size). In the proposed architecture, it is possible to share the _string's_ memory area (resulting in constant time complexity) while being completely safe against race conditions.