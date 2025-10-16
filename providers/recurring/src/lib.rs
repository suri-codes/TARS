use std::time::Duration;

use bitflags::bitflags;
use chrono::NaiveDateTime;
use common::TarsClient;
use provider_types::{ProviderRegistration, ProviderRuntime, RunResult};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use toml::Value;

use tracing::{error, info};
bitflags! {
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Days: u32 {
    const MON = 0b0000001;
    const TUE = 0b0000010;
    const WED = 0b0000100;
    const THU = 0b0001000;
    const FRI = 0b0010000;
    const SAT = 0b0100000;
    const SUN = 0b1000000;

    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RepeatInterval {
    Daily,
    Weekly,
    BiWeekly,
    Monthly,
    Yearly,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RecurringEvent {
    name: String,
    start_time: NaiveDateTime,
    days: Days,
    repeats: RepeatInterval,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RecurringProviderConfig {
    events: Vec<RecurringEvent>,
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

    fn run(self, _client: TarsClient) -> RunResult {
        Box::pin(async move {
            loop {
                for event in self.config.events.iter() {
                    info!("{event:#?}");
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
