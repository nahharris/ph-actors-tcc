# Terminal Actor

The Terminal actor provides a thread-safe interface to terminal UI operations using the Cursive library.

## Scope and Responsibilities

**What this actor does:**
- Manages the Cursive TUI (Terminal User Interface) event loop
- Renders different screen types (lists, feeds, patches, loading, error)
- Forwards UI events (key presses, selections) to the application layer
- Provides a message-based interface for updating the UI from other actors

**What this actor does NOT do:**
- Business logic or data processing
- Network requests or file operations
- Application state management
- Direct interaction with other system resources

## Dependencies

This actor depends on:
- **Log actor**: For recording terminal operations and events
- **UI Events Channel**: `mpsc::Sender<UiEvent>` for forwarding user input events to the application
- **Cursive library**: For terminal UI rendering and event handling

## Capabilities

### Screen Rendering
- **Lists Screen**: Display paginated mailing lists with selection support
- **Feed Screen**: Show patch feeds for specific mailing lists
- **Patch Screen**: Render individual patch content with scrolling
- **Loading Screen**: Display loading messages during operations
- **Error Screen**: Show error messages to the user

### Event Handling
- **Navigation**: Left/Right arrow keys for pagination
- **Selection**: Up/Down arrows and Enter for list navigation
- **Exit**: Escape key for returning to previous screens
- **Custom Events**: Selection change and submit events

### Actor Operations
- **show()**: Update the displayed screen
- **quit()**: Terminate the UI and exit the application

## Architecture

The terminal actor uses a hybrid approach to satisfy both the actor pattern requirements and Cursive's threading needs:

1. **Main Actor Task**: A tokio task that handles incoming messages and manages the UI state
2. **Cursive Thread**: A separate OS thread that runs the Cursive event loop (required by Cursive)
3. **Communication Bridge**: The actor task communicates with the Cursive thread via callback sinks

This design ensures:
- Compliance with the actor pattern (main logic in tokio task)
- Proper Cursive operation (UI thread for blocking terminal I/O)
- Thread-safe message passing between components

## Extending Functionality

To add new capabilities to the terminal actor:

### Adding New Screen Types
1. Add a new variant to the `Screen` enum in `data.rs`
2. Update the `handle_show_screen` method in `core.rs` to render the new screen type
3. Add any necessary UI event handling for the new screen

### Adding New UI Events
1. Add a new variant to the `UiEvent` enum in `data.rs`
2. Register the event handler in the Cursive setup (in `core.rs`)
3. Update the application layer to handle the new event type

### Adding New Operations
1. Add a new message variant to the `Message` enum in `message.rs`
2. Handle the new message in the actor's message loop
3. Add a corresponding public method to the `Terminal` enum
4. Update the mock implementation if needed

## Testing

The actor provides comprehensive mock support:
- **Mock Implementation**: `Terminal::Mock(Arc<Mutex<MockData>>)` for testing
- **State Tracking**: Mock tracks last shown screen and quit status
- **Deterministic Behavior**: All mock operations complete synchronously

Use `Terminal::mock(MockData::default())` to create a mock instance for testing.
