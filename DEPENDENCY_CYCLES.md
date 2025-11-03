# Late Dependency Injection for Actor Model

This document outlines the plan for implementing late dependency injection in the actor model architecture, focusing on breaking the `Log` ↔ `Fs` cycle while maintaining strict adherence to the actor pattern.

## Problem Statement

Currently, we have a dependency cycle:
- **Log** depends on **Fs** (needs to create/write log files)
- **Fs** should be able to log errors (when file operations fail)
- **Terminal** actor controls terminal output (can't use `eprintln!` or similar)
- We must maintain the actor model structure throughout

This creates a circular dependency that prevents `Fs` from logging.

## Solution Overview: Late Dependency Injection Pattern

We will implement a **late dependency injection pattern** that allows actors to:
1. Start in a minimal state with optional dependencies
2. Buffer operations that require missing dependencies
3. Accept dependency injection messages after initialization
4. Transition to full operational mode once dependencies are available

This pattern combines:
- **Two-Phase Initialization**: Actors initialize in bootstrap mode, then transition to operational mode
- **Optional Dependencies**: Dependencies are `Option<T>` and can be injected later
- **Buffering**: Operations requiring dependencies are buffered until dependencies are available

## Architecture Design

### 1. Log Actor: Two-Phase Initialization

The `Log` actor will operate in two modes:

#### Phase 1: Buffered Mode (No Fs dependency)
- Accepts log messages and buffers them in memory
- Does NOT require `Fs` at creation time
- Only stores messages, does not write to files
- Can still provide the logging interface to other actors

#### Phase 2: File Mode (Fs injected)
- Receives `Fs` dependency via message injection
- Creates log files using `Fs`
- Flushes buffered messages to files
- Transitions to normal file logging mode

### 2. Fs Actor: Optional Logging

The `Fs` actor will have optional logging:

#### Bootstrap Phase
- Created without `Log` dependency
- Buffers error messages when logging is requested
- Errors are stored in memory until logger is available

#### Operational Phase
- Receives `Log` dependency via message injection
- Flushes buffered error messages
- Logs errors normally going forward

## Implementation Plan

### Step 1: Modify Log Actor Core

#### Message Types

```rust
// src/log/message.rs
pub enum Message {
    Log(LogMessage),
    Flush { tx: oneshot::Sender<anyhow::Result<()>> },
    CollectGarbage,
    // NEW: Inject Fs dependency
    InjectFs { fs: Fs, tx: oneshot::Sender<anyhow::Result<()>> },
}
```

#### Core State

```rust
// src/log/core.rs
pub enum LogMode {
    /// Buffered mode: no Fs, messages stored in memory
    Buffered {
        buffer: Vec<LogMessage>,
        config: Config,
        print_level: LogLevel,
    },
    /// File mode: Fs available, normal operation
    FileMode {
        fs: Fs,
        config: Config,
        log_dir: ArcPath,
        log_path: ArcPath,
        log_file: tokio::fs::File,
        latest_log_file: tokio::fs::File,
        logs_to_print: Vec<LogMessage>,
        print_level: LogLevel,
        max_age: usize,
    },
}

pub struct Core {
    mode: LogMode,
}
```

#### Core Implementation

```rust
impl Core {
    /// Create Log in buffered mode (no Fs dependency)
    pub fn new_buffered(config: Config) -> anyhow::Result<Self> {
        let log_level = config.log_level().await;
        Ok(Self {
            mode: LogMode::Buffered {
                buffer: Vec::new(),
                config,
                print_level: log_level,
            },
        })
    }

    /// Create Log in file mode (with Fs dependency)
    /// This is used when Fs is already available or for testing
    pub async fn new_with_fs(fs: Fs, config: Config) -> anyhow::Result<Self> {
        // ... existing initialization code ...
    }

    pub async fn init(mut self, mut rx: Receiver<Message>) {
        while let Some(msg) = rx.recv().await {
            use Message::*;
            match msg {
                Log(msg) => {
                    self.handle_log(msg).await;
                }
                Flush { tx } => {
                    self.handle_flush(tx).await;
                    rx.close();
                    break;
                }
                CollectGarbage => {
                    self.handle_collect_garbage().await;
                }
                InjectFs { fs, tx } => {
                    let result = self.handle_inject_fs(fs).await;
                    let _ = tx.send(result);
                }
            }
        }
    }

    async fn handle_log(&mut self, message: LogMessage) {
        match &mut self.mode {
            LogMode::Buffered { buffer, print_level, .. } => {
                // Store message in buffer
                buffer.push(message.clone());
                
                // Note: In buffered mode, we can't print to stderr
                // because Terminal actor controls terminal output
                // Messages will be flushed once Fs is injected
            }
            LogMode::FileMode { log_file, latest_log_file, logs_to_print, print_level, .. } => {
                // Normal file logging (existing implementation)
                use tokio::io::AsyncWriteExt;
                let formatted = format!("{}\n", &message);
                
                log_file.write_all(formatted.as_bytes()).await
                    .expect("Failed to write to log file");
                log_file.flush().await
                    .expect("Failed to flush log file");
                
                latest_log_file.write_all(formatted.as_bytes()).await
                    .expect("Failed to write to latest log file");
                latest_log_file.flush().await
                    .expect("Failed to flush latest log file");
                
                if message.level >= *print_level {
                    logs_to_print.push(message);
                }
            }
        }
    }

    async fn handle_inject_fs(&mut self, fs: Fs) -> anyhow::Result<()> {
        // Extract current buffered state
        // Note: In actual implementation, we might need to restructure to avoid
        // the need for a placeholder in mem::replace. Options include:
        // 1. Store config separately outside LogMode
        // 2. Use Option<LogMode> to allow None temporarily
        // 3. Extract config before the replace using a match on reference
        let temp_config = match &self.mode {
            LogMode::Buffered { config, .. } => config.clone(),
            LogMode::FileMode { config, .. } => {
                // Already in file mode, no injection needed
                return Ok(());
            }
        };
        
        let (buffer, config, print_level) = match std::mem::replace(&mut self.mode, LogMode::Buffered {
            // Temporary placeholder - will be replaced immediately below
            buffer: vec![],
            config: temp_config.clone(),
            print_level: LogLevel::Info,
        }) {
            LogMode::FileMode { .. } => {
                // This case was already handled above, but keep for safety
                unreachable!("FileMode should have been handled above")
            }
            LogMode::Buffered { buffer, config, print_level } => {
                (buffer, config, print_level)
            }
        };
        
        // Get configuration values from config actor
        let max_age = config.usize(crate::app::config::USizeOpt::MaxAge).await;
        let log_dir = config.path(crate::app::config::PathOpt::LogDir).await;
        
        let log_path = ArcPath::from(&log_dir.join(format!(
            "patch-hub_{}.log",
            chrono::Utc::now().format("%Y-%m-%d-%H-%M-%S")
        )));
        let latest_log_path = ArcPath::from(&log_dir.join("latest.log"));
        
        // Create log directory and files
        fs.mkdir(log_dir.clone()).await
            .context("Failed to create log directory")?;
        
        let log_file = fs.write_file(log_path.clone()).await
            .context("Failed to create log file")?;
        
        let latest_log_file = fs.write_file(latest_log_path).await
            .context("Failed to create latest log file")?;
        
        // Flush buffered messages to files
        use tokio::io::AsyncWriteExt;
        for message in &buffer {
            let formatted = format!("{}\n", message);
            log_file.write_all(formatted.as_bytes()).await?;
            latest_log_file.write_all(formatted.as_bytes()).await?;
        }
        log_file.flush().await?;
        latest_log_file.flush().await?;
        
        // Extract messages that should be printed to stderr
        let logs_to_print: Vec<_> = buffer.iter()
            .filter(|m| m.level >= print_level)
            .cloned()
            .collect();
        
        // Transition to file mode with actual values
        self.mode = LogMode::FileMode {
            fs,
            config,
            log_dir,
            log_path,
            log_file,
            latest_log_file,
            logs_to_print,
            print_level,
            max_age,
        };
        
        Ok(())
    }

    async fn handle_flush(&mut self, tx: oneshot::Sender<anyhow::Result<()>>) {
        match &self.mode {
            LogMode::Buffered { buffer, .. } => {
                // In buffered mode, we can't print to stderr
                // Messages will be lost unless Fs is injected before flush
                // This is acceptable for early shutdown scenarios
                eprintln!("Warning: Flushing Log in buffered mode - {} messages may be lost", buffer.len());
            }
            LogMode::FileMode { logs_to_print, log_path, .. } => {
                // Existing flush implementation
                for message in logs_to_print {
                    eprintln!("{message}");
                }
                if !logs_to_print.is_empty() {
                    eprintln!("Check the full log file: {}", log_path.display());
                }
            }
        }
        let _ = tx.send(Ok(()));
    }

    async fn handle_collect_garbage(&mut self) {
        match &mut self.mode {
            LogMode::Buffered { .. } => {
                // Can't collect garbage without Fs
                return;
            }
            LogMode::FileMode { fs, log_dir, max_age, .. } => {
                // Existing garbage collection implementation
                // ...
            }
        }
    }
}
```

#### Public Interface

```rust
// src/log.rs
impl Log {
    /// Create Log in buffered mode (no Fs dependency)
    pub fn spawn_buffered(config: Config) -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = Core::new_buffered(config)?;
        let _ = tokio::spawn(async move {
            core.init(rx).await;
        });
        Ok(Self { tx })
    }

    /// Inject Fs dependency to enable file logging
    pub async fn inject_fs(&self, fs: Fs) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Message::InjectFs { fs, tx })
            .await
            .context("Sending InjectFs message to Log actor")
            .expect("Log actor died");
        rx.await
            .context("Awaiting response for InjectFs from Log actor")
            .expect("Log actor died")
    }

    // Keep existing methods (info, warn, error, flush, collect_garbage)
    // They work the same way, routing to handle_log
}
```

### Step 2: Modify Fs Actor Core

#### Message Types

```rust
// src/fs/message.rs
pub enum Message {
    // ... existing messages ...
    // NEW: Inject Log dependency
    InjectLogger { log: Option<Log>, tx: oneshot::Sender<()> },
}
```

#### Core State

```rust
// src/fs/core.rs
pub struct Core {
    log: Option<Log>,
    error_buffer: Vec<LogMessage>,  // Buffer errors until logger is ready
}

impl Core {
    pub fn new() -> Self {
        Self {
            log: None,
            error_buffer: Vec::new(),
        }
    }

    pub async fn init(mut self, mut rx: Receiver<Message>) {
        while let Some(msg) = rx.recv().await {
            use Message::*;
            match msg {
                ReadFile { tx, path } => self.handle_read_file(tx, path).await,
                WriteFile { tx, path } => self.handle_write_file(tx, path).await,
                AppendFile { tx, path } => self.handle_append_file(tx, path).await,
                RemoveFile { tx, path } => self.handle_remove_file(tx, path).await,
                ReadDir { tx, path } => self.handle_read_dir(tx, path).await,
                MkDir { tx, path } => self.handle_mkdir(tx, path).await,
                RmDir { tx, path } => self.handle_rmdir(tx, path).await,
                InjectLogger { log, tx } => {
                    self.handle_inject_logger(log).await;
                    let _ = tx.send(());
                }
            }
        }
    }

    fn log_error(&mut self, scope: &'static str, message: String) {
        let log_message = LogMessage {
            level: LogLevel::Error,
            scope,
            message,
        };
        
        if let Some(ref log) = self.log {
            // Logger available, log immediately
            log.error(scope, log_message.message.clone());
        } else {
            // Logger not available, buffer error
            self.error_buffer.push(log_message);
        }
    }

    async fn handle_inject_logger(&mut self, log: Option<Log>) {
        self.log = log;
        
        // Flush buffered errors
        if let Some(ref log) = self.log {
            for error in self.error_buffer.drain(..) {
                log.error(error.scope, error.message);
            }
        }
    }

    async fn handle_read_file(
        &mut self,
        tx: oneshot::Sender<Result<tokio::fs::File, io::Error>>,
        path: ArcPath,
    ) {
        match OpenOptions::new().read(true).open(&path).await {
            Ok(file) => {
                let _ = tx.send(Ok(file));
            }
            Err(e) => {
                self.log_error("fs", format!("Failed to read file: {} - {}", path.display(), e));
                let _ = tx.send(Err(e));
            }
        }
    }

    // Similar error logging for other operations...
}
```

#### Public Interface

```rust
// src/fs.rs
impl Fs {
    pub fn spawn() -> Self {
        // Existing implementation
    }

    /// Inject Log dependency to enable error logging
    pub async fn inject_logger(&self, log: Option<Log>) {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Message::InjectLogger { log, tx })
            .await
            .context("Sending InjectLogger message to Fs actor")
            .expect("Fs actor died");
        rx.await
            .context("Awaiting response for InjectLogger from Fs actor")
            .expect("Fs actor died")
    }
}
```

### Step 3: Update Initialization Sequence

```rust
// src/main.rs
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    install_panic_hook()?;

    // Phase 1: Initialize core actors without circular dependencies
    let env = Env::spawn();
    let fs = Fs::spawn();  // Fs without Log
    
    // Get config
    let home = std::env::var("HOME")
        .ok()
        .or_else(|| std::env::var("USERPROFILE").ok())
        .unwrap_or_else(|| ".".to_string());
    let config_dir = std::path::Path::new(&home)
        .join(".config")
        .join("patch-hub");
    let config_path = ArcPath::from(&config_dir.join("config.toml"));
    let config = Config::spawn(env.clone(), fs.clone(), config_path);
    config.load().await.ok();

    // Phase 2: Initialize Log in buffered mode (no Fs dependency yet)
    let log = Log::spawn_buffered(config.clone())?;
    
    // Phase 3: Inject Fs into Log to enable file logging
    log.inject_fs(fs.clone()).await?;
    
    // Phase 4: Inject Log into Fs to enable error logging
    fs.inject_logger(Some(log.clone())).await;
    
    // Phase 5: Continue with normal initialization
    let net = Net::spawn(config.clone(), log.clone());
    let shell = Shell::spawn(log.clone()).await?;
    // ... rest of initialization ...
    
    Ok(())
}
```

## Benefits

1. **No Circular Dependencies**: `Log` and `Fs` can be created independently
2. **No Message Loss**: Buffering ensures all messages are eventually logged
3. **Actor Model Compliance**: All communication via messages, no breaking the pattern
4. **Terminal Safety**: No direct stderr output, respects Terminal actor control
5. **Flexible Initialization**: Can inject dependencies in any order
6. **Backward Compatible**: Can still create actors with dependencies if needed (for testing)

## Trade-offs

1. **Memory Usage**: Buffering uses memory (typically small, but could grow)
2. **Complexity**: Slightly more complex state management
3. **Initialization Order**: Requires careful sequencing in main
4. **Early Shutdown**: If application exits before injection, buffered messages may be lost

## Testing Considerations

1. **Mock Support**: Both actors should work with `Option<Log>` and `Option<Fs>`
2. **Buffering Tests**: Verify messages are buffered and flushed correctly
3. **Injection Tests**: Test injection at various stages
4. **Integration Tests**: Test full initialization sequence

## Future Extensibility

This pattern can be extended to other actors that might have circular dependencies:
- `Config` might need logging for configuration errors
- `Net` might need logging for network errors
- Any actor that needs logging but is needed by Log

The same injection pattern can be applied.

## Implementation Checklist

- [ ] Modify `Log` actor core to support buffered mode
- [ ] Add `InjectFs` message to `Log`
- [ ] Update `Log::spawn()` to support buffered initialization
- [ ] Modify `Fs` actor core to support optional logging
- [ ] Add `InjectLogger` message to `Fs`
- [ ] Add error buffering in `Fs`
- [ ] Update initialization sequence in `main.rs`
- [ ] Update tests for both actors
- [ ] Update documentation
- [ ] Verify no message loss in integration tests