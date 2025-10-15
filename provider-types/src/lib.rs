use std::{any::Any, pin::Pin};

mod provider;

pub use provider::*;

use common::TarsClient;
use toml::Value;

// implement this after we finish poc
pub enum RunInterval {
    Minutely,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

pub trait ProviderRuntime: Sync + Send {
    type Config: Send + Sync + 'static;

    fn id(&self) -> &'static str;

    fn parse_config(&self, config: &Value) -> Self::Config;

    fn run(
        &self,
        config: &Self::Config,
        client: &TarsClient,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;

}
