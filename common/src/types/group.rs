use std::fmt::Display;

use color_eyre::owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{TarsClient, TarsError};

use super::{Id, Name};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, PartialOrd, Ord)]
pub struct Group {
    pub id: Id,
    pub name: Name,
    pub parent_id: Option<Id>,
}

impl Group {
    pub fn with_all_fields(
        id: impl Into<Id>,
        name: impl Into<Name>,
        parent_id: Option<Id>,
    ) -> Self {
        Group {
            id: id.into(),
            name: name.into(),
            parent_id,
        }
    }

    /// Creates a new `Group` through the `TarsDaemon`.
    ///
    /// # Errors
    ///
    /// This function will return an error if
    /// Something goes wrong with the requests to the Daemon.
    pub async fn new(
        client: &TarsClient,
        name: impl Into<Name>,
        parent_id: Option<Id>,
    ) -> Result<Self, TarsError> {
        let group = Group::with_all_fields(Id::default(), name, parent_id);

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

    /// Fetches all `Group`s.
    ///
    /// # Errors
    ///
    /// This function will return an error if
    /// Something goes wrong with the requests to the Daemon.
    pub async fn fetch_all(client: &TarsClient) -> Result<Vec<Group>, TarsError> {
        let res: Vec<Group> = client
            .conn
            .get(client.base_path.join("/group")?)
            .send()
            .await
            .inspect_err(|e| error!("Error Fetching Group: {:?}", e))?
            .json()
            .await
            .inspect_err(|e| error!("Error Fetching Group: {:?}", e))?;

        Ok(res)
    }

    /// Sync's this `Group` with its representation in database, via the `TarsDaemon`.
    ///
    /// # Errors
    ///
    /// This function will return an error if
    /// + Something goes wrong with the requests to the Daemon.
    /// + Will panic at runtime if the sync'd `Group` doesnt match with `self`
    pub async fn sync(&self, client: &TarsClient) -> Result<(), TarsError> {
        let res: Group = client
            .conn
            .post(client.base_path.join("/group/update")?)
            .json(self)
            .send()
            .await
            .inspect_err(|e| error!("Error Sync'ing Group: {:?}", e))?
            .json()
            .await
            .inspect_err(|e| error!("Error Sync'ing Group: {:?}", e))?;

        assert_eq!(res, *self);

        Ok(())
    }

    /// Deletes this `Group` via the `TarsDaemon`.
    ///
    /// # Errors
    ///
    /// This function will return an error if
    /// + Something goes wrong with the requests to the Daemon.
    /// + Will panic at runtime if deleted `Group` doesnt match the `Group` we wanted to delete.
    pub async fn delete(self, client: &TarsClient) -> Result<(), TarsError> {
        let deleted: Group = client
            .conn
            .post(client.base_path.join("/group/delete")?)
            .json(&self)
            .send()
            .await
            .inspect_err(|e| error!("Error Deleting Group: {:?}", e))?
            .json()
            .await
            .inspect_err(|e| error!("Error Deleting Group: {:?}", e))?;

        assert_eq!(deleted, self);

        Ok(())
    }
}

impl Display for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Name: {}", (*self.name).green())?;
        write!(f, "Id: {}", *self.id)?;
        if let Some(ref parent_id) = self.parent_id {
            write!(f, "\nParent Id: {}", **parent_id)?;
        }

        Ok(())
    }
}
