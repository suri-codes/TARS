use color_eyre::Result;
use common::DAEMON_ADDR;
use common::logging;
use tars_daemon::{DaemonState, Db, TarsDaemon};

#[tokio::main]
async fn main() -> Result<()> {
    logging::init("tars-d.log", true)?;

    let db = Db::new(false).await?;

    let state = DaemonState::new(db, DAEMON_ADDR);

    //TODO: also create a notifier thread later
    let daemon = TarsDaemon::init(state).await;

    daemon.run().await
}
