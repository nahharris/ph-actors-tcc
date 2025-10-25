#[cfg(test)]
use mockall::mock;

use crate::shell::data::Result;
use crate::{ArcSlice, ArcStr};

#[cfg(test)]
mock! {
    #[derive(Debug)]
    pub Shell {
        pub async fn execute(&self, program: ArcStr, args: ArcSlice<ArcStr>, stdin: Option<ArcStr>) -> anyhow::Result<Result>;
    }

    impl Clone for Shell {
        fn clone(&self) -> Self;
    }
}
