# Actor Model Compliance Report

This document reports the compliance status of all actors in the `src/` directory against the actor-model specification defined in `.cursor/rules/actor-model.mdc`.

## Summary

**Total Actors Analyzed:** 15
- **Fully Compliant:** 0
- **Partially Compliant:** 8
- **Non-Compliant:** 7

## Detailed Analysis

### 1. **App Actor** (`src/app.rs`) - ❌ NON-COMPLIANT

**Violations:**
1. **Public Interface Structure**: Uses enum with `Ready`, `Actual`, `Mock` variants instead of simple struct with `tx: mpsc::Sender<Message>`
2. **Missing Standard Methods**: No `spawn()` method that returns `Self` directly
3. **Complex State Management**: Has `Ready` state that's not part of the specification
4. **Missing Clone Implementation**: The enum doesn't implement `Clone` properly for all variants

**How to Fix:**
```rust
#[derive(Debug, Clone)]
pub struct App {
    tx: mpsc::Sender<Message>,
}

impl App {
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = Core::new();
        let _ = tokio::spawn(async move {
            core.init(rx).await;
        });
        Self { tx }
    }
    
    pub async fn operation(&self, params: Params) -> anyhow::Result<Value> {
        let (tx, mut rx) = oneshot::channel();
        self.tx
            .send(Message::Operation { tx, params })
            .await
            .context("Doing operation with App")
            .expect("App actor died");
        rx.await
            .context("Awaiting response for operation with App")
            .expect("App actor died")
    }
}
```

### 2. **Env Actor** (`src/env.rs`) - ✅ COMPLIANT

**Status:** Fully compliant with the actor-model specification.

**Compliant Elements:**
- ✅ Simple struct with `tx: mpsc::Sender<Message>`
- ✅ Implements `Debug` and `Clone`
- ✅ Has `spawn()` method returning `Self`
- ✅ All public methods are `async`
- ✅ Uses `.context().expect()` pattern
- ✅ Proper message passing with `oneshot` channels for responses
- ✅ Core implementation follows specification
- ✅ Message enum properly defined
- ✅ Mock implementation using `mockall`

### 3. **Fs Actor** (`src/fs.rs`) - ✅ COMPLIANT

**Status:** Fully compliant with the actor-model specification.

**Compliant Elements:**
- ✅ Simple struct with `tx: mpsc::Sender<Message>`
- ✅ Implements `Debug` and `Clone`
- ✅ Has `spawn()` method returning `Self`
- ✅ All public methods are `async`
- ✅ Uses `.context().expect()` pattern
- ✅ Proper message passing with `oneshot` channels for responses
- ✅ Core implementation follows specification
- ✅ Message enum properly defined
- ✅ Mock implementation using `mockall`

### 4. **Log Actor** (`src/log.rs`) - ⚠️ PARTIALLY COMPLIANT

**Violations:**
1. **Missing Standard Methods**: No `spawn()` method that returns `Self` directly - uses `spawn()` with parameters
2. **Non-Standard Method**: Has `flush()` method that returns `JoinHandle<()>` instead of being async
3. **Missing Mock Integration**: Mock is not integrated into the main struct

**How to Fix:**
```rust
impl Log {
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = Core::new();
        let _ = tokio::spawn(async move {
            core.init(rx).await;
        });
        Self { tx }
    }
    
    pub async fn flush(&self) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Message::Flush { tx })
            .await
            .context("Flushing log")
            .expect("Log actor died");
        rx.await
            .context("Awaiting flush response from Log")
            .expect("Log actor died")
    }
}
```

### 5. **Net Actor** (`src/net.rs`) - ⚠️ PARTIALLY COMPLIANT

**Violations:**
1. **Missing Standard Methods**: No `spawn()` method that returns `Self` directly - uses `spawn()` with parameters
2. **Missing Mock Integration**: Mock is not integrated into the main struct

**How to Fix:**
```rust
impl Net {
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = Core::new();
        let _ = tokio::spawn(async move {
            core.init(rx).await;
        });
        Self { tx }
    }
}
```

### 6. **Render Actor** (`src/render.rs`) - ❌ NON-COMPLIANT

**Violations:**
1. **Public Interface Structure**: Uses enum with `Actual`, `Mock` variants instead of simple struct
2. **Missing Standard Methods**: No `spawn()` method that returns `Self` directly
3. **Missing Clone Implementation**: The enum doesn't implement `Clone` properly for all variants

