# Patch Hub

A Text User Interface (TUI) application designed to help Linux kernel developers manage and review patches from the [Lore Kernel Archive](https://lore.kernel.org). Built with Rust using the Actor Model architecture for improved flexibility and maintainability.

**This is a capstone project that reimplements and extends the original [patch-hub](https://github.com/kworkflow/patch-hub) by kworkflow, applying the Actor Model architecture to overcome Rust's ownership constraints in complex applications.**

## Overview

Patch Hub is a terminal-based application that provides a user-friendly interface for accessing and managing patches from various Linux kernel mailing lists. It's designed to overcome the limitations of traditional single-threaded applications by implementing a flexible Actor Model architecture.

### Key Features

- **Browse Mailing Lists**: Access and navigate through different kernel mailing lists
- **View Patch Details**: Examine patch content, metadata, and discussions
- **Patch Management**: Apply patches to local repositories and mark them as reviewed
- **Configurable Interface**: Customizable settings and keybindings
- **Comprehensive Logging**: Built-in logging system for debugging and monitoring
- **Actor-Based Architecture**: Thread-safe, modular design using Rust's async/await

## Architecture

Patch Hub uses the Actor Model pattern to provide thread-safe interfaces to various system operations. Each actor is responsible for a specific domain:

- **Lore API Actor**: Interfaces with the Lore Kernel Archive
- **Terminal Actor**: Handles low-level terminal interactions
- **Filesystem Actor**: Manages file operations
- **Environment Actor**: Handles environment variables
- **Configuration Actor**: Manages application settings
- **Logging Actor**: Handles logging operations
- **Network Actor**: Manages HTTP requests

### Actor Communication

Actors communicate through message passing using Tokio channels, ensuring thread safety and preventing data races. Each actor can be easily mocked for testing purposes.

## Installation

### Prerequisites

- **Rust**: Version 1.88 or later (any recent version should work)
- **Cargo**: Rust's package manager (included with Rust)

### Building from Source

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd patch-hub
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Run the application:
   ```bash
   cargo run --release
   ```

## Configuration

Patch Hub creates its configuration file at `~/.config/patch-hub/config.toml`. A sample configuration is provided in `samples/config.toml`:

```toml
log_dir = "/tmp/logs"
log_level = "Info"
max_age = 30
```

### Configuration Options

- `log_dir`: Directory where log files are stored
- `log_level`: Logging level (Debug, Info, Warn, Error)
- `max_age`: Maximum age of log files in days

## Examples

### Browse Available Mailing Lists

View all available mailing lists from the Lore Kernel Archive:

```bash
cargo run --example lore
```

This example demonstrates:
- Fetching available mailing lists
- Displaying patch feeds from specific lists
- Basic usage of the Lore API actor

## Development

### Project Structure

```
src/
├── api/           # API actors (Lore, etc.)
├── config/        # Configuration management
├── env/           # Environment variable handling
├── fs/            # Filesystem operations
├── log/           # Logging system
├── net/           # Network operations
├── terminal/      # Terminal interface
└── utils/         # Utility functions

examples/          # Example programs
samples/           # Sample configuration files
monografia/        # Documentation and research
```

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

### Actor Pattern Implementation

The project implements a comprehensive Actor Model pattern. See `ACTOR.md` for detailed documentation on:

- Actor architecture and design patterns
- Message passing and communication
- Thread safety considerations
- Mock implementations for testing
- Best practices and usage examples

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## License

This project is licensed under the GNU General Public License v2.0 - see the [LICENSE](LICENSE) file for details.

The GNU GPL is a free software license that guarantees your freedom to share and change free software. For more information about the GNU GPL, visit the [Free Software Foundation](https://www.gnu.org/licenses/gpl-2.0.html).

## Acknowledgments

- Built for the Linux kernel development community
- Inspired by the need for better patch management tools
- Uses the Actor Model to overcome Rust's ownership constraints in complex applications
- **Derived from the original [patch-hub](https://github.com/kworkflow/patch-hub) project by kworkflow**
- **This is a capstone project demonstrating advanced software architecture patterns**