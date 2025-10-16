use color_eyre::eyre::Result;
use common::TarsClient;
use std::pin::Pin;
use toml::Value;

pub type RunResult = Pin<Box<dyn Future<Output = Result<()>> + Send>>;

pub trait ProviderRuntime: Sync + Send {
    fn id(&self) -> &'static str;
    fn run(self, client: TarsClient) -> RunResult;
}

pub struct ProviderRegistration {
    pub id: &'static str,
    pub create_and_run: fn(Value, TarsClient) -> RunResult,
}

inventory::collect!(ProviderRegistration);
