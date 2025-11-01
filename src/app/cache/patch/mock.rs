use crate::ArcStr;
use mockall::mock;

mock! {
    #[derive(Debug)]
    pub PatchCache {
        pub async fn get(&self, list: ArcStr, message_id: ArcStr) -> anyhow::Result<String>;
        pub async fn invalidate(&self, list: ArcStr, message_id: ArcStr) -> anyhow::Result<()>;
        pub async fn is_available(&self, list: ArcStr, message_id: ArcStr) -> bool;
    }

    impl Clone for PatchCache {
        fn clone(&self) -> Self;
    }
}
