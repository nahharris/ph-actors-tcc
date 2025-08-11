# Mailing List Actor

## Scope and Responsibilities

The Mailing List Actor is responsible for caching and managing mailing lists from the Lore Kernel Archive. It provides a thread-safe interface for accessing mailing lists with the following responsibilities:

- **Data Retrieval**: Fetches all mailing lists from the Lore API
- **Alphabetical Sorting**: Maintains mailing lists in alphabetical order by name
- **Cache Management**: Provides in-memory caching with filesystem persistence
- **Cache Validation**: Validates cache freshness based on the 0-th item's updated time
- **Smart Fetching**: Intelligently refreshes cache when data is stale

## Dependencies

The Mailing List Actor depends on:

- **LoreApi**: For fetching mailing lists from the Lore Kernel Archive
- **Fs**: For filesystem operations (reading/writing cache files)
- **Config**: For configuration management (cache directory paths)
- **Log**: For logging operations and debugging

## Capabilities

### Core Operations

1. **Get by Index**: Retrieve a single mailing list by its index
2. **Get Slice**: Retrieve a range of mailing lists for pagination
3. **Refresh**: Force refresh the cache by fetching from API
4. **Invalidate**: Clear the cache and force reload
5. **Availability Check**: Check if requested data is available in cache
6. **Length**: Get the total number of cached mailing lists

### Cache Management

- **Persistence**: Caches data to `cache/mailing_lists.toml`
- **Validation**: Validates cache based on 0-th item's `last_update` time
- **Smart Refresh**: Only fetches new data when cache is stale
- **Alphabetical Order**: Maintains sorted order for consistent access

### Performance Features

- **In-Memory Access**: Fast access to cached data
- **Lazy Loading**: Loads cache from disk on startup
- **Efficient Sorting**: Sorts data once after fetching
- **Minimal API Calls**: Only calls API when necessary

## How to Extend Functionality

### Adding New Operations

1. **Add Message Variant** (`message.rs`):
   ```rust
   pub enum Message {
       // ... existing variants ...
       NewOperation {
           // parameters
           tx: oneshot::Sender<Result<ReturnType, Error>>,
       },
   }
   ```

2. **Update Core Handler** (`core.rs`):
   ```rust
   match message {
       // ... existing matches ...
       Message::NewOperation { /* params */, tx } => {
           let result = core.handle_new_operation(/* params */).await;
           let _ = tx.send(result);
       }
   }
   ```

3. **Implement Handler Method** (`core.rs`):
   ```rust
   async fn handle_new_operation(&self, /* params */) -> anyhow::Result<ReturnType> {
       // Implementation
   }
   ```

4. **Add Public Interface** (`mailing_list.rs`):
   ```rust
   pub async fn new_operation(&self, /* params */) -> anyhow::Result<ReturnType> {
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

- **Cache Path**: Modify `Core::new()` to change cache file location
- **Validation Logic**: Update `is_cache_valid()` method
- **Sorting**: Modify `refresh_cache()` method for different sorting
- **Persistence Format**: Change `CacheData` structure and serialization

### Adding New Data Fields

1. **Update Data Structure** (`data.rs`):
   ```rust
   pub struct MailingListData {
       // ... existing fields ...
       pub new_field: NewType,
   }
   ```

2. **Update Cache Data** (`data.rs`):
   ```rust
   pub struct CacheData {
       // ... existing fields ...
       pub new_field: NewType,
   }
   ```

3. **Update Methods**: Modify `to_cache_data()` and `from_cache_data()` methods

## Testing

The actor provides a mock implementation for testing:

```rust
let mock_data = MockData {
    mailing_lists: vec![/* test data */],
};
let cache = MailingListCache::mock(mock_data);

// Test operations
let result = cache.get(0).await?;
assert_eq!(result.unwrap().name, "expected_name");
```

## Error Handling

The actor uses `anyhow::Result` for error propagation and provides meaningful error context:

- **API Errors**: Network failures, malformed responses
- **Filesystem Errors**: Permission issues, disk full
- **Serialization Errors**: Corrupted cache files
- **Actor Errors**: Channel communication failures

## Performance Considerations

- **Memory Usage**: Caches all mailing lists in memory for fast access
- **API Efficiency**: Minimizes API calls through smart validation
- **Disk I/O**: Only writes to disk when cache changes
- **Concurrency**: Thread-safe through actor pattern
