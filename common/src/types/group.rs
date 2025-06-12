use serde::{Deserialize, Serialize};

use super::{Id, Name};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Group {
    pub id: Id,
    pub name: Name,
}

impl Group {
    pub fn with_all_fields(id: impl Into<Id>, name: impl Into<Name>) -> Self {
        Group {
            id: id.into(),
            name: name.into(),
        }
    }
}
