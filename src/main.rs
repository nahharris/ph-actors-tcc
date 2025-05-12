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

pub(crate) const BUFFER_SIZE: usize = 128;

mod config;
mod env;
mod fs;
mod log;
mod terminal;
mod utils;

pub(crate) type ArcStr = std::sync::Arc<str>;
pub(crate) type ArcOsStr = std::sync::Arc<std::ffi::OsStr>;
pub(crate) type ArcFile = std::sync::Arc<tokio::sync::RwLock<tokio::fs::File>>;
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
