use std::time::Duration;

use bitflags::bitflags;
use chrono::NaiveDateTime;
use common::{
    TarsClient,
    types::{Color, Group},
};
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
pub struct RecurringProvider;

const RECURRING_ID: &str = "recurring";

impl ProviderRuntime for RecurringProvider {
    type Config = RecurringProviderConfig;

    fn id(&self) -> &'static str {
        RECURRING_ID
    }

    fn parse_config(&self, raw: &Value) -> RecurringProviderConfig {
        RecurringProviderConfig { events: Vec::new() }
        // raw.clone().try_into().expect("Failed to parse MyConfig") // or serde::Deserialize
    }

    fn run(
        &self,
        config: &RecurringProviderConfig,
        client: TarsClient,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async move {
            loop {
                info!("running recurring!");
                info!("running client!");
                sleep(Duration::from_secs(5)).await;
                // Group::new(
                //     &client,
                //     "default",
                //     None,
                //     common::types::Priority::Far,
                //     Color::random(),
                // )
                // .await
                // .unwrap();
            }
        })
    }
}

inventory::submit! {
    ProviderRegistration {
        id: RECURRING_ID,
        create_and_run: |raw: &Value| {
            let recurring_provider = RecurringProvider;
            let cfg = recurring_provider.parse_config(raw);
            Box::pin(async move {

                let client = TarsClient::default().await.unwrap();
                recurring_provider.run(&cfg, client).await;
            })
        }
    }
}
