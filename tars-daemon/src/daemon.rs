use axum::{Extension, Router, routing::get};
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
            .layer(Extension(db.pool));

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
