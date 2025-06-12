use common::DAEMON_ADDR;
use daemon::TarsDaemon;
use db::Db;

mod daemon;
mod db;
mod handlers;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    // TODO: need to integrate a daemon log file aswell

    //TODO: also create a notifier thread later
    // let daemon = TarsDaemon::init().await;

    // daemon.run(DAEMON_ADDR).await;
    //
    let db = Db::new(true).await;
}
