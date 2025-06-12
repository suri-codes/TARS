use std::{fs::create_dir_all, str::FromStr};

use axum::{Extension, Router, routing::get};
use common::dirs::get_data_dir;
use sqlx::{
    Pool, Sqlite, SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
};
use tokio::net::TcpListener;
use tracing::{error, info};

use crate::{
    db::Db,
    handlers::{add_group_handlers, add_task_handlers},
};

pub struct TarsDaemon {
    app: Router,
}

impl TarsDaemon {
    pub async fn init(db: Db) -> Self {
        let app = Router::new()
            .route("/", get(root))
            // TODO: fix this bruh
            // .layer(Extension(create_pool(db_url).await));
            ;

        let app = add_task_handlers(app);
        let app = add_group_handlers(app);

        Self { app }
    }

    pub async fn run(self, addr: &str) {
        let listener = TcpListener::bind(addr).await.unwrap();

        info!("App lisening on {}", addr);

        if let Err(e) = axum::serve(listener, self.app).await {
            error!("{e}");
            panic!("{e}")
        };
    }
}

async fn root() -> &'static str {
    "ligma nuts pal"
}

async fn create_pool(db_url: &str) -> Pool<Sqlite> {
    let mut data_dir = get_data_dir();

    let _ = create_dir_all(&data_dir);

    data_dir.push(db_url);

    let db_path = data_dir;

    let full_path = format!(
        "sqlite://{}",
        db_path
            .to_str()
            .expect("Database Path should be a valid string.")
    );

    let sqlite_opts = {
        SqliteConnectOptions::from_str(&full_path)
            .inspect_err(|e| {
                error!(
                    "Failed to connect to sqlite db at url: {}. error: {}",
                    full_path, e
                )
            })
            .unwrap()
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
    };

    SqlitePool::connect_with(sqlite_opts.clone())
        .await
        .inspect_err(|e| {
            error!(
                "Failed to create sqlite pool. Connection Options: {:?}. Error: {}",
                sqlite_opts, e
            )
        })
        .unwrap()
}
