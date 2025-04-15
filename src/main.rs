use std::{
    ffi::{OsStr, OsString},
    path::Path,
    sync::Arc,
    time::Duration,
};

use config::{ConfigCore, PathOpts, USizeOpts};
use log::LogCore;
use sys::SysCore;
use terminal::TerminalCore;
use utils::install_panic_hook;

pub(crate) const BUFFER_SIZE: usize = 128;

mod config;
mod log;
mod sys;
mod terminal;
mod utils;

pub(crate) type ArcStr = std::sync::Arc<str>;
pub(crate) type ArcOsStr = std::sync::Arc<std::ffi::OsStr>;
pub(crate) type ArcFile = std::sync::Arc<tokio::sync::RwLock<tokio::fs::File>>;
pub(crate) type ArcPath = std::sync::Arc<std::path::Path>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    install_panic_hook()?;

    let (sys, _) = SysCore::new().spawn();

    let config_path = sys.env(OsString::from("HOME").into()).await?;
    let config_path = Path::new(config_path.as_ref())
        .join(".config")
        .join("patch-hub")
        .join("config.toml");
    let config_path: ArcPath = Arc::from(config_path);

    let (config, _) = ConfigCore::new(sys.clone(), config_path).spawn();

    let (log, _) = LogCore::build(
        sys.clone(),
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
        sys.env(OsStr::new("HOME").into()).await.unwrap()
    );
    tokio::time::sleep(Duration::from_secs(2)).await;

    term.release().await?;
    Ok(())
}
