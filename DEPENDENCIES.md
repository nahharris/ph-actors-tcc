# Actor Dependencies

This document describes the message dependencies between actors in the Patch Hub codebase. Each edge in the diagram represents a distinct message type sent from one actor to another.

## Dependency Diagram

```mermaid
graph TD
    %% Core System Actors
    Env[Env]
    Fs[Fs]
    Log[Log]
    Net[Net]
    Shell[Shell]
    Terminal[Terminal]

    %% Application Actors
    Config[Config]
    Render[Render]
    LoreApi[LoreApi]
    App[App]
    Ui[Ui]

    %% Cache Actors
    MailingListCache[MailingListCache]
    PatchCache[PatchCache]
    FeedCache[FeedCache]

    %% Log Actor Dependencies
    Log -->|GetLogLevel| Config
    Log -->|GetUSize| Config
    Log -->|GetPath| Config
    Log -->|ReadFile| Fs
    Log -->|WriteFile| Fs
    Log -->|MkDir| Fs
    Log -->|ReadDir| Fs
    Log -->|RemoveFile| Fs

    %% Shell Actor Dependencies
    Shell -->|Log| Log

    %% Config Actor Dependencies
    Config -->|ReadFile| Fs
    Config -->|WriteFile| Fs

    %% Render Actor Dependencies
    Render -->|GetRenderer| Config
    Render -->|Execute| Shell

    %% LoreApi Actor Dependencies
    LoreApi -->|Get| Net

    %% MailingListCache Actor Dependencies
    MailingListCache -->|GetPath| Config
    MailingListCache -->|GetAvailableListsPage| LoreApi
    MailingListCache -->|MkDir| Fs
    MailingListCache -->|WriteFile| Fs
    MailingListCache -->|ReadFile| Fs
    MailingListCache -->|Log| Log

    %% PatchCache Actor Dependencies
    PatchCache -->|GetPath| Config
    PatchCache -->|GetRawPatch| LoreApi
    PatchCache -->|ReadFile| Fs
    PatchCache -->|WriteFile| Fs
    PatchCache -->|MkDir| Fs
    PatchCache -->|RemoveFile| Fs
    PatchCache -->|Log| Log

    %% FeedCache Actor Dependencies
    FeedCache -->|GetPath| Config
    FeedCache -->|GetPatchFeedPage| LoreApi
    FeedCache -->|MkDir| Fs
    FeedCache -->|WriteFile| Fs
    FeedCache -->|ReadFile| Fs
    FeedCache -->|Log| Log

    %% App Actor Dependencies
    App -->|Persist| MailingListCache
    App -->|Persist| FeedCache
    App -->|Log| Log
    App -->|ShowLists| Ui
    App -->|GetUiEvent| Terminal

    %% Ui Actor Dependencies
    Ui -->|Show| Terminal
    Ui -->|GetSlice| MailingListCache
    Ui -->|Len| MailingListCache
    Ui -->|Refresh| MailingListCache
    Ui -->|GetSlice| FeedCache
    Ui -->|Len| FeedCache
    Ui -->|Refresh| FeedCache
    Ui -->|IsAvailable| FeedCache
    Ui -->|Load| FeedCache
    Ui -->|EnsureLoaded| FeedCache
    Ui -->|Persist| FeedCache
    Ui -->|Invalidate| FeedCache
    Ui -->|Get| PatchCache
    Ui -->|Render| Render
    Ui -->|Log| Log

    style Env fill:#e1f5ff
    style Fs fill:#e1f5ff
    style Log fill:#e1f5ff
    style Net fill:#e1f5ff
    style Shell fill:#e1f5ff
    style Terminal fill:#e1f5ff
    style Config fill:#fff4e1
    style Render fill:#fff4e1
    style LoreApi fill:#fff4e1
    style App fill:#fff4e1
    style Ui fill:#fff4e1
    style MailingListCache fill:#e8f5e9
    style PatchCache fill:#e8f5e9
    style FeedCache fill:#e8f5e9
```

