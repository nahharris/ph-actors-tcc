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
