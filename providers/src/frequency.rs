use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy)]
pub enum Frequency {
    Second,
    #[default]
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

impl From<Frequency> for Duration {
    fn from(value: Frequency) -> Self {
        // this is too simplistic, think about how we actually want to set this up
        //
        // because we ideally want to refresh on like set times
        //
        // think about scenarios where we are opening and closing the
        //
        // application a lot, we dont want to spam providers

        let secs = match value {
            Frequency::Second => 1,

            Frequency::Hourly => 60 * 60,

            Frequency::Daily => 60 * 60 * 24,

            Frequency::Weekly => 60 * 60 * 24 * 7,

            Frequency::Monthly => 60 * 60 * 24 * 7 * 4,
        };

        Duration::from_secs(secs)
    }
}
