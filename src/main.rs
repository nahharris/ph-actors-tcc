use std::{path::Path, time::Duration};

use config::{Config, PathOpt, USizeOpt};
use env::Env;
use fs::Fs;
use log::LogCore;
use terminal::TerminalCore;
use utils::install_panic_hook;

pub(crate) use utils::{ArcFile, ArcOsStr, ArcPath, ArcStr};

/// Default buffer size used for various operations in the application.
/// This constant defines the size of buffers used for reading and writing operations.
pub(crate) const BUFFER_SIZE: usize = 128;

mod config;
mod env;
mod fs;
mod log;
mod net;
mod terminal;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    install_panic_hook()?;

    let env = Env::spawn();
    let fs = Fs::spawn();

    let config_path = env.env(ArcOsStr::from("HOME")).await?;
    let config_path = Path::new(&config_path)
        .join(".config")
        .join("patch-hub")
        .join("config.toml");
    let config_path = ArcPath::from(&config_path);

    let config = Config::spawn(env.clone(), fs.clone(), config_path);
    let res = config.load().await;

    if res.is_err() {
        config.save().await?;
    }

    let (log, _) = LogCore::build(
        fs.clone(),
        config.log_level().await,
        config.usize(USizeOpt::MaxAge).await,
        config.path(PathOpt::LogDir).await,
    )
    .await?
    .spawn();
    let (term, _) = TerminalCore::build(log.clone())?.spawn();

    log.info("Starting patch-hub");
    term.take_over().await?;
    println!(
        "Hello, world! {}",
        env.env(ArcOsStr::from("HOME")).await.unwrap()
    );
    tokio::time::sleep(Duration::from_secs(2)).await;

    term.release().await?;
    Ok(())
}
