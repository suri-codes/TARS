use common::DAEMON_ADDR;
use tars_daemon::{Db, TarsDaemon};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    // TODO: need to integrate a daemon log file aswell
    let db = Db::new(true).await;

    //TODO: also create a notifier thread later
    let daemon = TarsDaemon::init(db).await;

    daemon.run(DAEMON_ADDR).await;
}