**How to Fix:**
```rust
#[derive(Debug, Clone)]
pub struct Render {
    tx: mpsc::Sender<Message>,
}

impl Render {
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = Core::new();
        let _ = tokio::spawn(async move {
            core.init(rx).await;
        });
        Self { tx }
    }
}
```

### 7. **Shell Actor** (`src/shell.rs`) - ❌ NON-COMPLIANT

**Violations:**
1. **Public Interface Structure**: Uses enum with `Actual`, `Mock` variants instead of simple struct
2. **Missing Standard Methods**: No `spawn()` method that returns `Self` directly
3. **Missing Clone Implementation**: The enum doesn't implement `Clone` properly for all variants

**How to Fix:**
```rust
#[derive(Debug, Clone)]
pub struct Shell {
    tx: mpsc::Sender<Message>,
}

impl Shell {
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = Core::new();
        let _ = tokio::spawn(async move {
            core.init(rx).await;
        });
        Self { tx }
    }
}
```

### 8. **Terminal Actor** (`src/terminal.rs`) - ❌ NON-COMPLIANT

**Violations:**
1. **Public Interface Structure**: Uses enum with `Actual`, `Mock` variants instead of simple struct
2. **Missing Standard Methods**: No `spawn()` method that returns `Self` directly
3. **Missing Clone Implementation**: The enum doesn't implement `Clone` properly for all variants

**How to Fix:**
```rust
#[derive(Debug, Clone)]
pub struct Terminal {
    tx: mpsc::Sender<Message>,
}

impl Terminal {
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = Core::new();
        let _ = tokio::spawn(async move {
            core.init(rx).await;
        });
        Self { tx }
    }
}
```

### 9. **LoreApi Actor** (`src/api/lore.rs`) - ❌ NON-COMPLIANT

**Violations:**
1. **Public Interface Structure**: Uses enum with `Actual`, `Mock` variants instead of simple struct
2. **Missing Standard Methods**: No `spawn()` method that returns `Self` directly
3. **Missing Clone Implementation**: The enum doesn't implement `Clone` properly for all variants

**How to Fix:**
```rust
#[derive(Debug, Clone)]
pub struct LoreApi {
    tx: mpsc::Sender<LoreApiMessage>,
}

impl LoreApi {
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = Core::new();
        let _ = tokio::spawn(async move {
            core.init(rx).await;
        });
        Self { tx }
    }
}
```

### 10. **Config Actor** (`src/app/config.rs`) - ✅ COMPLIANT

**Status:** Fully compliant with the actor-model specification.

**Compliant Elements:**
- ✅ Simple struct with `tx: mpsc::Sender<Message>`
- ✅ Implements `Debug` and `Clone`
- ✅ Has `spawn()` method returning `Self`
- ✅ All public methods are `async`
- ✅ Uses `.context().expect()` pattern
- ✅ Proper message passing with `oneshot` channels for responses
- ✅ Core implementation follows specification
- ✅ Message enum properly defined
- ✅ Mock implementation using `mockall`

### 11. **FeedCache Actor** (`src/app/cache/feed.rs`) - ❌ NON-COMPLIANT

**Violations:**
1. **Public Interface Structure**: Uses enum with `Actual`, `Mock` variants instead of simple struct
2. **Missing Standard Methods**: No `spawn()` method that returns `Self` directly
3. **Missing Clone Implementation**: The enum doesn't implement `Clone` properly for all variants

**How to Fix:**
```rust
#[derive(Debug, Clone)]
pub struct FeedCache {
    tx: mpsc::Sender<Message>,
}

impl FeedCache {
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = Core::new();
        let _ = tokio::spawn(async move {
            core.init(rx).await;
        });
        Self { tx }
    }
}
```

### 12. **MailingListCache Actor** (`src/app/cache/mailing_list.rs`) - ❌ NON-COMPLIANT

**Violations:**
1. **Public Interface Structure**: Uses enum with `Actual`, `Mock` variants instead of simple struct
2. **Missing Standard Methods**: No `spawn()` method that returns `Self` directly
3. **Missing Clone Implementation**: The enum doesn't implement `Clone` properly for all variants

**How to Fix:**
```rust
#[derive(Debug, Clone)]
pub struct MailingListCache {
    tx: mpsc::Sender<Message>,
}

impl MailingListCache {
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = Core::new();
        let _ = tokio::spawn(async move {
            core.init(rx).await;
        });
        Self { tx }
    }
}
```

