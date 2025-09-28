use std::{f64, fmt::Display};

use chrono::NaiveDateTime;
use color_eyre::owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{TarsClient, TarsError};

use super::{Group, Id, Name, Priority};

/// Task type that holds all information relavant to a task.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, PartialOrd, Ord)]
pub struct Task {
    pub id: Id,
    pub name: Name,
    pub group: Group,
    pub priority: Priority,
    pub description: String,
    pub finished_at: Option<NaiveDateTime>,
    pub due: Option<NaiveDateTime>,
}

impl Task {
    /// Initializes a `Task` with all fields.
    pub fn with_all_fields(
        id: impl Into<Id>,
        group: impl Into<Group>,
        name: impl Into<Name>,
        priority: Priority,
        description: String,
        finished_at: Option<NaiveDateTime>,
        due: Option<NaiveDateTime>,
    ) -> Self {
        Self {
            id: id.into(),
            group: group.into(),
            name: name.into(),
            priority,
            description,
            finished_at,
            due,
        }
    }

    /// Creates a new `Task` through the `TarsDaemon`.
    ///
    /// # Errors
    ///
    /// This function will return an error if
    /// Something goes wrong with the requests to the Daemon.
    pub async fn new(
        client: &TarsClient,
        group: &Group,
        name: impl Into<Name>,
        priority: Priority,
        description: impl Into<String>,

        due: Option<NaiveDateTime>,
    ) -> Result<Self, TarsError> {
        let task = Self {
            id: Default::default(),
            group: group.clone(),
            name: name.into(),
            priority,
            finished_at: None,
            description: description.into(),
            due,
        };

        let res: Task = client
            .conn
            .post(client.base_path.join("/task/create")?)
            .json(&task)
            .send()
            .await
            .inspect_err(|e| error!("Error creating Task: {:?}", e))?
            .json()
            .await
            .inspect_err(|e| error!("Error creating Task: {:?}", e))?;

        Ok(res)
    }
    /// Forcefully creates this `Task`...
    /// WARN: If youre just trying to make a new task / don't know
    /// what youre doing, use `Task::new()` instead.
    ///
    /// # Errors
    ///
    /// This function will return an error if
    /// Something goes wrong with the requests to the Daemon.
    pub async fn raw_create(&self, client: &TarsClient) -> Result<(), TarsError> {
        let _: Task = client
            .conn
            .post(client.base_path.join("/task/create")?)
            .json(&self)
            .send()
            .await
            .inspect_err(|e| error!("Error creating Task: {:?}", e))?
            .json()
            .await
            .inspect_err(|e| error!("Error creating Task: {:?}", e))?;

        Ok(())
    }

    /// Fetches `Task`s that match the criteria specified by `TaskFetchOptions`.
    ///
    /// # Errors
    ///
    /// This function will return an error if
    /// Something goes wrong with the requests to the Daemon.
    pub async fn fetch(
        client: &TarsClient,
        opts: TaskFetchOptions,
    ) -> Result<Vec<Task>, TarsError> {
        let res: Vec<Task> = client
            .conn
            .post(client.base_path.join("task/fetch")?)
            .json(&opts)
            .send()
            .await
            .inspect_err(|e| error!("Error creating Task: {:?}", e))?
            .json()
            .await
            .inspect_err(|e| error!("Error creating Task: {:?}", e))?;

        Ok(res)
    }

    /// Sync's this `Task` with its representation in database, via the `TarsDaemon`.
    ///
    /// # Errors
    ///
    /// This function will return an error if
    /// + Something goes wrong with the requests to the Daemon.
    /// + Will panic at runtime if the sync'd task doesnt match with `self`
    pub async fn sync(&self, client: &TarsClient) -> Result<(), TarsError> {
        let task: Task = client
            .conn
            .post(client.base_path.join("/task/update")?)
            .json(&self)
            .send()
            .await
            .inspect_err(|e| error!("Error creating Task: {:?}", e))?
            .json()
            .await
            .inspect_err(|e| error!("Error creating Task: {:?}", e))?;

        // make sure that its actually sync'd
        assert_eq!(*self, task);
        Ok(())
    }

    /// Deletes this `Task` via the `TarsDaemon`.
    ///
    /// # Errors
    ///
    /// This function will return an error if
    /// + Something goes wrong with the requests to the Daemon.
    /// + Will panic at runtime if deleted task doesnt match the task we wanted to delete.
    pub async fn delete(&self, client: &TarsClient) -> Result<(), TarsError> {
        let deleted_task: Task = client
            .conn
            .post(client.base_path.join("/task/delete")?)
            .json(&self.id)
            .send()
            .await
            .inspect_err(|e| error!("Error creating Task: {:?}", e))?
            .json()
            .await
            .inspect_err(|e| error!("Error creating Task: {:?}", e))?;

        assert_eq!(deleted_task, *self);

        Ok(())
    }

    /// Returns the p score of this [`Task`].
    pub async fn p_score(&self, client: &TarsClient) -> Result<f64, TarsError> {
        let score: f64 = client
            .conn
            .post(client.base_path.join("/task/score")?)
            .json(&self.id)
            .send()
            .await
            .inspect_err(|e| error!("Error fetching score for Task: {:?}", e))?
            .json()
            .await
            .inspect_err(|e| error!("Error parsing score for Task: {:?}", e))?;

        Ok(score)
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Name: {}", (*self.name).green())?;
        writeln!(f, "Id: {}", *self.id)?;
        writeln!(f, "Group: {}", (*self.group.name).green())?;
        match self.priority {
            Priority::Far => writeln!(f, "Priority: {}", "Far".blue()),
            Priority::Low => writeln!(f, "Priority: {}", "Low".green()),
            Priority::Medium => writeln!(f, "Priority: {}", "Medium".yellow()),
            Priority::High => writeln!(f, "Priority: {}", "High".red()),
            Priority::Asap => writeln!(f, "Priority: {}", "ASAP".red()),
        }?;
        writeln!(f, "Description:\n {}", self.description)?;
        if let Some(time) = self.finished_at {
            let str = format!("Completed at {} ", time.format("%m/%d/%Y %I:%M:%S %p"));
            let str = str.green();
            write!(f, "{str}",)?;
        } else {
            write!(f, "{}", "Incomplete! ❌".red())?;
        }

        if let Some(due_date) = self.due {
            writeln!(f, "{due_date}")?;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TaskFetchOptions {
    // NOTE: only add those we deem necessary, dont have to add shit just to add them
    // ById { id: Id },
    ByGroup { group_id: Id, recursive: bool },
    All,
}
