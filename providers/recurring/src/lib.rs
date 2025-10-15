use std::any::Any;

use bitflags::bitflags;
use chrono::NaiveDateTime;
use common::TarsClient;
use provider_types::{ProviderRegistration, ProviderRuntime, register_provider};
use serde::{Deserialize, Serialize};
use toml::Value;

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
    fn id(&self) -> &'static str {
        RECURRING_ID
    }

    fn register(&self, config: &Value) {
        register_provider(RECURRING_ID, |value| {
            println!("{:#?}", value);

            let cfg = RecurringProviderConfig { events: Vec::new() };

            Box::new(cfg)
        });
    }

    fn run(
        &self,
        config: &Box<dyn Any>,
        client: &TarsClient,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send>> {
        let config = config
            .downcast_ref::<RecurringProviderConfig>()
            .expect("should be valid");

        Box::pin(async move {
            println!("running recurring!");
            println!("running client!")
        })
    }
}
inventory::submit! {ProviderRegistration {
    id: RECURRING_ID,
    runtime: &RecurringProvider
}}
