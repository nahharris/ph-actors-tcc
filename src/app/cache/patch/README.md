# Patch Actor

## Scope and Responsibilities

The Patch Actor is responsible for caching and managing individual patch content from the Lore Kernel Archive. It provides a thread-safe interface for accessing raw patch content with the following responsibilities:

- **Individual Patch Caching**: Caches raw patch content for each patch separately
- **Permanent Cache Validity**: Once cached, patches are considered valid forever
- **In-Memory Buffer**: Provides fast access to recently used patches
- **Filesystem Persistence**: Stores patches as individual .mbox files
- **Smart Fetching**: Fetches from API only when not cached

## Dependencies

The Patch Actor depends on:

- **LoreApi**: For fetching raw patch content from the Lore Kernel Archive
- **Fs**: For filesystem operations (reading/writing patch files)
- **Config**: For configuration management (cache directory paths)
- **Log**: For logging operations and debugging
- **lru**: For in-memory LRU cache implementation

## Capabilities

### Core Operations

1. **Get**: Retrieve raw patch content by mailing list and message ID
2. **Invalidate**: Remove a specific patch from cache
3. **Availability Check**: Check if a patch is available in cache

### Cache Management

- **Persistence**: Caches data to `cache/patch/<list_name>/<message_id>.mbox` files
- **Permanent Validity**: Cached patches never expire
- **LRU Buffer**: Small in-memory buffer for fast access
- **Individual Storage**: Each patch is stored as a separate file

### Performance Features

- **In-Memory Access**: Fast access to recently used patches via LRU buffer
- **Lazy Loading**: Loads patches from disk on demand
- **Permanent Cache**: Once fetched, patches are cached forever
- **Minimal API Calls**: Only calls API when patch is not cached

## How to Extend Functionality

### Adding New Operations

1. **Add Message Variant** (`message.rs`):
   ```rust
   pub enum Message {
       // ... existing variants ...
       NewOperation {
           list: ArcStr,
           message_id: ArcStr,
           // other parameters
           tx: oneshot::Sender<Result<ReturnType, Error>>,
       },
   }
   ```

2. **Update Core Handler** (`core.rs`):
   ```rust
   match message {
       // ... existing matches ...
       Message::NewOperation { list, message_id, /* params */, tx } => {
           let result = core.handle_new_operation(&list, &message_id, /* params */).await;
           let _ = tx.send(result);
       }
   }
   ```

3. **Implement Handler Method** (`core.rs`):
   ```rust
   async fn handle_new_operation(&self, list: &str, message_id: &str, /* params */) -> anyhow::Result<ReturnType> {
       // Implementation
   }
   ```

4. **Add Public Interface** (`patch.rs`):
   ```rust
   pub async fn new_operation(&self, list: ArcStr, message_id: ArcStr, /* params */) -> anyhow::Result<ReturnType> {
       match self {
           Self::Actual(sender) => {
               // Real implementation
           }
           Self::Mock(data) => {
               // Mock implementation
           }
       }
   }
   ```

### Modifying Cache Behavior

- **Cache Path**: Modify `Core::new()` to change cache directory structure
- **Buffer Size**: Change the LRU cache size in `PatchData::new()`
- **File Format**: Modify file extension and content format
- **Persistence Strategy**: Change how patches are stored and retrieved

### Adding New Data Fields

1. **Update Data Structure** (`data.rs`):
   ```rust
   pub struct PatchData {
       // ... existing fields ...
       pub new_field: NewType,
   }
   ```

2. **Update Methods**: Modify buffer management methods as needed

## Testing

The actor provides a mock implementation for testing:

```rust
let mock_data = MockData {
    patches: HashMap::from([
        ("test-list:test-message-id".to_string(), "patch content".to_string()),
    ]),
};
let cache = PatchCache::mock(mock_data);

// Test operations
let result = cache.get(ArcStr::from("test-list"), ArcStr::from("test-message-id")).await?;
assert_eq!(result, "patch content");
```

## Error Handling

The actor uses `anyhow::Result` for error propagation and provides meaningful error context:

- **API Errors**: Network failures, malformed responses
- **Filesystem Errors**: Permission issues, disk full
- **Actor Errors**: Channel communication failures
- **Cache Errors**: File corruption, missing files

## Performance Considerations

- **Memory Usage**: Small LRU buffer (50 items) for fast access
- **API Efficiency**: Only calls API when patch is not cached
- **Disk I/O**: Reads from disk when not in buffer
- **Concurrency**: Thread-safe through actor pattern
- **Permanent Cache**: Once fetched, patches never need re-fetching

## Cache Strategy

The Patch Actor uses a simple but effective caching strategy:

1. **Buffer Check**: First checks the in-memory LRU buffer
2. **Disk Check**: If not in buffer, checks if file exists on disk
3. **Load from Disk**: If file exists, loads content and adds to buffer
4. **API Fetch**: If not cached, fetches from API
5. **Save to Disk**: Saves fetched content to disk and adds to buffer

This strategy ensures that:
- Recently used patches are accessed instantly
- All patches are permanently cached on disk
- API calls are minimized
- Memory usage is bounded
- Disk space grows linearly with number of patches

## File Organization

Patches are organized on disk as follows:

```
cache/patch/
├── linux-kernel/
│   ├── 20241201.123456.mbox
│   ├── 20241201.234567.mbox
│   └── ...
├── amd-gfx/
│   ├── 20241201.345678.mbox
│   └── ...
└── ...
```

Each patch is stored as a separate .mbox file named with its message ID, organized by mailing list.
