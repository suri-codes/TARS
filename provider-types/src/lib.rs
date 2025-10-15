use common::TarsClient;
use std::pin::Pin;
use toml::Value;

pub trait ProviderRuntime: Sync + Send {
    fn id(&self) -> &'static str;
    fn run(&self, client: TarsClient) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

type RunResult = Pin<Box<dyn Future<Output = ()> + Send>>;
pub struct ProviderRegistration {
    pub id: &'static str,
    pub create_and_run: fn(&Value, TarsClient) -> RunResult,
}

inventory::collect!(ProviderRegistration);
