use axum::{Router, routing::get};
use color_eyre::eyre::{Result, eyre};
use common::{Diff, TarsConfig};
use provider_types::ProviderRegistration;
use sqlx::{Pool, Sqlite};
use tokio::{
    net::TcpListener,
    sync::broadcast::{self, Sender},
    task::JoinError,
};
use tracing::{error, info, warn};

use crate::{
    db::Db,
    handlers::{group_router, subscribe_router, task_router},
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
    pub diff_tx: Sender<Diff>,
}

impl DaemonState {
    /// Returns a new instance of DaemonState
    pub fn new(db: Db, addr: &str) -> Self {
        let (tx, _) = broadcast::channel::<Diff>(50);

        DaemonState {
            pool: db.pool,
            addr: addr.to_owned(),
            diff_tx: tx,
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
            .nest("/subscribe", subscribe_router())
            .with_state(state.clone());

        Self { app, state }
    }

    /// Runs the daemon, will panic if something goes wrong.
    pub async fn run(self) -> Result<()> {
        let listener = TcpListener::bind(&self.state.addr).await?;
        info!("App listening on {}", self.state.addr);

        let axum_handle = tokio::spawn(async move {
            axum::serve(listener, self.app).await.map_err(|e| {
                error!("{:?}", e);
                eyre!(e)
            })
        });

        let providers_handle = tokio::spawn(async move { spawn_providers().await });

        let (axum_res, provider_res) = tokio::join!(axum_handle, providers_handle);

        match axum_res {
            Err(join_err) => return Err(eyre!("Axum task panicked: {}", join_err)),
            Ok(inner) => inner?, // This propagates serve(...) errors (eyre::Report)
        }

        match provider_res {
            Err(join_err) => warn!("Provider task panicked: {}", join_err),
            Ok(Err(e)) => warn!("Provider returned error: {}", e),
            Ok(Ok(())) => {} // All good
        }

        Ok(())
    }
}

async fn root() -> &'static str {
    "ligma nuts pal"
}

async fn spawn_providers() -> Result<()> {
    let provider_configs = {
        let Some(config_table) = TarsConfig::get_toml_table()? else {
            // no table exists, meaning trivially no providers exist either
            return Ok(());
        };

        info!("config_table: {config_table:#?}");
        // hard check for provider being a table
        if let Some(provider_table) = config_table.get("provider")
            && provider_table.is_table()
        {
            provider_table.as_table().cloned().unwrap()
        } else {
            return Ok(());
        }
    };

    for provider in inventory::iter::<ProviderRegistration> {
        if let Some(cfg) = provider_configs.get(provider.id) {
            tokio::spawn((provider.create_and_run)(cfg));
        }
    }

    Ok(())
}
