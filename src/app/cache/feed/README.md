# Feed Actor

## Scope and Responsibilities

The Feed Actor is responsible for caching and managing patch metadata for individual mailing lists from the Lore Kernel Archive. It provides a thread-safe interface for accessing patch feeds with the following responsibilities:

- **Per-List Caching**: Caches patch metadata separately for each mailing list
- **Smart Pagination**: Fetches data on demand with intelligent cache validation
- **Cache Management**: Provides in-memory caching with filesystem persistence
- **Cache Validation**: Validates cache freshness based on the 0-th item's updated time
- **Smart Fetching**: Intelligently refreshes cache by fetching until finding existing items

## Dependencies

The Feed Actor depends on:

- **LoreApi**: For fetching patch metadata from the Lore Kernel Archive
- **Fs**: For filesystem operations (reading/writing cache files)
- **Config**: For configuration management (cache directory paths)
- **Log**: For logging operations and debugging

## Capabilities

### Core Operations

1. **Get by Index**: Retrieve a single patch metadata item by index for a mailing list
2. **Get Slice**: Retrieve a range of patch metadata items for pagination
3. **Refresh**: Force refresh the cache for a specific mailing list
4. **Invalidate**: Clear the cache for a specific mailing list
5. **Availability Check**: Check if requested data is available in cache
6. **Length**: Get the total number of cached items for a mailing list

### Cache Management

- **Persistence**: Caches data to `cache/feed/<list_name>.toml` files
- **Validation**: Validates cache based on 0-th item's `last_update` time
- **Smart Refresh**: Only fetches new data when cache is stale
- **Per-List Storage**: Each mailing list has its own cache file

### Performance Features

- **In-Memory Access**: Fast access to cached data
- **Lazy Loading**: Loads cache from disk on demand
- **Smart Fetching**: Fetches pages until finding existing 0-th item
- **Minimal API Calls**: Only calls API when necessary

## How to Extend Functionality

### Adding New Operations

1. **Add Message Variant** (`message.rs`):
   ```rust
   pub enum Message {
       // ... existing variants ...
       NewOperation {
           list: ArcStr,
           // other parameters
           tx: oneshot::Sender<Result<ReturnType, Error>>,
       },
   }
   ```

2. **Update Core Handler** (`core.rs`):
   ```rust
   match message {
       // ... existing matches ...
       Message::NewOperation { list, /* params */, tx } => {
           let result = core.handle_new_operation(&list, /* params */).await;
           let _ = tx.send(result);
       }
   }
   ```

3. **Implement Handler Method** (`core.rs`):
   ```rust
   async fn handle_new_operation(&self, list: &str, /* params */) -> anyhow::Result<ReturnType> {
       // Implementation
   }
   ```

4. **Add Public Interface** (`feed.rs`):
   ```rust
   pub async fn new_operation(&self, list: ArcStr, /* params */) -> anyhow::Result<ReturnType> {
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
- **Validation Logic**: Update `is_cache_valid()` method
- **Fetching Strategy**: Modify `refresh_cache()` method for different fetching patterns
- **Persistence Format**: Change `CacheData` structure and serialization

### Adding New Data Fields

1. **Update Data Structure** (`data.rs`):
   ```rust
   pub struct FeedData {
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
    feeds: HashMap::from([
        (ArcStr::from("test-list"), vec![/* test data */]),
    ]),
};
let cache = FeedCache::mock(mock_data);

// Test operations
let result = cache.get(ArcStr::from("test-list"), 0).await?;
assert_eq!(result.unwrap().title, "expected_title");
```

## Error Handling

The actor uses `anyhow::Result` for error propagation and provides meaningful error context:

- **API Errors**: Network failures, malformed responses
- **Filesystem Errors**: Permission issues, disk full
- **Serialization Errors**: Corrupted cache files
- **Actor Errors**: Channel communication failures

## Performance Considerations

- **Memory Usage**: Caches patch metadata per mailing list in memory
- **API Efficiency**: Minimizes API calls through smart validation
- **Disk I/O**: Only writes to disk when cache changes
- **Concurrency**: Thread-safe through actor pattern
- **Smart Fetching**: Stops fetching when existing items are found

## Cache Strategy

The Feed Actor uses a sophisticated caching strategy:

1. **Initial Load**: Loads cache from disk when first accessed
2. **Validation**: Checks if 0-th item's updated time matches API
3. **Smart Refresh**: If cache is stale, fetches pages until finding existing 0-th item
4. **Persistence**: Saves updated cache to disk after changes
5. **Per-List Isolation**: Each mailing list's cache is independent

This strategy ensures that:
- Cached items remain valid across sessions
- New items are fetched efficiently
- Disk space is used optimally
- API calls are minimized
