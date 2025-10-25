use mockall::mock;

use crate::app::config::{PathOpt, Renderer, RendererOpt, USizeOpt, data::Data};
use crate::{ArcPath, log::LogLevel};

mock! {
    #[derive(Debug)]
    pub Config {
        pub async fn load(&self) -> anyhow::Result<()>;
        pub async fn save(&self) -> anyhow::Result<()>;
        pub async fn path(&self, opt: PathOpt) -> ArcPath;
        pub async fn set_path(&self, opt: PathOpt, path: ArcPath);
        pub async fn log_level(&self) -> LogLevel;
        pub async fn set_log_level(&self, level: LogLevel);
        pub async fn usize(&self, opt: USizeOpt) -> usize;
        pub async fn set_usize(&self, opt: USizeOpt, value: usize);
        pub async fn renderer(&self, opt: RendererOpt) -> Renderer;
        pub async fn set_renderer(&self, opt: RendererOpt, renderer: Renderer);
        pub async fn get_data(&self) -> Data;
    }

    impl Clone for Config {
        fn clone(&self) -> Self;
    }
}
