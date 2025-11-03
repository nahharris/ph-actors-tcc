use mockall::mock;

use crate::{error::UiError, ArcStr};
use crate::app::ui::{NavigationAction, UiState};

mock! {
    #[derive(Debug)]
    pub Ui {
        pub async fn show_lists(&self, page: usize) -> Result<(), UiError>;
        pub async fn show_feed(&self, list: ArcStr, page: usize) -> Result<(), UiError>;
        pub async fn show_patch(&self, list: ArcStr, message_id: ArcStr, title: ArcStr) -> Result<(), UiError>;
        pub async fn update_selection(&self, index: usize);
        pub async fn previous_page(&self) -> Result<(), UiError>;
        pub async fn next_page(&self) -> Result<(), UiError>;
        pub async fn navigate_back(&self) -> Result<(), UiError>;
        pub async fn submit_selection(&self) -> Result<Option<NavigationAction>, UiError>;
        pub async fn get_state(&self) -> UiState;
    }

    impl Clone for Ui {
        fn clone(&self) -> Self;
    }
}
