use common::DAEMON_ADDR;
use daemon::TarsDaemon;

mod daemon;
mod handlers;
const DB_FILE_NAME: &str = "tars.db";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    // TODO: need to integrate a daemon log file aswell

    //TODO: also create a notifier thread later
    let daemon = TarsDaemon::init(DB_FILE_NAME).await;

    daemon.run(DAEMON_ADDR).await;
}
