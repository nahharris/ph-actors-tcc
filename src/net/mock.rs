#[cfg(test)]
use mockall::mock;

#[cfg(test)]
use std::collections::HashMap;

#[cfg(test)]
use crate::ArcStr;

#[cfg(test)]
mock! {
    #[derive(Debug)]
    pub Net {
        pub async fn get(&self, url: ArcStr, headers: Option<HashMap<ArcStr, ArcStr>>) -> Result<ArcStr, anyhow::Error>;
        pub async fn post(&self, url: ArcStr, headers: Option<HashMap<ArcStr, ArcStr>>, body: Option<ArcStr>) -> Result<ArcStr, anyhow::Error>;
        pub async fn put(&self, url: ArcStr, headers: Option<HashMap<ArcStr, ArcStr>>, body: Option<ArcStr>) -> Result<ArcStr, anyhow::Error>;
        pub async fn delete(&self, url: ArcStr, headers: Option<HashMap<ArcStr, ArcStr>>) -> Result<ArcStr, anyhow::Error>;
        pub async fn patch(&self, url: ArcStr, headers: Option<HashMap<ArcStr, ArcStr>>, body: Option<ArcStr>) -> Result<ArcStr, anyhow::Error>;
    }

    impl Clone for Net {
        fn clone(&self) -> Self;
    }
}
