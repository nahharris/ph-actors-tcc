use mockall::mock;

use crate::{ArcStr, error::RenderError};

mock! {
    #[derive(Debug)]
    pub Render {
        pub async fn render_patch(&self, content: ArcStr) -> Result<ArcStr, RenderError>;
    }

    impl Clone for Render {
        fn clone(&self) -> Self;
    }
}
