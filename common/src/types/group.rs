use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{TarsClient, TarsError};

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

    pub async fn new(client: &TarsClient, name: impl Into<Name>) -> Result<Self, TarsError> {
        let group = Group::with_all_fields(Id::default(), name);

        let res: Group = client
            .conn
            .post(client.base_path.join("/group/create")?)
            .json(&group)
            .send()
            .await
            .inspect_err(|e| error!("Error creating Group: {:?}", e))?
            .json()
            .await
            .inspect_err(|e| error!("Error creating Group: {:?}", e))?;

        Ok(res)
    }
}
