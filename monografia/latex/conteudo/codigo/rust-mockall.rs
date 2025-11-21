mock!{
    #[derive(Debug)]
    pub App {
        pub async fn operation(&self, params: Params) -> Result<Value>;
    }

    impl Clone for Actor {
        fn clone(&self) -> Self;
    }
}

