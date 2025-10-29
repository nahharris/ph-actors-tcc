use mockall::mock;
use crate::api::lore::LoreMailingList;

mock!{
    #[derive(Debug)]
    pub MailingListCache {
        pub async fn get(&self, index: usize) -> anyhow::Result<Option<LoreMailingList>>;
        pub async fn get_slice(&self, range: std::ops::Range<usize>) -> anyhow::Result<Vec<LoreMailingList>>;
        pub async fn refresh(&self) -> anyhow::Result<()>;
        pub async fn invalidate(&self) -> anyhow::Result<()>;
        pub async fn is_available(&self, range: std::ops::Range<usize>) -> bool;
        pub async fn len(&self) -> usize;
        pub async fn persist(&self) -> anyhow::Result<()>;
        pub async fn load(&self) -> anyhow::Result<()>;
    }

    impl Clone for MailingListCache {
        fn clone(&self) -> Self;
    }
}