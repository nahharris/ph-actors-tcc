## Motivation

The limitations described above provide a compelling case for architectural redesign. The Actor Model offers several advantages that address these issues:

### Concurrency and Isolation

The Actor Model naturally supports concurrent operations through message passing, allowing the application to handle multiple I/O operations simultaneously without blocking the user interface.

### Loose Coupling

Actors communicate exclusively through messages, creating clear boundaries between components and reducing coupling. This makes the system more modular and easier to test.

### State Isolation

Each actor manages its own state independently, eliminating the complex ownership issues that arise from shared mutable state in Rust.

### Extensibility

The message-based architecture makes it easier to add new features without extensive refactoring, as new actors can be added without modifying existing ones.

## Objectives

This work won't create a full rewrite of Patch-Hub with 100% feature compatibility. Instead, a subset of the features were picked for implementation, enough to have the core features of Patch-Hub which are:

- An interactive text user interface
- Integration with Lore API with cache support
- Navigate through the catalogued mailing lists
- Access to a feed of patches sent to a mailing list
- Provide a view to the content and metadata of a patch
- Customizable settings
- A logging system

## Primitive Actors

The forementioned features are composed of some primitive capabilities:

- File system operations
- HTTP requests
- Environment variables manipulation
- Terminal control
- Shell integration

Primitive capabilities can be understood as building blocks of the system that are not part of the business logic and rely mainly on interacting with external systems (networking, shell, files, etc). Each of those capabilities can be translated to an actor in our application following the SOLID principles, in special:

- **Single Responsibility**: each actor handles one and only one domain;
- **Interface Segregation**: by splitting into different actors, dependency injection is more granular so clients depend only on interfaces that matter for them.

Those actors will be called primitive actors since they do not depend on other actors.

### Filesystem Actor

The Filesystem actor provides thread-safe access to filesystem operations through the Actor Model pattern. It encapsulates all file and directory operations, providing a unified interface for both real filesystem access and mock implementations for testing.

**Messages:**

