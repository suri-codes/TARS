use std::time::Duration;

use bitflags::bitflags;
use chrono::NaiveDateTime;
use common::{TarsClient, TarsError};
use provider_types::{ProviderRegistration, ProviderRuntime};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use toml::Value;

use tracing::info;
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
    Weeky,
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
    _config: RecurringProviderConfig,
}

//NOTE: this should be try_from
impl TryFrom<&Value> for RecurringProviderConfig {
    fn try_from(_value: &Value) -> Result<Self, Self::Error> {
        // Err(TarsError::Parse(ParseError::FailedToParse))

        Ok(Self { events: Vec::new() })
    }

    type Error = TarsError;
}

impl RecurringProvider {
    pub fn new(config: RecurringProviderConfig) -> Self {
        RecurringProvider { _config: config }
    }
}

const RECURRING_ID: &str = "recurring";

impl ProviderRuntime for RecurringProvider {
    fn id(&self) -> &'static str {
        RECURRING_ID
    }

    fn run(&self, _client: TarsClient) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async move {
            loop {
                info!("running recurring!");
                info!("running client!");
                sleep(Duration::from_secs(5)).await;
            }
        })
    }
}

inventory::submit! {
    ProviderRegistration {
        id: RECURRING_ID,
        create_and_run: |raw: &Value, client: TarsClient| {
            let cfg = RecurringProviderConfig::try_from(raw).unwrap();
            let recurring_provider = RecurringProvider::new(cfg);
            Box::pin(async move {
                recurring_provider.run(client).await;
            })
        }
    }
}
