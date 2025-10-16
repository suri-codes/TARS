use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use color_eyre::owo_colors::OwoColorize;
use tracing::error;

use crate::types::{Color, Priority};
use crate::{TarsClient, TarsResult};

use super::{Id, Name};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, PartialOrd, Ord)]
/// A group is what represents a collection of tasks or other groups that share some property.
pub struct Group {
    pub id: Id,
    pub name: Name,
    pub priority: Priority,
    pub parent_id: Option<Id>,
    pub created_at: NaiveDateTime,
    pub color: Color,
}

impl Group {
    pub fn with_all_fields(
        id: impl Into<Id>,
        name: impl Into<Name>,
        parent_id: Option<Id>,
        priority: Priority,
        created_at: NaiveDateTime,
        color: Color,
    ) -> Self {
        Group {
            id: id.into(),
            name: name.into(),
            parent_id,
            priority,
            created_at,
            color,
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
        priority: Priority,
        color: Color,
    ) -> TarsResult<Self> {
        let created_at = Local::now().naive_local();

        let group =
            Group::with_all_fields(Id::default(), name, parent_id, priority, created_at, color);

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

    /// Forcefully creates this `Group`...
    /// WARN: If youre just trying to make a new group / don't know
    /// what youre doing, use `Group::new()` instead.
    ///
    /// # Errors
    ///
    /// This function will return an error if
    /// Something goes wrong with the requests to the Daemon.
    pub async fn raw_create(&self, client: &TarsClient) -> TarsResult<()> {
        let _: Group = client
            .conn
            .post(client.base_path.join("/group/create")?)
            .json(&self)
            .send()
            .await
            .inspect_err(|e| error!("Error creating Group: {:?}", e))?
            .json()
            .await
            .inspect_err(|e| error!("Error creating Group: {:?}", e))?;

        Ok(())
    }

    /// Fetches all `Group`s.
    ///
    /// # Errors
    ///
    /// This function will return an error if
    /// Something goes wrong with the requests to the Daemon.
    pub async fn fetch_all(client: &TarsClient) -> TarsResult<Vec<Group>> {
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
    pub async fn sync(&self, client: &TarsClient) -> TarsResult<()> {
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
    pub async fn delete(&self, client: &TarsClient) -> TarsResult<()> {
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

        assert_eq!(deleted, *self);

        Ok(())
    }

    /// Returns the p score of this [`Group`].
    pub async fn p_score(&self, client: &TarsClient) -> TarsResult<f64> {
        let score: f64 = client
            .conn
            .post(client.base_path.join("/group/score")?)
            .json(&self.id)
            .send()
            .await
            .inspect_err(|e| error!("Error fetching score for Group: {:?}", e))?
            .json()
            .await
            .inspect_err(|e| error!("Error parsing score for Group: {:?}", e))?;

        Ok(score)
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
