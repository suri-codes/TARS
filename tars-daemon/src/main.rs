use std::fs::OpenOptions;

use common::{DAEMON_ADDR, dirs::get_data_dir};
use tars_daemon::{DaemonState, Db, TarsDaemon};

use tracing::{Level, level_filters::LevelFilter};
use tracing_subscriber::{Layer, Registry, fmt, layer::SubscriberExt};

#[tokio::main]
async fn main() {
    let mut log_file_path = get_data_dir();
    log_file_path.push("tars-d.log");

    let log_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&log_file_path)
        .unwrap_or_else(|_| panic!("Unable to open log file at path: {:?}", log_file_path));

    let subscriber = Registry::default()
        .with(
            fmt::layer()
                .compact()
                .with_ansi(true)
                .with_file(true)
                .with_line_number(true)
                .with_filter(LevelFilter::from_level(Level::INFO)),
        )
        .with(
            fmt::layer()
                .with_writer(log_file)
                .with_filter(LevelFilter::from_level(Level::DEBUG)),
        );

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let db = Db::new(true).await;

    let state = DaemonState::new(db, DAEMON_ADDR);

    //TODO: also create a notifier thread later
    let daemon = TarsDaemon::init(state).await;

    daemon.run().await;
}
