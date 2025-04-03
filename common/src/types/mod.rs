mod group;
mod id;
mod name;
mod priority;

pub use group::*;
pub use id::*;
pub use name::*;
pub use priority::*;

use sqlx::types::chrono::NaiveDateTime;

#[derive(PartialEq, Eq, Debug)]
pub struct Task {
    pub id: Id,
    pub group: Group,
    pub name: Name,
    pub priority: Priority,
    pub description: String,
    pub completed: bool,
    pub due: Option<NaiveDateTime>,
}

impl Task {
    pub fn with_all_fields(
        id: Id,
        group: Group,
        name: Name,
        priority: Priority,
        description: String,
        completed: bool,
        due: Option<NaiveDateTime>,
    ) -> Self {
        Self {
            id,
            group,
            name,
            priority,
            description,
            completed,
            due,
        }
    }
    pub fn new(
        group: Group,
        name: Name,
        priority: Priority,
        description: String,
        due: Option<NaiveDateTime>,
    ) -> Self {
        Self {
            id: Default::default(),
            group,
            name,
            priority,
            completed: false,
            description,
            due,
        }
    }
}
