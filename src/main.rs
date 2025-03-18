use std::{ffi::OsString, time::Duration};

use sys::SysCore;
use terminal::TerminalCore;
use utils::install_panic_hook;

pub(crate) const BUFFER_SIZE: usize = 128;

mod sys;
mod terminal;
mod utils;

pub(crate) type ArcString = std::sync::Arc<String>;
pub(crate) type ArcOsString = std::sync::Arc<std::ffi::OsString>;
pub(crate) type ArcFile = std::sync::Arc<tokio::fs::File>;
pub(crate) type ArcPathBuf = std::sync::Arc<std::path::PathBuf>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    install_panic_hook()?;

    let (sys, _) = SysCore::new().spawn();
    let (term, _) = TerminalCore::build()?.spawn();

    term.take_over().await?;
    println!(
        "Hello, world! {}",
        sys.get_env(OsString::from("HOME").into()).await.unwrap()
    );
    tokio::time::sleep(Duration::from_secs(2)).await;

    term.release().await?;
    Ok(())
}
