use std::ffi::OsString;

use sys::SysCore;

pub(crate) const BUFFER_SIZE: usize = 128;

mod sys;

pub(crate) type ArcString = std::sync::Arc<String>;
pub(crate) type ArcOsString = std::sync::Arc<std::ffi::OsString>;
pub(crate) type ArcFile = std::sync::Arc<tokio::fs::File>;
pub(crate) type ArcPathBuf = std::sync::Arc<std::path::PathBuf>;

#[tokio::main]
async fn main() {
    let (sys, _) = SysCore::new().spawn();
    println!(
        "Hello, world! {}",
        sys.get_env(OsString::from("HOME").into()).await.unwrap()
    );
}
