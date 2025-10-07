# Agent Guidelines for Patch Hub

## Build/Lint/Test Commands
- **Build**: `cargo build` or `cargo build --release`
- **Test all**: `cargo test`
- **Test single**: `cargo test <test_name>` (e.g., `cargo test test_mock_env_creation`)
- **Format**: `cargo fmt` (MUST run before finishing work)
- **Lint**: `cargo clippy` (run to check for common mistakes)
- **Run**: `cargo run` or `cargo run --release`
- **Examples**: `cargo run --example <name>` (e.g., `cargo run --example lore`)
- **Add dependency**: `cargo add <dep>` (always check `Cargo.toml` first)

## Code Style Guidelines
- **Imports**: Always check `Cargo.toml` before using libraries; run `cargo add <dep>` if needed
- **Module structure**: Use `something.rs` + `something/` directory, NEVER `something/mod.rs`
- **Documentation**: Add `///` doc comments to all public functions, structs, and modules; avoid inline comments except for unusual code
- **Comments**: NO inline comments unless code is truly unusual; let doc comments and code speak for themselves
- **Error handling**: Use `anyhow::Result` for error propagation; `.context().expect()` pattern for actor communication (assume actors never die)
- **Async**: All actor public methods MUST be `async`; use Tokio runtime exclusively (`tokio::sync::mpsc`, `tokio::sync::oneshot`, `tokio::task`)
- **Concurrency**: NEVER use `std::thread`, always use `tokio::task` unless impossible otherwise
- **Thread-safe types**: Always use `ArcStr`, `ArcOsStr`, `ArcPath`, `ArcSlice<T>`, `ArcVec<T>` from `utils.rs` for shared data
- **Formatting**: MUST run `cargo fmt` before completing any work

## Actor Pattern (see `.cursor/rules/actor-model.mdc` for complete specification)
- **Structure**: 3 files: `core.rs` (impl), `message.rs` (message types), `data.rs` (data structures), plus module file and `README.md`
- **Public interface**: Enum with `Actual(Sender<Message>)` and `Mock(Arc<Mutex<MockData>>)` variants
- **Core creation**: `new()` if always succeeds, `build()` if returns `Result`
- **Methods**: `spawn()` for real implementation, `mock()` for testing; all public methods must be `async`
- **Communication**: `mpsc` channels for messages, `oneshot` channels for responses
- **State management**: NEVER share mutable state; use message passing for ALL communication
- **Error handling**: Use `.context().expect()` when sending/receiving messages (assume actors never die)

## Conventional Commits
- **Format**: `type(scope): subject` in imperative mood, lowercase, no period, under 50 chars
- **Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`
- **Examples**: `feat(lore): add message parsing`, `fix(cache): resolve race condition in feed updates`
