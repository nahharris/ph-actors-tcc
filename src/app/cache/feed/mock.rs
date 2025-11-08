use mockall::mock;

use crate::api::lore::LorePatchMetadata;
use crate::{ArcStr, error::CacheError};

mock! {
    #[derive(Debug)]
    pub FeedCache {
        pub async fn get(&self, list: ArcStr, index: usize) -> Result<Option<LorePatchMetadata>, CacheError>;
        pub async fn get_slice(&self, list: ArcStr, range: std::ops::Range<usize>) -> Result<Vec<LorePatchMetadata>, CacheError>;
        pub async fn refresh(&self, list: ArcStr) -> Result<(), CacheError>;
        pub async fn invalidate(&self, list: ArcStr) -> Result<(), CacheError>;
        pub async fn is_available(&self, list: ArcStr, range: std::ops::Range<usize>) -> Result<bool, CacheError>;
        pub async fn len(&self, list: ArcStr) -> Result<usize, CacheError>;
        pub async fn persist(&self, list: ArcStr) -> Result<(), CacheError>;
        pub async fn load(&self, list: ArcStr) -> Result<(), CacheError>;
        pub async fn is_loaded(&self, list: ArcStr) -> Result<bool, CacheError>;
        pub async fn ensure_loaded(&self, list: ArcStr) -> Result<(), CacheError>;
    }

    impl Clone for FeedCache {
        fn clone(&self) -> Self;
    }
}
