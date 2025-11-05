use mockall::mock;

use crate::app::config::{ConfigError, PathOpt, Renderer, RendererOpt, USizeOpt, data::Data};
use crate::{ArcPath, log::LogLevel};

mock! {
    #[derive(Debug)]
    pub Config {
        pub async fn load(&self) -> Result<(), ConfigError>;
        pub async fn save(&self) -> Result<(), ConfigError>;
        pub async fn path(&self, opt: PathOpt) -> Result<ArcPath, ConfigError>;
        pub async fn set_path(&self, opt: PathOpt, path: ArcPath) -> Result<(), ConfigError>;
        pub async fn log_level(&self) -> Result<LogLevel, ConfigError>;
        pub async fn set_log_level(&self, level: LogLevel) -> Result<(), ConfigError>;
        pub async fn usize(&self, opt: USizeOpt) -> Result<usize, ConfigError>;
        pub async fn set_usize(&self, opt: USizeOpt, value: usize) -> Result<(), ConfigError>;
        pub async fn renderer(&self, opt: RendererOpt) -> Result<Renderer, ConfigError>;
        pub async fn set_renderer(&self, opt: RendererOpt, renderer: Renderer) -> Result<(), ConfigError>;
        pub async fn get_data(&self) -> Result<Data, ConfigError>;
    }

    impl Clone for Config {
        fn clone(&self) -> Self;
    }
}
