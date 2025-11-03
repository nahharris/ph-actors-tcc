use crate::{error::CacheError, api::lore::LoreMailingList};
use mockall::mock;

mock! {
    #[derive(Debug)]
    pub MailingListCache {
        pub async fn get(&self, index: usize) -> Result<Option<LoreMailingList>, CacheError>;
        pub async fn get_slice(&self, range: std::ops::Range<usize>) -> Result<Vec<LoreMailingList>, CacheError>;
        pub async fn refresh(&self) -> Result<(), CacheError>;
        pub async fn invalidate(&self) -> Result<(), CacheError>;
        pub async fn is_available(&self, range: std::ops::Range<usize>) -> Result<bool, CacheError>;
        pub async fn len(&self) -> Result<usize, CacheError>;
        pub async fn persist(&self) -> Result<(), CacheError>;
        pub async fn load(&self) -> Result<(), CacheError>;
    }

    impl Clone for MailingListCache {
        fn clone(&self) -> Self;
    }
}
