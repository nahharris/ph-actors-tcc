use crate::utils::ArcSlice;
use crate::{
    ArcStr,
    api::lore::{LoreMailingList, LorePage, LorePatchMetadata},
};

use mockall::mock;

mock! {
    #[derive(Debug)]
    pub LoreApi {
        pub async fn get_patch_feed_page(&self, target_list: ArcStr, min_index: usize) -> anyhow::Result<Option<LorePage<LorePatchMetadata>>>;
        pub async fn get_available_lists_page(&self, min_index: usize) -> anyhow::Result<Option<LorePage<LoreMailingList>>>;
        pub async fn get_available_lists(&self) -> anyhow::Result<ArcSlice<LoreMailingList>>;
        pub async fn get_patch_html(&self, target_list: ArcStr, message_id: ArcStr) -> anyhow::Result<ArcStr>;
        pub async fn get_raw_patch(&self, target_list: ArcStr, message_id: ArcStr) -> anyhow::Result<ArcStr>;
        pub async fn get_patch_metadata(&self, target_list: ArcStr, message_id: ArcStr) -> anyhow::Result<ArcStr>;
    }

    impl Clone for LoreApi {
        fn clone(&self) -> Self;
    }
}
