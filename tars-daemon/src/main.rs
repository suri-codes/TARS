use color_eyre::Result;
use common::DAEMON_ADDR;
use common::TarsClient;
use common::logging;
use provider_types::ProviderRegistration;
use provider_types::initialize_providers;
use provider_types::parse_all_configs;
use tars_daemon::{DaemonState, Db, TarsDaemon};
use toml::Value;

#[tokio::main]
async fn main() -> Result<()> {
    logging::init("tars-d.log", true)?;

    let db = Db::new(false).await?;

    let state = DaemonState::new(db, DAEMON_ADDR);

    let raw = r#"
providers = ["recurring"]

[recurring]
# whatever keys your provider expects
interval = "30d"
        "#;

    let raw_config: Value = raw.parse().expect("invalid TOML");

    initialize_providers(&raw_config);

    let configs = parse_all_configs(&raw_config);

    println!("loaded configs for providers : {:?}", configs.keys());

    // Spawn Axum daemon in background — does NOT block
    let daemon_handle = tokio::spawn(async move {
        let daemon = TarsDaemon::init(state).await;
        daemon.run().await; // This will run Axum forever unless shutdown signal
    });

    let providers_handle = tokio::spawn(async move {
        let client = TarsClient::default().await.expect("should be just fine");
        for reg in inventory::iter::<ProviderRegistration> {
            if let Some(cfg) = configs.get(reg.id) {
                // Assuming run() is async and sequential execution is fine
                reg.runtime.run(cfg, &client).await;
            }
        }
    });

    Ok(())
}
