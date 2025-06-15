use axum::{Router, routing::get};
use color_eyre::eyre::{Result, eyre};
use sqlx::{Pool, Sqlite};
use tokio::net::TcpListener;
use tracing::{error, info};

use crate::{
    db::Db,
    handlers::{group_router, task_router},
};

/// Daemon that exposes access to the database, as well as being responsible
/// for sending notifications regarding task duedates.
pub struct TarsDaemon {
    app: Router,
    state: DaemonState,
}

// State required for the `TarsDaemon` to function properly.
#[derive(Clone, Debug)]
pub struct DaemonState {
    pub pool: Pool<Sqlite>,
    addr: String,
}

impl DaemonState {
    /// Returns a new instance of DaemonState
    pub fn new(db: Db, addr: &str) -> Self {
        DaemonState {
            pool: db.pool,
            addr: addr.to_owned(),
        }
    }
}

impl TarsDaemon {
    /// Initializes a new Daemon
    pub async fn init(state: DaemonState) -> Self {
        let app = Router::new()
            .route("/", get(root))
            .nest("/task", task_router())
            .nest("/group", group_router())
            .with_state(state.clone());

        Self { app, state }
    }

    /// Runs the daemon, will panic if something goes wrong.
    pub async fn run(self) -> Result<()> {
        let listener = TcpListener::bind(&self.state.addr).await?;

        info!("App lisening on {}", self.state.addr);

        axum::serve(listener, self.app).await.map_err(|e| {
            error!("{:?}", e);
            eyre!(e)
        })
    }
}

async fn root() -> &'static str {
    "ligma nuts pal"
}
