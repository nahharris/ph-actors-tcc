# UI Actor

The UI actor manages the Terminal User Interface (TUI) for Patch Hub. It handles view state, navigation between screens, and coordinates with the Terminal actor for rendering different views (Lists, Feed, Patch).

## Scope and Responsibilities

### What the UI Actor Does:
- **View State Management**: Maintains current view, page numbers, and selection indices
- **Screen Rendering**: Renders Lists, Feed, and Patch screens through the Terminal actor
- **Navigation Logic**: Handles transitions between different views and pages
- **Selection Management**: Tracks and updates current selection in each view
- **Data Coordination**: Fetches data from caches and API for rendering
- **Error Handling**: Displays error screens and handles cache invalidation

### What the UI Actor Does NOT Do:
- Direct terminal I/O operations (delegates to Terminal actor)
- HTTP requests (delegates to LoreApi actor)  
- File system operations (delegates to cache actors)
- Logging operations (delegates to Log actor)
- Patch rendering (delegates to Render actor)
- Key event capture (handled by Terminal and App actors)

## Dependencies

The UI actor depends on and coordinates with the following actors:

### Core Actors:
- **Log**: Application logging for UI events and debugging
- **Terminal**: Terminal I/O operations and screen rendering

### Service Actors:
- **LoreApi**: Fetching patch content from the Lore Kernel Archive
- **Render**: Rendering raw patch content for display

### Cache Actors:
- **MailingListCache**: Cached mailing list data for Lists view
- **PatchMetaCache**: Cached patch metadata for Feed view

## Capabilities

### View Management
- **Lists View**: Displays paginated mailing lists with selection
- **Feed View**: Shows patch feed for a specific mailing list
- **Patch View**: Displays individual patch content

### Navigation
- **Page Navigation**: Previous/next page within current view
- **View Transitions**: Navigate between Lists -> Feed -> Patch
- **Back Navigation**: Return to previous view (Patch -> Feed -> Lists)
- **Selection Handling**: Track and update current selection

### Data Integration
- **Cache Coordination**: Fetches data from appropriate caches
- **Loading States**: Shows loading screens during data fetching
- **Error Recovery**: Handles errors with cache invalidation and error screens

### State Management
- **View State**: Current view type (Lists, Feed, Patch)
- **Pagination State**: Current page and selection for each view
- **Context State**: Current mailing list when in Feed/Patch views

## Extending the UI Actor

### Adding New Views
1. Add new variant to `ViewKind` enum in `ui/data.rs`
2. Update `UiState` struct with new view-specific state
3. Add new message variants in `ui/message.rs` for the view
4. Implement rendering logic in `ui/core.rs`
5. Add public methods in `ui.rs` interface

### Adding New Navigation Actions
1. Add new variant to `NavigationAction` enum in `ui/message.rs`
2. Update `handle_submit_selection()` in `ui/core.rs` to return new action
3. Handle new action in App actor's key event processing

### Adding New UI Operations
1. Add new message variant in `ui/message.rs`
2. Add handler method in `ui/core.rs`
3. Add public method in `ui.rs` interface
4. Update mock implementation for testing

### Customizing Screen Layout
- Modify rendering logic in `ui/core.rs`
- Update Terminal Screen variants if needed
- Adjust pagination logic (currently 20 items per page)

## Error Handling

The UI actor uses `anyhow::Result` for comprehensive error handling:

- **Rendering Errors**: Displays error screens and logs issues
- **Data Fetch Errors**: Invalidates caches and shows appropriate messages
- **Navigation Errors**: Gracefully handles invalid state transitions
- **Actor Communication Errors**: Uses `.context().expect()` pattern

## Testing

The UI actor provides comprehensive mock support:

### Mock Capabilities:
- Track rendered screens and navigation actions
- Simulate view state changes
- Verify UI method calls and state transitions

### Mock Usage:
```rust
let mock_data = MockData::default();
let ui = Ui::mock(mock_data);
ui.show_lists(0).await?;
ui.update_selection(2);
let action = ui.submit_selection().await?;
```

## Performance Characteristics

- **Async Operation**: All operations are fully async using Tokio
- **Message Passing**: Thread-safe communication through message channels
- **Lazy Rendering**: Only renders when explicitly requested
- **Cache Efficiency**: Leverages caches to minimize API calls
- **Memory Efficiency**: Uses Arc types for cheap string/data cloning

## UI Flow

### Typical Navigation Flow:
1. **Start**: Lists view (page 0, selection 0)
2. **Select List**: Navigate to Feed view for selected mailing list
3. **Select Patch**: Navigate to Patch view for selected patch
4. **Back Navigation**: Patch -> Feed -> Lists -> Quit

### Key Event Mapping (handled by App actor):
- **Up/Down**: Update selection within current view
- **Left/Right**: Navigate between pages
- **Enter**: Submit current selection (navigate to next view)
- **Esc**: Navigate back to previous view or quit

## Configuration

The UI actor respects configuration through its dependencies:
- Terminal rendering settings through Terminal actor
- Logging levels through Log actor  
- Cache behavior through cache actors
- Network timeouts through LoreApi actor
