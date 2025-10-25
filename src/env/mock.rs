#[cfg(test)]
use mockall::mock;

#[cfg(test)]
use std::env::VarError;

#[cfg(test)]
use crate::{ArcOsStr, ArcStr};

#[cfg(test)]
mock! {
    #[derive(Debug)]
    pub Env {
        pub async fn set_env(&self, key: ArcOsStr, value: String);
        pub async fn unset_env(&self, key: ArcOsStr);
        pub async fn env(&self, key: ArcOsStr) -> Result<ArcStr, VarError>;
    }

    impl Clone for Env {
        fn clone(&self) -> Self;
    }
}
