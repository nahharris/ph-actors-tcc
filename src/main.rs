use std::{
    ffi::{OsStr, OsString},
    path::Path,
    sync::Arc,
    time::Duration,
};

use config::{ConfigCore, PathOpts, USizeOpts};
use env::EnvCore;
use fs::FsCore;
use log::LogCore;
use terminal::TerminalCore;
use utils::install_panic_hook;

/// Default buffer size used for various operations in the application.
/// This constant defines the size of buffers used for reading and writing operations.
pub(crate) const BUFFER_SIZE: usize = 128;

mod config;
mod env;
mod fs;
mod log;
mod terminal;
mod utils;

/// A thread-safe reference-counted string type.
/// This type is used throughout the application for sharing string data between threads.
/// 
/// # Examples
/// ```
/// let shared_str: ArcStr = Arc::from("Hello, world!");
/// ```
pub(crate) type ArcStr = std::sync::Arc<str>;

/// A thread-safe reference-counted OS string type.
/// This type is used for handling operating system specific string data across threads.
/// 
/// # Examples
/// ```
/// let shared_os_str: ArcOsStr = Arc::from(OsStr::new("path/to/file"));
/// ```
pub(crate) type ArcOsStr = std::sync::Arc<std::ffi::OsStr>;

/// A thread-safe reference-counted file handle with read-write lock.
/// This type provides synchronized access to file operations across multiple threads.
/// 
/// # Examples
/// ```
/// let file = tokio::fs::File::open("example.txt").await?;
/// let shared_file: ArcFile = Arc::new(RwLock::new(file));
/// ```
pub(crate) type ArcFile = std::sync::Arc<tokio::sync::RwLock<tokio::fs::File>>;

/// A thread-safe reference-counted path type.
/// This type is used for sharing path information across threads safely.
/// 
/// # Examples
/// ```
/// let shared_path: ArcPath = Arc::from(Path::new("path/to/file"));
/// ```
pub(crate) type ArcPath = std::sync::Arc<std::path::Path>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    install_panic_hook()?;

    let (env, _) = EnvCore::new().spawn();
    let (fs, _) = FsCore::new().spawn();

    let config_path = env.env(OsString::from("HOME").into()).await?;
    let config_path = Path::new(config_path.as_ref())
        .join(".config")
        .join("patch-hub")
        .join("config.toml");
    let config_path: ArcPath = Arc::from(config_path);

    let (config, _) = ConfigCore::new(env.clone(), fs.clone(), config_path).spawn();
    let res = config.load().await;

    if res.is_err() {
        config.save().await?;
    }

    let (log, _) = LogCore::build(
        fs.clone(),
        config.log_level().await,
        config.usize(USizeOpts::MaxAge).await,
        config.path(PathOpts::LogDir).await,
    )
    .await?
    .spawn();
    let (term, _) = TerminalCore::build(log.clone())?.spawn();

    log.info("Starting patch-hub");
    term.take_over().await?;
    println!(
        "Hello, world! {}",
        env.env(OsStr::new("HOME").into()).await.unwrap()
    );
    tokio::time::sleep(Duration::from_secs(2)).await;

    term.release().await?;
    Ok(())
}
