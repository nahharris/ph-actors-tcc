#[cfg(test)]
use mockall::mock;

#[cfg(test)]
mock! {
    #[derive(Debug)]
    pub Shell {
        pub async fn execute(&self, program: crate::ArcStr, args: crate::ArcSlice<crate::ArcStr>, stdin: Option<crate::ArcStr>) -> anyhow::Result<crate::shell::data::Result>;
    }

    impl Clone for Shell {
        fn clone(&self) -> Self;
    }
}
