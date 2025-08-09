# App Actor

The App actor is the central coordinator for the entire Patch Hub application. It manages application state and coordinates all other actors to provide a unified interface for command execution, cache management, and UI coordination.

## Scope and Responsibilities

### What the App Actor Does:
- **Application Lifecycle Management**: Initializes and coordinates all system actors (env, fs, config, log, net, lore, shell, render)
- **Command Execution**: Handles CLI commands (Lists, Feed, Patch) by orchestrating appropriate actors
- **Cache Management**: Manages cache lifecycle including loading, persistence, and invalidation
- **TUI Coordination**: Manages Terminal User Interface mode and coordinates with UI components
- **State Management**: Maintains application state and current operation context
- **Graceful Shutdown**: Ensures proper cleanup and cache persistence on application exit

### What the App Actor Does NOT Do:
- Direct HTTP requests (delegates to Net/LoreApi actors)
- File system operations (delegates to Fs actor)  
- Environment variable access (delegates to Env actor)
- Logging operations (delegates to Log actor)
- Shell command execution (delegates to Shell actor)
- Patch rendering (delegates to Render actor)
- Terminal I/O operations (delegates to Terminal actor)

## Dependencies

The App actor depends on and coordinates the following actors:

### Core System Actors:
- **Env**: Environment variable management
- **Fs**: File system operations
- **Config**: Configuration management
- **Log**: Application logging

### Service Actors:
- **Net**: HTTP networking operations
- **LoreApi**: Lore Kernel Archive API interface
- **Shell**: External command execution
- **Render**: Patch content rendering

### Cache Actors:
- **MailingListCache**: Caching for mailing list data
- **PatchMetaCache**: Caching for patch metadata

### UI Actors (TUI mode):
- **Terminal**: Terminal I/O management
- **AppUi**: Application user interface

## Capabilities

### Command Execution
- **Lists Command**: Displays paginated mailing lists using cache
- **Feed Command**: Shows patch feed for a specific mailing list
- **Patch Command**: Retrieves and displays patch content (raw or HTML)

### TUI Mode
- Launches interactive terminal user interface
- Coordinates between Terminal and AppUi actors
- Handles user input and screen transitions

### Cache Management
- Automatic cache loading on startup
- Cache persistence on shutdown
- Cache invalidation when needed
- Error handling for cache operations

### Application State
- Tracks initialization status
- Maintains current command context
- Provides state for debugging and monitoring

## Extending the App Actor

### Adding New Commands
1. Add new variant to `Command` enum in `app/data.rs`
2. Add handling logic in `Core::handle_execute_command()`
3. Update message handling in the actor spawn loop

### Adding New Actors
1. Add actor field to `Core` struct
2. Initialize actor in `Core::build()` method
3. Use actor in appropriate command handlers

### Adding New Cache Types
1. Create new cache actor following cache patterns
2. Add cache field to `Core` struct
3. Initialize and manage cache in `Core::build()` and `Core::handle_shutdown()`

### Adding New UI Modes
1. Add new message variant in `message.rs`
2. Add handler method in `Core`
3. Add public method in `App` interface
4. Coordinate with appropriate UI actors

## Error Handling

The App actor uses `anyhow::Result` for comprehensive error handling:

- **Initialization Errors**: Propagated from `build()` method
- **Command Execution Errors**: Captured and returned to caller
- **Cache Errors**: Logged as warnings, don't fail the application
- **Actor Communication Errors**: Use `.context().expect()` pattern assuming actors don't die

## Testing

The App actor provides comprehensive mock support:

### Mock Capabilities:
- Track executed commands
- Simulate TUI mode execution
- Verify shutdown calls
- Maintain mock application state

### Mock Usage:
```rust
let mock_data = MockData::default();
let app = App::mock(mock_data);
app.execute_command(Command::Lists { page: 0, count: 10 }).await?;
```

## Performance Characteristics

- **Async Operation**: All operations are fully async using Tokio
- **Message Passing**: Thread-safe communication through message channels
- **Resource Pooling**: Reuses actor connections and resources
- **Cache Efficiency**: Minimizes API calls through intelligent caching
- **Memory Efficiency**: Uses Arc types for cheap cloning across threads

## Configuration

The App actor respects all configuration options through the Config actor:
- Logging levels and directories
- Network timeouts and retry settings
- Cache sizes and TTL settings
- Path configurations for data storage
