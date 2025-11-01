#[cfg(test)]
use mockall::mock;

#[cfg(test)]
mock! {
    #[derive(Debug)]
    pub Log {
        pub async fn collect_garbage(&self);
        pub async fn flush(self) -> anyhow::Result<()>;
        pub fn info(&self, scope: &'static str, message: String);
        pub fn warn(&self, scope: &'static str, message: String);
        pub fn error(&self, scope: &'static str, message: String);
    }

    impl Clone for Log {
        fn clone(&self) -> Self;
    }
}
