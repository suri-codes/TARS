use std::path::Path;

use common::{dirs::get_data_dir, types::Id};
use sqlx::{Pool, Sqlite};
use tokio::fs::create_dir_all;

// A temporary sqlite db useful for testing
struct TempDB(String);

impl TempDB {
    pub async fn new() -> Self {
        // TODO: actually create the db here, apply migrations, and
        // add a drop impl that deletes the file.
        let db = TempDB(format!("/tmp/tars/{}.db", *Id::default()));

        let x = sqlx::migrate!("./migrations").run(&pool).await;

        todo!()
    }
}
