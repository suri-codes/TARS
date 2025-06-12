use std::{path::PathBuf, str::FromStr};

use common::{dirs::get_data_dir, types::Id};
use sqlx::{
    Pool, Sqlite, SqlitePool,
    error::DatabaseError,
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
};
use tokio::{fs::create_dir_all, time::error};
use tracing::error;

const DB_FILE_NAME: &str = "tars.db";

pub struct Db {
    pub pool: Pool<Sqlite>,
}

impl Db {
    pub async fn new(is_test: bool) -> Self {
        let path = {
            let mut dir = if is_test {
                PathBuf::from(format!("/tmp/tars/{}/", *Id::default()))
            } else {
                get_data_dir()
            };

            let _ = create_dir_all(&dir).await;
            dir.push(DB_FILE_NAME);

            format!(
                "sqlite://{}",
                dir.to_str()
                    .expect("Database Path should be a valid string.")
            )
        };

        // if the db doesnt exist already, lets apply migrations to it too.
        let sqlite_opts = SqliteConnectOptions::from_str(&path)
            .inspect_err(|e| {
                error!(
                    "Failed to process path to sqlite db: {}. error: {}",
                    path, e
                )
            })
            .unwrap()
            // .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal);

        // ok lets try connecting
        let pool = match SqlitePool::connect_with(sqlite_opts.clone()).await {
            Ok(p) => Ok(p),
            Err(sqlx::Error::Database(_)) => {
                // database doesnt exist yet, create it
                let sqlite_opts = sqlite_opts.create_if_missing(true);

                // now connect to it
                let pool = SqlitePool::connect_with(sqlite_opts.clone())
                    .await
                    .inspect_err(|e| error!("{e}"))
                    .unwrap();

                // apply the migrations to it
                sqlx::migrate!("./migrations")
                    .run(&pool)
                    .await
                    .inspect_err(|e| error!("{e}"))
                    .unwrap();

                Ok(pool)
            }
            Err(e) => {
                error!("{e}");
                Err(e)
            }
        }
        .expect("Failed to establish connection pool.");

        Self { pool }
    }
}
