pub mod group;
pub mod name;
pub mod priority;

use group::Group;
use name::Name;
use priority::Priority;
use sqlx::types::chrono::NaiveDateTime;

#[derive(PartialEq, Eq, Debug)]
pub struct Task {
    pub group: Group,
    pub name: Name,
    pub priority: Priority,
    pub description: String,
    pub due: Option<NaiveDateTime>,
}

impl Task {
    pub fn new(
        group: Group,
        name: Name,
        priority: Priority,
        description: String,
        due: Option<NaiveDateTime>,
    ) -> Self {
        Self {
            group,
            name,
            priority,
            description,
            due,
        }
    }
}
