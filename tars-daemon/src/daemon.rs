use axum::{Router, routing::get};
use sqlx::{Pool, Sqlite};
use tokio::net::TcpListener;
use tracing::{error, info};

use crate::{
    db::Db,
    handlers::{group_router, task_router},
};

pub struct TarsDaemon {
    app: Router,
    state: DaemonState,
}

#[derive(Clone)]
pub struct DaemonState {
    pub pool: Pool<Sqlite>,
    addr: String,
}

impl DaemonState {
    pub fn new(db: Db, addr: &str) -> Self {
        DaemonState {
            pool: db.pool,
            addr: addr.to_owned(),
        }
    }
}

impl TarsDaemon {
    pub async fn init(state: DaemonState) -> Self {
        let app = Router::new()
            .route("/", get(root))
            .nest("/task", task_router())
            .nest("/group", group_router())
            .with_state(state.clone());

        Self { app, state }
    }

    pub async fn run(self) {
        let listener = TcpListener::bind(&self.state.addr).await.unwrap();

        info!("App lisening on {}", self.state.addr);

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
