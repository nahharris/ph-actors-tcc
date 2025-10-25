#[cfg(test)]
use mockall::mock;

#[cfg(test)]
use crate::log::LogMessage;
#[cfg(test)]
use tokio::task::JoinHandle;

#[cfg(test)]
mock! {
    #[derive(Debug)]
    pub Log {
        pub async fn collect_garbage(&self);
        pub fn flush(self) -> JoinHandle<()>;
        pub async fn get_messages(&self) -> Option<Vec<LogMessage>>;
    }
}
