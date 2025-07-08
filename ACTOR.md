# Actor Pattern Implementation

This document describes the actor pattern implementation used throughout the codebase for providing thread-safe interfaces to various system operations.

## Overview

The actor pattern is implemented using Rust's async/await with Tokio channels to provide thread-safe access to shared resources. Each actor consists of three main components:

1. **Core** - The internal implementation that handles the actual work
2. **Message** - The message types that can be sent to the actor
3. **Public Interface** - The enum that provides a unified interface with both real and mock implementations

## Architecture

### Core Component (`core.rs`)

The core component contains the actual implementation logic and state:

```rust
pub struct Core {
    // Internal state and dependencies
}

impl Core {
    /// Creates a new core instance
    pub fn new() -> Self { /* ... */ }
    
    /// Spawns the actor and returns the interface and task handle
    pub fn spawn(self) -> (PublicInterface, JoinHandle<()>) {
        let (tx, mut rx) = tokio::sync::mpsc::channel(BUFFER_SIZE);
        let handle = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                match message {
                    // Handle different message types
                }
            }
        });
        (PublicInterface::Actual(tx), handle)
    }
}
```

### Message Types (`message.rs`)

Messages define the operations that can be performed on the actor:

```rust
pub enum Message {
    /// Operation that returns a value
    GetValue {
        tx: oneshot::Sender<Result<Value, Error>>,
        // ... other parameters
    },
    /// Operation that doesn't return a value
    SetValue {
        // ... parameters
    },
}
```

### Public Interface

The public interface provides a unified API with both real and mock implementations:

```rust
#[derive(Debug, Clone)]
pub enum PublicInterface {
    /// Real actor implementation
    Actual(Sender<Message>),
    /// Mock implementation for testing
    Mock(Arc<Mutex<MockData>>),
}

impl PublicInterface {
    /// Creates a new real actor instance
    pub fn spawn() -> Self {
        let (interface, _) = Core::new().spawn();
        interface
    }
    
    /// Creates a new mock instance for testing
    pub fn mock(data: MockData) -> Self {
        Self::Mock(Arc::new(Mutex::new(data)))
    }
    
    /// Public method that sends messages to the actor
    pub async fn operation(&self, params: Params) -> Result<Value, Error> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender.send(Message::Operation { tx, params }).await?;
                rx.await?
            }
            Self::Mock(data) => {
                // Mock implementation
            }
        }
    }
}
```

## Pattern Characteristics

### Thread Safety
- All actors use message passing through Tokio channels
- Internal state is never shared directly between threads
- Operations are processed sequentially within each actor

### Async/Await Support
- All public methods are async
- Internal message handling is async
- Uses Tokio's runtime for concurrency

### Mock Support
- Each actor provides a mock implementation for testing
- Mock implementations store state in memory using `Arc<Mutex<T>>`
- Real and mock implementations share the same public interface

### Error Handling
- Uses `anyhow::Result` for error propagation
- Channel communication errors are handled gracefully
- Actor death is detected through channel send failures

## Implementation Examples

### Network Actor (`net`)
- **Purpose**: HTTP requests
- **State**: HTTP client, configuration, logging
- **Messages**: GET requests
- **Mock**: Not implemented (only real actor)

### Logging Actor (`log`)
- **Purpose**: File and stderr logging
- **State**: Log files, message buffer, log level
- **Messages**: Log, Flush, CollectGarbage
- **Mock**: No-op implementation

### Filesystem Actor (`fs`)
- **Purpose**: File operations
- **State**: File handle cache
- **Messages**: OpenFile, CloseFile, RemoveFile, ReadDir, MkDir, RmDir
- **Mock**: In-memory file storage

### Environment Actor (`env`)
- **Purpose**: Environment variable operations
- **State**: None (uses system environment)
- **Messages**: SetEnv, UnsetEnv, GetEnv
- **Mock**: In-memory environment storage

### Configuration Actor (`config`)
- **Purpose**: Configuration file management
- **State**: Configuration data, file path
- **Messages**: Load, Save, GetPath, SetPath, GetLogLevel, SetLogLevel, GetUSize, SetUSize
- **Mock**: In-memory configuration storage

## Usage Patterns

### Creating Actors
```rust
// Real actor
let fs = Fs::spawn();
let env = Env::spawn();
let config = Config::spawn(env, fs, config_path);

// Mock actor for testing
let fs = Fs::mock(HashMap::new());
let env = Env::mock();
let config = Config::mock(Data::default());
```

### Using Actors
```rust
// Async operations
let file = fs.open_file(path).await?;
let value = env.env(key).await?;
config.set_log_level(LogLevel::Info).await;
```

### Error Handling
```rust
// Actor death detection
match fs.open_file(path).await {
    Ok(file) => { /* use file */ }
    Err(e) => {
        // Actor may have died, handle gracefully
    }
}
```

## Best Practices

1. **Always use async methods** - All actor operations are async
2. **Handle actor death** - Check for channel send failures
3. **Use mocks for testing** - Avoid real system operations in tests
4. **Clone interfaces** - Actor interfaces are cheap to clone (only copy channel sender)
5. **Document messages** - Each message variant should have clear documentation
6. **Use oneshot channels** - For operations that return values
7. **Use mpsc channels** - For the main message channel

## Dependencies

The actor pattern relies on these Tokio components:
- `tokio::sync::mpsc` - Multi-producer, single-consumer channels for message passing
- `tokio::sync::oneshot` - Single-use channels for response handling
- `tokio::task` - Task spawning and management
- `tokio::sync::Mutex` - For mock implementations that need shared state