use color_eyre::Result;
use common::DAEMON_ADDR;
use common::logging;
use provider_types::ProviderRegistration;
use tars_daemon::{DaemonState, Db, TarsDaemon};
use toml::Table;
use toml::Value;
use tracing::info;

// need to do this to link to providers
use recurring as _;

#[tokio::main]
async fn main() -> Result<()> {
    logging::init("tars-d.log", true)?;

    let db = Db::new(false).await?;

    let state = DaemonState::new(db, DAEMON_ADDR);

    let raw = r#"
providers = ["recurring"]

[recurring]
interval = "30d"
"#;

    let raw_config: Table = raw.trim().parse().expect("invalid TOML");

    // Spawn Axum daemon in background — does NOT block
    let daemon_handle = tokio::spawn(async move {
        let daemon = TarsDaemon::init(state).await;
        daemon.run().await; // This will run Axum forever unless shutdown signal
    });

    let enabled = raw_config
        .get("providers")
        .and_then(|v| v.as_array())
        .expect("providers array missing");

    info!("enabled providers: {:#?}", enabled);

    let enabled_ids: Vec<_> = enabled.iter().filter_map(|v| v.as_str()).collect();
    info!("enabled provider_ids: {:#?}", enabled_ids);

    for reg in inventory::iter::<ProviderRegistration> {
        info!("found reg: {}", reg.id);
        if enabled_ids.contains(&reg.id) {
            let cfg = raw_config.get(reg.id).expect("config missing");
            tokio::spawn((reg.create_and_run)(cfg));
        }
    }

    // let providers_handle = tokio::spawn(async move {
    //     let client = TarsClient::default().await.expect("should be just fine");
    //     for reg in inventory::iter::<ProviderRegistration> {
    //         if let Some(cfg) = configs.get(reg.id) {
    //             // Assuming run() is async and sequential execution is fine
    //             reg.runtime.run(cfg, &client).await;
    //         }
    //     }
    // });

    Ok(())
}
