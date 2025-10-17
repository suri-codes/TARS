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
