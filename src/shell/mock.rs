use mockall::mock;

use crate::shell::data;
use crate::{ArcSlice, ArcStr, error::ShellError};

mock! {
    #[derive(Debug)]
    pub Shell {
        pub async fn execute(&self, program: ArcStr, args: ArcSlice<ArcStr>, stdin: Option<ArcStr>) -> Result<data::Result, ShellError>;
    }

    impl Clone for Shell {
        fn clone(&self) -> Self;
    }
}
