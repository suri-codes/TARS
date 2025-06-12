use common::DAEMON_ADDR;
use daemon::TarsDaemon;
use db::Db;

mod daemon;
mod db;
mod handlers;
mod utils;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    // TODO: need to integrate a daemon log file aswell
    let db = Db::new(true).await;

    //TODO: also create a notifier thread later
    let daemon = TarsDaemon::init(db).await;

    daemon.run(DAEMON_ADDR).await;
}
