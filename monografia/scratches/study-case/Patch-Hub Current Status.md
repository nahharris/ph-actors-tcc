## Project Overview

This work examines the rewrite of Patch-Hub, an open-source tool developed under the KWorkflow (kw) organization, primarily by USP students. Patch-Hub is a terminal-based interface for browsing and interacting with patches on lore.kernel.org, supporting the Linux kernel and related projects. It streamlines the traditional, mailing-list–centric workflow used in kernel development.

Patch-Hub belongs to the broader KWorkflow project, a Developer Automation Workflow System that reduces kernel setup overhead and provides tools for day-to-day tasks. Rewriting Patch-Hub using the Actor Model aims both to improve the tool and to demonstrate the benefits of actor-based architectures for centralized Rust applications.

This project is a suitable case study because it:

- Has a clear scope and feature set
- Presents architectural challenges well-suited to the Actor Model
- Is actively maintained and used by the kernel community
- Enables concrete measurement of architectural impact

## Current Architecture

The current Patch-Hub implementation follows a mostly single-threaded, monolithic architecture with the following key modules:

- **`lore`**: Interface to interact with the lore API for fetching patch data
- **`ui`**: Terminal rendering and display logic
- **`cli`**: Command-line argument parsing
- **`app`**: Central application state and data management
  - `logger`: Handles logging functionality
  - `popup`: Creates and manages popup dialogs
  - `config`: Manages application configurations
- **`handler`**: User input handling and event processing

## Key Features

The current implementation provides several essential features for kernel developers:

- **Mailing List Selection**: Dynamically fetch and browse mailing lists archived on lore.kernel.org
- **Latest Patchsets**: View the most recent patchsets from selected mailing lists
- **Patchset Details & Actions**: View individual patch contents and metadata (title, author, version, number of patches, last update, code-review trailers)
- **Patch Application**: Apply patches to local kernel trees
- **Bookmarking System**: Bookmark important patches for easy reference
- **Review Integration**: Reply with `Reviewed-by` tags to patch series
- **Enhanced Rendering**: Support for external tools like `bat`, `delta`, and `diff-so-fancy` for better patch visualization

## Architectural Limitations

Despite its functionality, the current Patch-Hub implementation faces significant architectural challenges that limit its flexibility and maintainability:

### Rust Ownership System Constraints

The most critical limitation comes from Rust's strict ownership system. The application uses a central `app` variable that holds configurations, popups, and application state. This creates a fundamental architectural problem, for example:

- Every value in Rust must have exactly one owner
- To modify the app, a mutable reference (`&mut`) is required
- Popups need to modify data in the app, requiring a second `&mut` reference to app
- Rust prohibits having two simultaneous `&mut` references to the same data

This restriction prevents popups from modifying application data, limiting them to display-only functionality. The current workaround involves extensive refactoring whenever new features need to be added.

### Single-Threaded Limitations

The application is primarily single-threaded, which creates bottlenecks when performing I/O operations such as:

- Network requests to the lore API
- File system operations
- Terminal rendering updates
- Complicated logic to show loading screens

This design prevents the application from efficiently handling concurrent operations and can lead to unresponsive user interfaces during network or file operations.

### Tight Coupling

The current architecture exhibits tight coupling between different components:

- UI logic is intertwined with business logic
- Configuration management is deeply embedded in the application state
- Error handling is scattered throughout the codebase
- Testing becomes difficult due to the monolithic structure and side-effectful code used everywhere