## Legend

- **Blue nodes**: Core system actors (Env, Fs, Log, Net, Shell, Terminal)
- **Orange nodes**: Application actors (Config, Render, LoreApi, App, Ui)
- **Green nodes**: Cache actors (MailingListCache, PatchCache, FeedCache)

## Message Types

### Log Actor Messages
- `Log` - Logs a message (used by info, warn, error methods)

### Config Actor Messages
- `GetLogLevel` - Get the current log level
- `GetUSize` - Get a numeric configuration value
- `GetPath` - Get a path-based configuration value
- `GetRenderer` - Get renderer configuration

### Fs Actor Messages
- `ReadFile` - Open a file for reading
- `WriteFile` - Open a file for writing
- `AppendFile` - Open a file for appending
- `RemoveFile` - Remove a file
- `ReadDir` - Read directory contents
- `MkDir` - Create directory and parents
- `RmDir` - Remove directory

### Shell Actor Messages
- `Execute` - Execute an external program

### Net Actor Messages
- `Get` - HTTP GET request
- `Post` - HTTP POST request
- `Put` - HTTP PUT request
- `Delete` - HTTP DELETE request
- `Patch` - HTTP PATCH request

### LoreApi Actor Messages
- `GetPatchFeedPage` - Get patch feed page
- `GetAvailableListsPage` - Get available lists page
- `GetAvailableLists` - Get all available lists
- `GetPatchHtml` - Get patch HTML content
- `GetRawPatch` - Get raw patch content
- `GetPatchMetadata` - Get patch metadata

### Terminal Actor Messages
- `Show` - Render a screen
- `GetUiEvent` - Get next UI event
- `ClearUiEvents` - Clear UI events queue
- `Quit` - Quit the terminal

### Ui Actor Messages
- `ShowLists` - Show mailing lists view
- `ShowFeed` - Show patch feed view
- `ShowPatch` - Show patch content view
- `UpdateSelection` - Update selection index
- `PreviousPage` - Navigate to previous page
- `NextPage` - Navigate to next page
- `NavigateBack` - Navigate back
- `SubmitSelection` - Submit current selection
- `GetState` - Get current UI state

### MailingListCache Actor Messages
- `Get` - Get a mailing list by index
- `GetSlice` - Get a slice of mailing lists
- `Refresh` - Refresh cache from API
- `Invalidate` - Invalidate cache
- `IsAvailable` - Check if range is available
- `Len` - Get cache length
- `Persist` - Persist cache to disk
- `Load` - Load cache from disk

### PatchCache Actor Messages
- `Get` - Get a patch by list and message ID
- `Invalidate` - Invalidate a specific patch
- `IsAvailable` - Check if patch is available

### FeedCache Actor Messages
- `Get` - Get patch metadata by index
- `GetSlice` - Get a slice of patch metadata
- `Refresh` - Refresh cache from API
- `Invalidate` - Invalidate cache
- `IsAvailable` - Check if range is available
- `Len` - Get cache length
- `Persist` - Persist cache to disk
- `Load` - Load cache from disk
- `IsLoaded` - Check if cache is loaded
- `EnsureLoaded` - Ensure cache is loaded from disk

### App Actor Messages
- `Shutdown` - Shutdown the application

## Notes

1. Messages are traced from each actor's `init()` method and all methods called from `init()` (directly or indirectly).

2. The diagram shows one edge per distinct message type. For example, `Log` actor's `info()`, `warn()`, and `error()` methods all send the same `Log` message type with different log levels, so there is only one edge from actors to `Log`.

3. Some actors don't send messages to other actors (they only receive):
   - `Env` - Only receives messages
   - `Fs` - Only receives messages
   - `Net` - Only receives messages

4. Actors that don't appear in the diagram as targets of edges are terminal actors that don't receive messages from other actors, or they receive messages but those sending actors aren't part of the dependency graph from `init()` methods.
