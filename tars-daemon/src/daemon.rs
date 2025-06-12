use axum::{
    Extension, Router,
    extract::State,
    routing::{get, post},
};
use sqlx::{Pool, Sqlite};
use tokio::net::TcpListener;
use tracing::{error, info};

use crate::{
    db::Db,
    handlers::test,
    // handlers::{add_group_handlers, add_task_handlers},
};

pub struct TarsDaemon {
    app: TarsRouter,
}

pub type TarsRouter = Router;

impl TarsDaemon {
    pub async fn init(db: Db) -> Self {
        let app: TarsRouter = Router::new()
            .route("/", get(root))
            .route("/task/test", post(test))
            .with_state(db.pool);

        // let app = add_task_handlers(app);
        // let app = add_group_handlers(app);

        info!("final router: {:?}", app);

        Self { app }
    }

    pub async fn run(self, addr: &str) {
        let listener = TcpListener::bind(addr).await.unwrap();

        info!("App lisening on {}", addr);

        if let Err(e) = axum::serve(listener, self.app).await {
            error!("{e}");
            panic!("{e}")
        };

        // With this:
    }
}

async fn root() -> &'static str {
    "ligma nuts pal"
}