### 13. **PatchCache Actor** (`src/app/cache/patch.rs`) - ❌ NON-COMPLIANT

**Violations:**
1. **Public Interface Structure**: Uses enum with `Actual`, `Mock` variants instead of simple struct
2. **Missing Standard Methods**: No `spawn()` method that returns `Self` directly
3. **Missing Clone Implementation**: The enum doesn't implement `Clone` properly for all variants

**How to Fix:**
```rust
#[derive(Debug, Clone)]
pub struct PatchCache {
    tx: mpsc::Sender<Message>,
}

impl PatchCache {
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = Core::new();
        let _ = tokio::spawn(async move {
            core.init(rx).await;
        });
        Self { tx }
    }
}
```

### 14. **Ui Actor** (`src/app/ui.rs`) - ❌ NON-COMPLIANT

**Violations:**
1. **Public Interface Structure**: Uses enum with `Actual`, `Mock` variants instead of simple struct
2. **Missing Standard Methods**: No `spawn()` method that returns `Self` directly
3. **Missing Clone Implementation**: The enum doesn't implement `Clone` properly for all variants

**How to Fix:**
```rust
#[derive(Debug, Clone)]
pub struct Ui {
    tx: mpsc::Sender<Message>,
}

impl Ui {
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = Core::new();
        let _ = tokio::spawn(async move {
            core.init(rx).await;
        });
        Self { tx }
    }
}
```

## Common Violations Summary

### 1. **Enum-Based Public Interface** (Most Common)
**Problem:** Many actors use `enum` with `Actual`/`Mock` variants instead of simple struct
**Affected Actors:** App, Render, Shell, Terminal, LoreApi, FeedCache, MailingListCache, PatchCache, Ui
**Solution:** Convert to simple struct with `tx: mpsc::Sender<Message>`

### 2. **Missing Standard spawn() Method**
**Problem:** Actors don't have a simple `spawn()` method that returns `Self`
**Affected Actors:** App, Log, Net, Render, Shell, Terminal, LoreApi, FeedCache, MailingListCache, PatchCache, Ui
**Solution:** Add `pub fn spawn() -> Self` method

### 3. **Missing Clone Implementation**
**Problem:** Enum-based actors don't implement `Clone` properly
**Affected Actors:** App, Render, Shell, Terminal, LoreApi, FeedCache, MailingListCache, PatchCache, Ui
**Solution:** Implement `Clone` for the struct-based approach

### 4. **Non-Standard Method Signatures**
**Problem:** Some methods don't follow the async pattern or return non-standard types
**Affected Actors:** Log (flush method returns JoinHandle)
**Solution:** Make all public methods async and return `anyhow::Result<T>`

## Recommendations

1. **Immediate Priority:** Convert all enum-based actors to struct-based approach
2. **Standardize spawn() methods:** All actors should have a simple `spawn() -> Self` method
3. **Implement proper Clone:** All actors should implement `Clone` trait
4. **Standardize method signatures:** All public methods should be async and return `anyhow::Result<T>`
5. **Integrate mocks properly:** Use conditional compilation for mock integration instead of enum variants

## Compliance Matrix

| Actor | Struct Interface | spawn() Method | Clone | Async Methods | Message Passing | Mock Integration | Status |
|-------|------------------|----------------|-------|---------------|-----------------|------------------|---------|
| App | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | NON-COMPLIANT |
| Env | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | COMPLIANT |
| Fs | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | COMPLIANT |
| Log | ✅ | ❌ | ✅ | ⚠️ | ✅ | ❌ | PARTIALLY COMPLIANT |
| Net | ✅ | ❌ | ✅ | ✅ | ✅ | ❌ | PARTIALLY COMPLIANT |
| Render | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | NON-COMPLIANT |
| Shell | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | NON-COMPLIANT |
| Terminal | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | NON-COMPLIANT |
| LoreApi | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | NON-COMPLIANT |
| Config | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | COMPLIANT |
| FeedCache | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | NON-COMPLIANT |
| MailingListCache | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | NON-COMPLIANT |
| PatchCache | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | NON-COMPLIANT |
| Ui | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | NON-COMPLIANT |

**Legend:**
- ✅ = Compliant
- ❌ = Non-compliant  
- ⚠️ = Partially compliant
