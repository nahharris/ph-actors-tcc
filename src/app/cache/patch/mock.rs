use crate::{error::CacheError, ArcStr};
use mockall::mock;

mock! {
    #[derive(Debug)]
    pub PatchCache {
        pub async fn get(&self, list: ArcStr, message_id: ArcStr) -> Result<String, CacheError>;
        pub async fn invalidate(&self, list: ArcStr, message_id: ArcStr) -> Result<(), CacheError>;
        pub async fn is_available(&self, list: ArcStr, message_id: ArcStr) -> Result<bool, CacheError>;
    }

    impl Clone for PatchCache {
        fn clone(&self) -> Self;
    }
}
