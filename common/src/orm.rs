use std::fs::create_dir_all;
use std::str::FromStr;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::{ConnectOptions, SqliteConnection};
use thiserror::Error;

use crate::dirs::get_data_dir;
use crate::types::Task;

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

/// Differnt fetch types to specify
/// ways to gather Tasks from the database.
pub enum FetchType {
    ByGroup { group: String },
    All,
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

        // println!("db path: {}", full_path);

        let conn = SqliteConnectOptions::from_str(&full_path)?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .connect()
            .await?;

        Ok(ORM { conn })
    }

    /// Adds todo entry
    pub async fn add_entry(&mut self, task: Task) -> Result<(), ORMError> {
        let p = task.priority as i64;

        let record= sqlx::query!(
            r#"
                INSERT INTO Tasks (pub_id, group_id, name, priority, description, due)
                VALUES (
                    ?,
                    (SELECT pub_id FROM Groups WHERE name = ?),
                    ?,
                    ?,
                    ?,
                    ?
                )
                RETURNING Tasks.pub_id, Tasks.name, Tasks.priority as "priority: i64", Tasks.description, Tasks.due, Tasks.group_id, Tasks.completed
            "#,
            *task.id,
            *task.group,
            *task.name,
            p,
            task.description,
            task.due
        )
        .fetch_one(&mut self.conn)
        .await?;

        let group_name =
            sqlx::query_scalar!("SELECT name FROM Groups WHERE id = $1", record.group_id)
                .fetch_one(&mut self.conn)
                .await?;

        let created_task = Task::with_all_fields(
            record.pub_id.try_into()?,
            record.name.try_into()?,
            group_name.try_into()?,
            record.priority.try_into()?,
            record.description,
            record.completed,
            record.due,
        );

        assert_eq!(task, created_task);

        Ok(())
    }
}
