use axum::{
    Extension, Router,
    routing::{get, post},
};
use common::{DAEMON_ADDR, dirs::get_data_dir};
use handlers::{add_group_handlers, add_task_handlers};
use sqlx::{
    Pool, Sqlite, SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
};
use std::{fs::create_dir_all, str::FromStr};
use tokio::net::TcpListener;
use tracing::{error, info};

mod handlers;

const DB_FILE_NAME: &str = "tars.db";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    // TODO: need to integrate a daemon log file aswell

    //TODO: also create a notifier thread later

    let app = Router::new()
        .route("/", get(root))
        .layer(Extension(create_pool().await));

    let app = add_task_handlers(app);
    let app = add_group_handlers(app);

    let listener = TcpListener::bind(DAEMON_ADDR).await.unwrap();

    info!("App lisening on {}", DAEMON_ADDR);

    if let Err(e) = axum::serve(listener, app).await {
        error!("{e}");
        panic!("{e}")
    };
}

async fn root() -> &'static str {
    "ligma nuts pal"
}

async fn create_pool() -> Pool<Sqlite> {
    let mut data_dir = get_data_dir();

    let _ = create_dir_all(&data_dir);

    data_dir.push(DB_FILE_NAME);

    let db_path = data_dir;

    let full_path = format!(
        "sqlite://{}",
        db_path
            .to_str()
            .expect("Database Path should be a valid string.")
    );

    let sqlite_opts = SqliteConnectOptions::from_str(&full_path)
        .expect("failed")
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal);

    let Ok(pool) = SqlitePool::connect_with(sqlite_opts.clone()).await else {
        let msg = format!(
            "Failed to create sqlite pool. Connection Options: {:?}",
            sqlite_opts,
        );
        error!("{msg}");
        panic!("{msg}")
    };

    pool
}
