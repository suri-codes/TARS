use std::{any::Any, pin::Pin};

use common::types::Task;
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
    fn id(&self) -> &'static str;

    fn register(&self, config: &Value);

    fn run(&self, config: &Box<dyn Any>)
    -> Pin<Box<dyn Future<Output = Option<RunResult>> + Send>>;
}
