use std::fmt::Display;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::{Group, Id, Name, Priority};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
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
        id: impl Into<Id>,
        group: impl Into<Group>,
        name: impl Into<Name>,
        priority: Priority,
        description: String,
        completed: bool,
        due: Option<NaiveDateTime>,
    ) -> Self {
        Self {
            id: id.into(),
            group: group.into(),
            name: name.into(),
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

impl Display for Task {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TaskFetchOptions {
    // NOTE: only add those we deem necessary, dont have to add shit just to add them
    // ById { id: Id },
    // ByGroup { group: Group },
    All,
}
