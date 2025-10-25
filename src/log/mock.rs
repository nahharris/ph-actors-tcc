#[cfg(test)]
use mockall::mock;

#[cfg(test)]
mock! {
    #[derive(Debug)]
    pub Log {
        pub async fn collect_garbage(&self);
        pub async fn flush(self) -> anyhow::Result<()>;
    }
}
