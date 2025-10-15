use color_eyre::Result;
use common::DAEMON_ADDR;
use common::logging;
use tars_daemon::{DaemonState, Db, TarsDaemon};

// need to do this to link to providers
use recurring as _;

#[tokio::main]
async fn main() -> Result<()> {
    logging::init("tars-d.log", true)?;

    let db = Db::new(false).await?;

    let state = DaemonState::new(db, DAEMON_ADDR);

    let daemon = TarsDaemon::init(state).await;
    daemon.run().await
}
