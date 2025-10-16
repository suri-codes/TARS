use std::time::Duration;

use common::TarsClient;
use providers::{ProviderRegistration, ProviderRuntime, RunResult};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use toml::Value;

use tracing::{error, info};

use crate::recur_task::RecurringTask;
mod recur_task;

#[derive(Serialize, Deserialize, Debug)]
pub struct RecurringProviderConfig {
    tasks: Vec<RecurringTask>,
    // TODO: work on the frequency inside the providers crate and use that here instead
    // update_period: Duration,
}

/// A simple provider that can handle recurring events
pub struct RecurringProvider {
    config: RecurringProviderConfig,
}

impl RecurringProvider {
    pub fn new(config: RecurringProviderConfig) -> Self {
        RecurringProvider { config }
    }
}

const RECURRING_ID: &str = "recurring";

impl ProviderRuntime for RecurringProvider {
    fn id(&self) -> &'static str {
        RECURRING_ID
    }

    fn run(self, client: TarsClient) -> RunResult {
        Box::pin(async move {
            loop {
                for recur_task in self.config.tasks.iter() {
                    recur_task.materialize_tasks(&client).await?;
                }

                sleep(Duration::from_secs(5)).await;
            }
        })
    }
}

inventory::submit! {
    ProviderRegistration {
        id: RECURRING_ID,
        create_and_run: |config: Value, client: TarsClient| -> RunResult{
            Box::pin(async move {

                info!("{config:#?}");

                let cfg: RecurringProviderConfig = config.try_into().inspect_err(|e|{
                    error!("{e}")
                })?;

                let recurring_provider = RecurringProvider::new(cfg);

                recurring_provider.run(client).await.inspect_err(|e|{
                    error!("{e}")
                })

            })
        }
    }
}
