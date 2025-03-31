use std::fs::{File, OpenOptions, create_dir_all};
use std::str::FromStr;

use color_eyre::eyre::{Result, eyre};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::{ConnectOptions, SqliteConnection};

use crate::dirs::get_data_dir;

const DB_FILE_NAME: &str = "tars.db";

pub struct Db {
    conn: SqliteConnection,
}

impl Db {
    pub async fn connect() -> Result<Self> {
        let mut data_dir = get_data_dir();
        // create the directory
        let _ = create_dir_all(&data_dir);

        data_dir.push(DB_FILE_NAME);

        let db_path = data_dir;

        let full_path = format!(
            "sqlite://{}",
            db_path.to_str().ok_or(eyre!("Failed to convert path"))?
        );

        println!("db path: {}", &full_path);

        let conn = SqliteConnectOptions::from_str(&full_path)?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .connect()
            .await?;

        Ok(Db { conn })
    }
}
