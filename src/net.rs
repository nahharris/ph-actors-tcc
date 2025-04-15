use reqwest::Client;
use tokio::sync::mpsc::Sender;

use crate::{config::Config, log::Log};

pub struct NetCore {
    config: Config,
    log: Log,
    client: Client,
}

impl NetCore {
    pub fn new(config: Config, log: Log) -> Self {
        let client = Client::new();

        Self {
            config,
            log,
            client,
        }
    }

    pub fn spawn(mut self) -> anyhow::Result<(Self, tokio::task::JoinHandle<anyhow::Result<()>>)> {
        let handle = tokio::spawn(async move {
            self.log.info("NetCore spawned");
            Ok(())
        });
        Ok((self, handle))
    }
}

#[derive(Debug)]
pub enum Message {
    Get(String),
}

pub enum Net {
    Actual(Sender<Message>),
}
