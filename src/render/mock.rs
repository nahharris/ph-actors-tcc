use mockall::mock;

use crate::{error::RenderError, ArcStr};

mock! {
    #[derive(Debug)]
    pub Render {
        pub async fn render_patch(&self, content: ArcStr) -> Result<ArcStr, RenderError>;
    }

    impl Clone for Render {
        fn clone(&self) -> Self;
    }
}