- **ReadFile**: Opens a file for reading only (does not create if it doesn't exist)
- **WriteFile**: Opens a file for writing (truncates content, creates if needed)
- **AppendFile**: Opens a file for appending (creates if needed)
- **RemoveFile**: Removes a file from the filesystem
- **ReadDir**: Reads the contents of a directory
- **MkDir**: Creates a directory and its parents
- **RmDir**: Removes a directory and its contents

### Network Actor

The Network actor provides thread-safe access to HTTP operations through the Actor Model pattern. It handles all HTTP requests using the reqwest library, providing a unified interface for both real network operations and mock implementations for testing.

**Messages:**

- **Get**: Performs an HTTP GET request to the specified URL with optional headers
- **Post**: Performs an HTTP POST request to the specified URL with optional headers and body
- **Put**: Performs an HTTP PUT request to the specified URL with optional headers and body
- **Delete**: Performs an HTTP DELETE request to the specified URL with optional headers
- **Patch**: Performs an HTTP PATCH request to the specified URL with optional headers and body

### Environment Variables Actor

The Environment actor provides thread-safe access to environment variable operations through the Actor Model pattern. It encapsulates all environment variable access, providing a unified interface for both real system environment access and mock implementations for testing.

**Messages:**

- **Set**: Sets an environment variable to a specified value
- **Unset**: Removes an environment variable
- **Get**: Retrieves an environment variable value (returns VarError if not found)

### Terminal Actor

The Terminal actor provides thread-safe access to terminal UI operations through the Actor Model pattern. It manages the Cursive TUI (Terminal User Interface) event loop and provides a message-based interface for updating the UI from other actors.

**Messages:**

- **Show**: Updates the displayed screen with a new screen type
- **Quit**: Terminates the UI and exits the application

### Shell Actor

The Shell actor provides thread-safe access to external program execution through the Actor Model pattern. It encapsulates all shell command execution, providing a unified interface for both real process execution and mock implementations for testing.

**Messages:**

- **Execute**: Executes an external program with given command, arguments, and optional stdin input
## More Actors

These actors build upon the primitive ones to implement the application's business logic. They often depend on one or more primitive actors to perform their tasks.

### Log Actor

The Log actor provides a centralized, thread-safe logging service. It receives log messages from other actors and writes them to a designated log file, providing different log levels for debugging and monitoring.

**Depends on:**

- **Filesystem**: To persist logs to a file.

**Messages:**

- **Log**: Records a message with a specific log level (e.g., Info, Warn, Error, Debug).
- **GetLastLogs**: Retrieves the last N log entries.

### Config Actor

The Config actor manages the application's configuration settings. It is responsible for loading settings from a configuration file, providing them to other actors on request, and persisting changes.

**Depends on:**

- **Filesystem**: To save and load the configuration file.

**Messages:**

- **Get**: Retrieves the value of a specific configuration key.
- **Set**: Updates the value of a specific configuration key.
- **Load**: Reloads the configuration from the source file.
- **Persist**: Saves the current configuration to the source file.

### Lore API Actor

The Lore API actor handles all communication with the external Lore patchwork API. It abstracts the details of HTTP requests and data parsing, providing a clean, typed interface for fetching data like mailing lists, patch series, and individual patches. It acts as a translator between the application's internal data structures and the raw API responses. It does not perform any caching itself.

**Depends on:**

- **Network**: To perform the underlying HTTP requests.

**Messages:**

- **GetAvailableListsPage**: Fetches a paginated list of available mailing lists.
- **GetPatchFeedPage**: Fetches a paginated feed of patches for a specific mailing list.
- **GetRawPatch**: Fetches the raw content of a single patch.

### Cache Actors

To meet the objective of providing cache support, the system uses a set of specialized cache actors. Instead of a single, monolithic cache, the responsibility is divided among actors, each tailored to a specific type of data and capable of directly hitting Lore API to handle cache validation and cache misses. This approach improves modularity and allows for different caching strategies for different data.

#### Mailing List Cache Actor

Caches the list of all available mailing lists from the Lore API. It fetches the complete list, sorts it alphabetically, and persists it to the filesystem to speed up subsequent application startups. It handles cache validation to periodically refresh the list.

**Depends on:**

- **Lore API**: To fetch the mailing list data.
- **Filesystem**: To persist the cache to disk.
- **Config**: To get the path for the cache file.
- **Log**: To log cache operations.

**Messages:**

- **Get**: Retrieves a single mailing list by its index in the sorted list.
- **GetSlice**: Retrieves a range of mailing lists for pagination.
- **Refresh**: Forces a full refresh of the cache from the API.
- **Invalidate**: Clears the in-memory and on-disk cache.
- **Len**: Returns the total number of cached mailing lists.

#### Feed Cache Actor

Provides per-mailing-list caching of patch metadata (like author, subject, and date). It fetches patch feeds on demand, supports pagination, and persists the metadata to the filesystem. It uses a smart validation strategy to only fetch new patches when a feed has been updated on the server.

**Depends on:**

- **Lore API**: To fetch patch metadata.
- **Filesystem**: To persist the cache to disk.
- **Config**: To get the path for the cache directory.
- **Log**: To log cache operations.

**Messages:**

- **Get**: Retrieves a single patch's metadata by its index within a mailing list feed.
- **GetSlice**: Retrieves a range of patch metadata for pagination.
- **Refresh**: Intelligently refreshes the cache for a specific mailing list, fetching only new items.
- **Invalidate**: Clears the cache for a specific mailing list.
- **Len**: Returns the number of cached metadata entries for a list.

#### Patch Cache Actor

Caches the raw content of individual patches (`.mbox` format). Since patch content is immutable, once a patch is fetched and cached, it is considered valid forever. This actor stores each patch in a separate file on disk and uses an in-memory LRU (Least Recently Used) buffer to provide fast access to recently viewed patches.

**Depends on:**

- **Lore API**: To fetch the raw patch content.
- **Filesystem**: To store patch files permanently.
- **Config**: To get the path for the cache directory.
- **Log**: To log cache operations.

**Messages:**

- **Get**: Retrieves the raw content of a patch given its mailing list and message ID.
- **Invalidate**: Removes a specific patch from the disk cache and in-memory buffer.
- **IsAvailable**: Checks if a patch is cached without fetching it.
