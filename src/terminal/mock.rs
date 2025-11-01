use mockall::mock;

use crate::terminal::data::Screen;


mock! {
    #[derive(Debug)]
    pub Terminal {
        pub async fn show(&self, screen: Screen);
        pub async fn quit(&self);
    }

    impl Clone for Terminal {
        fn clone(&self) -> Self;
    }
}
