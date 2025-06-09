use std::fs::create_dir_all;
use std::str::FromStr;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::{ConnectOptions, SqliteConnection};
use thiserror::Error;

use crate::dirs::get_data_dir;

mod fetch;
mod insert;
pub use fetch::*;
pub use insert::*;

const DB_FILE_NAME: &str = "tars.db";

/// Represents the Sqlite database.
/// Has methods to interact with stored data.
pub struct ORM {
    conn: SqliteConnection,
}

#[derive(Error, Debug)]
pub enum ORMError {
    #[error("Failed to convert type")]
    ConversionError(#[from] crate::ParseError),

    #[error("Sqlx Error!")]
    SqlxError(#[from] sqlx::Error),
}

impl ORM {
    /// Connects to the sqlite database located
    /// in the data directory for the app
    pub async fn connect() -> Result<Self, ORMError> {
        let mut data_dir = get_data_dir();
        // create the directory
        let _ = create_dir_all(&data_dir);

        data_dir.push(DB_FILE_NAME);

        let db_path = data_dir;

        let full_path = format!(
            "sqlite://{}",
            db_path
                .to_str()
                .expect("Database Path should be a valid string.")
        );

        let conn = SqliteConnectOptions::from_str(&full_path)?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .connect()
            .await?;

        Ok(ORM { conn })
    }
}
