use mockall::mock;
use std::sync::Arc;
use tokio::task::JoinHandle;

use crate::{
    error::TerminalError,
    terminal::data::{Screen, UiEvent},
};

mock! {
    #[derive(Debug)]
    pub Terminal {
        pub fn handle(&self) -> Arc<JoinHandle<()>>;
        pub async fn show(&self, screen: Screen) -> Result<(), TerminalError>;
        pub async fn get_ui_event(&self) -> Result<Option<UiEvent>, TerminalError>;
        pub async fn clear_ui_events(&self) -> Result<(), TerminalError>;
        pub async fn quit(&self) -> Result<(), TerminalError>;
    }

    impl Clone for Terminal {
        fn clone(&self) -> Self;
    }
}
