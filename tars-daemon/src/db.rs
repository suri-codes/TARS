use std::{path::PathBuf, str::FromStr};

use color_eyre::Result;
use common::{dirs::get_data_dir, types::Id};
use sqlx::{
    Pool, Sqlite, SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
};
use tokio::fs::create_dir_all;
use tracing::error;

/// Holds the pool to a database.
/// Mostly constructed as this type because it has logic to create a new db w migrations
/// or test databases.
///
/// NOTE: Will panic if anything goes wrong.
pub struct Db {
    pub pool: Pool<Sqlite>,
}

impl Db {
    /// Creates a new Db
    ///
    /// If database already exists, will just return a pool connected to that
    /// If not, creates a new one with migrations
    ///
    /// Can also create test databases.
    ///
    pub async fn new(is_test: bool) -> Result<Self> {
        let path = {
            let mut dir = if is_test {
                PathBuf::from(format!("/tmp/tars/{}/", *Id::default()))
            } else {
                get_data_dir()
            };

            let _ = create_dir_all(&dir).await;
            dir.push("tars.db");

            format!(
                "sqlite://{}",
                dir.to_str()
                    .expect("Database Path should be a valid string.")
            )
        };

        let sqlite_opts = SqliteConnectOptions::from_str(&path)
            .inspect_err(|e| {
                error!(
                    "Failed to process path to sqlite db: {}. error: {}",
                    path, e
                )
            })
            .unwrap()
            .journal_mode(SqliteJournalMode::Wal);

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
        }?;

        Ok(Self { pool })
    }
}
