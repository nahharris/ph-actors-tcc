use mockall::mock;
use std::sync::Arc;
use tokio::task::JoinHandle;

use crate::terminal::data::{Screen, UiEvent};

mock! {
    #[derive(Debug)]
    pub Terminal {
        pub fn handle(&self) -> Arc<JoinHandle<()>>;
        pub async fn show(&self, screen: Screen);
        pub async fn get_ui_event(&self) -> Option<UiEvent>;
        pub async fn clear_ui_events(&self);
        pub async fn quit(&self);
    }

    impl Clone for Terminal {
        fn clone(&self) -> Self;
    }
}
