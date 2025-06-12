use common::DAEMON_ADDR;
use tars_daemon::{DaemonState, Db, TarsDaemon};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    // TODO: need to integrate a daemon log file aswell
    let db = Db::new(true).await;

    let state = DaemonState::new(db, DAEMON_ADDR);

    //TODO: also create a notifier thread later
    let daemon = TarsDaemon::init(state).await;

    daemon.run().await;
}
