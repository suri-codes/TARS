use std::time::Duration;

use bitflags::bitflags;
use chrono::NaiveDateTime;
use color_eyre::eyre::eyre;
use common::{
    TarsClient, TarsResult,
    types::{Group, Priority, Task},
};
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Serialize, Deserialize, Debug)]
    struct Days: u32 {
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
enum RepeatInterval {
    Daily,
    Weekly,
    BiWeekly,
    Monthly,
    Yearly,
}

#[derive(Serialize, Deserialize, Debug)]
struct Depth(u8);

impl Default for Depth {
    fn default() -> Self {
        Depth(1)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RecurringTask {
    /// Name of the task
    name: String,

    /// Path to group
    #[serde(rename = "group")]
    group_name: String,

    /// The Optional Due Date for this task, (if unset, will assign the calculated duedate from days / repeat interval)
    due: Option<NaiveDateTime>,

    /// Task Priority
    #[serde(default)]
    priority: Priority,

    /// Task Description
    #[serde(default)]
    description: String,

    /// What days this task is on
    days: Days,

    /// How often this event should be repeated
    repeats: RepeatInterval,

    /// How many events into the future should be shown
    #[serde(default)]
    depth: Depth,
}

impl RecurringTask {
    fn canonicalize_group(&self) -> TarsResult<Option<Group>> {
        unimplemented!()
    }
}
