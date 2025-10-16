use std::time::Duration;

use color_eyre::eyre::eyre;
use common::{TarsClient, types::Group};
use provider_types::{ProviderRegistration, ProviderRuntime, RunResult};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use toml::Value;

use tracing::{error, info};

use crate::recur_task::RecurringTask;
mod recur_task;

#[derive(Serialize, Deserialize, Debug)]
pub struct RecurringProviderConfig {
    events: Vec<RecurringTask>,
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
                for event in self.config.events.iter() {
                    let groups = Group::fetch_all(&client).await?;

                    // let group =
                    //     groups
                    //         .iter()
                    //         .find(|e| *e.name == event.group_name)
                    //         .ok_or(eyre!(format!(
                    //             "Group: {}, does not exist! unable to create event: {}",
                    //             event.group_name, event.name
                    //         )))?;

                    // let task = Task::new(
                    //     &client,
                    //     group,
                    //     event.name.clone(),
                    //     event.priority,
                    //     event.description.clone(),
                    //     event.due,
                    // )
                    // .await?;
                    // 3
                    // 3

                    // info!("{task:#?}");
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
