use color_eyre::Result;
use common::DAEMON_ADDR;
use common::TarsConfig;
use common::logging;
use provider_types::ProviderRegistration;
use tars_daemon::{DaemonState, Db, TarsDaemon};
use toml::Table;
use tracing::info;

// need to do this to link to providers
use recurring as _;

#[tokio::main]
async fn main() -> Result<()> {
    logging::init("tars-d.log", true)?;

    let db = Db::new(false).await?;

    let state = DaemonState::new(db, DAEMON_ADDR);

    let daemon = TarsDaemon::init(state).await;
    daemon.run().await // This will run Axum forever unless shutdown signal
}
