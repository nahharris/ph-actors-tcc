use mockall::mock;

use crate::ArcStr;
use crate::api::lore::LorePatchMetadata;

mock! {
    #[derive(Debug)]
    pub FeedCache {
        pub async fn get(&self, list: ArcStr, index: usize) -> anyhow::Result<Option<LorePatchMetadata>>;
        pub async fn get_slice(&self, list: ArcStr, range: std::ops::Range<usize>) -> anyhow::Result<Vec<LorePatchMetadata>>;
        pub async fn refresh(&self, list: ArcStr) -> anyhow::Result<()>;
        pub async fn invalidate(&self, list: ArcStr) -> anyhow::Result<()>;
        pub async fn is_available(&self, list: ArcStr, range: std::ops::Range<usize>) -> bool;
        pub async fn len(&self, list: ArcStr) -> usize;
        pub async fn persist(&self, list: ArcStr) -> anyhow::Result<()>;
        pub async fn load(&self, list: ArcStr) -> anyhow::Result<()>;
        pub async fn is_loaded(&self, list: ArcStr) -> bool;
        pub async fn ensure_loaded(&self, list: ArcStr) -> anyhow::Result<()>;
    }

    impl Clone for FeedCache {
        fn clone(&self) -> Self;
    }
}
