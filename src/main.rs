use std::{path::Path, time::Duration};

use ph::app::config::{Config, PathOpt, USizeOpt};
use ph::env::Env;
use ph::fs::Fs;
use ph::log::Log;
use ph::terminal::Terminal;
use ph::utils::install_panic_hook;

use ph::{ArcOsStr, ArcPath};

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

    let log = Log::spawn(
        fs.clone(),
        config.log_level().await,
        config.usize(USizeOpt::MaxAge).await,
        config.path(PathOpt::LogDir).await,
    )
    .await?;
    let term = Terminal::spawn(log.clone())?;

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
