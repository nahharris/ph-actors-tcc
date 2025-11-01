use mockall::mock;

use crate::ArcStr;
use crate::app::ui::{NavigationAction, UiState};

mock! {
    #[derive(Debug)]
    pub Ui {
        pub async fn show_lists(&self, page: usize) -> anyhow::Result<()>;
        pub async fn show_feed(&self, list: ArcStr, page: usize) -> anyhow::Result<()>;
        pub async fn show_patch(&self, list: ArcStr, message_id: ArcStr, title: ArcStr) -> anyhow::Result<()>;
        pub async fn update_selection(&self, index: usize);
        pub async fn previous_page(&self) -> anyhow::Result<()>;
        pub async fn next_page(&self) -> anyhow::Result<()>;
        pub async fn navigate_back(&self) -> anyhow::Result<()>;
        pub async fn submit_selection(&self) -> anyhow::Result<Option<NavigationAction>>;
        pub async fn get_state(&self) -> UiState;
    }

    impl Clone for Ui {
        fn clone(&self) -> Self;
    }
}
