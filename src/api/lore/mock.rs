use crate::utils::ArcSlice;
use crate::{
    ArcStr,
    api::lore::{LoreMailingList, LorePage, LorePatchMetadata},
    error::LoreApiError,
};

use mockall::mock;

mock! {
    #[derive(Debug)]
    pub LoreApi {
        pub async fn get_patch_feed_page(&self, target_list: ArcStr, min_index: usize) -> Result<Option<LorePage<LorePatchMetadata>>, LoreApiError>;
        pub async fn get_available_lists_page(&self, min_index: usize) -> Result<Option<LorePage<LoreMailingList>>, LoreApiError>;
        pub async fn get_available_lists(&self) -> Result<ArcSlice<LoreMailingList>, LoreApiError>;
        pub async fn get_patch_html(&self, target_list: ArcStr, message_id: ArcStr) -> Result<ArcStr, LoreApiError>;
        pub async fn get_raw_patch(&self, target_list: ArcStr, message_id: ArcStr) -> Result<ArcStr, LoreApiError>;
        pub async fn get_patch_metadata(&self, target_list: ArcStr, message_id: ArcStr) -> Result<ArcStr, LoreApiError>;
    }

    impl Clone for LoreApi {
        fn clone(&self) -> Self;
    }
}
