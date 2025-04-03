use sqlx::query_scalar;

use crate::types::{Group, Id, Name, Priority, Task};

use super::{ORM, ORMError};

/// Differnt fetch types to specify
/// ways to gather Tasks from the database.
pub enum FetchType {
    ByGroup { group: Group },
    All,
    ById { id: Id },
}

/// Completion status for different tasks
pub enum CompletionStatus {
    Any,
    Done,
    NotDone,
}

pub struct FetchOptions {
    fetch_type: FetchType,
    completion_status: CompletionStatus,
}

impl ORM {
    pub async fn fetch_groups(&mut self) -> Result<Vec<Group>, ORMError> {
        let results: Vec<Group> = query_scalar!(r#"SELECT name FROM Groups"#)
            .fetch_all(&mut self.conn)
            .await?
            .iter()
            .map(|e| e.as_str().try_into().expect("lol"))
            .collect();
        // let results = sqlx::query_as!(
        //     Group,
        //     r#"
        //             SELECT
        //             name as "group: Group"
        //             FROM Groups

        //     "#,
        // )
        // .fetch_all(&mut self.conn)
        // .await;

        todo!()
    }

    pub async fn fetch_tasks(
        &mut self,
        fetch_options: FetchOptions,
    ) -> Result<Vec<Task>, ORMError> {
        let res = match (fetch_options.fetch_type, fetch_options.completion_status) {
            // ByGroup variants
            (FetchType::ByGroup { group }, CompletionStatus::Any) => {
                sqlx::query_as!(
                    Task,
                    r#"
                    SELECT 
                        t.pub_id as "id: Id",
                        g.name as "group: Group",
                        t.name as "name: Name",
                        t.priority as "priority: Priority",
                        t.description,
                        t.completed,
                        t.due
                    FROM Tasks t
                    JOIN Groups g ON t.group_id = g.pub_id
                    WHERE g.name = ?
                "#,
                    *group
                )
                .fetch_all(&mut self.conn)
                .await
            }
            (FetchType::ByGroup { group }, CompletionStatus::Done) => {
                sqlx::query_as!(
                    Task,
                    r#"
                    SELECT 
                        t.pub_id as "id: Id",
                        g.name as "group: Group",
                        t.name as "name: Name",
                        t.priority as "priority: Priority",
                        t.description,
                        t.completed,
                        t.due
                    FROM Tasks t
                    JOIN Groups g ON t.group_id = g.pub_id
                    WHERE g.name = ? AND t.completed = true
                "#,
                    *group
                )
                .fetch_all(&mut self.conn)
                .await
            }
            (FetchType::ByGroup { group }, CompletionStatus::NotDone) => {
                // Similar pattern for not done
                sqlx::query_as!(
                    Task,
                    r#"
                    SELECT 
                        t.pub_id as "id: Id",
                        g.name as "group: Group",
                        t.name as "name: Name",
                        t.priority as "priority: Priority",
                        t.description,
                        t.completed,
                        t.due
                    FROM Tasks t
                    JOIN Groups g ON t.group_id = g.pub_id
                    WHERE g.name = ? AND t.completed = false
                "#,
                    *group
                )
                .fetch_all(&mut self.conn)
                .await
            }

            (FetchType::All, CompletionStatus::Any) => {
                sqlx::query_as!(
                    Task,
                    r#"
                    SELECT 
                        t.pub_id as "id: Id",
                        g.name as "group: Group",
                        t.name as "name: Name",
                        t.priority as "priority: Priority",
                        t.description,
                        t.completed,
                        t.due
                    FROM Tasks t
                    JOIN Groups g ON t.group_id = g.pub_id
                "#
                )
                .fetch_all(&mut self.conn)
                .await
            }
            (FetchType::All, CompletionStatus::Done) => {
                sqlx::query_as!(
                    Task,
                    r#"
                    SELECT 
                        t.pub_id as "id: Id",
                        g.name as "group: Group",
                        t.name as "name: Name",
                        t.priority as "priority: Priority",
                        t.description,
                        t.completed,
                        t.due
                    FROM Tasks t
                    JOIN Groups g ON t.group_id = g.pub_id
                    WHERE t.completed = true
                "#
                )
                .fetch_all(&mut self.conn)
                .await
            }
            (FetchType::All, CompletionStatus::NotDone) => {
                sqlx::query_as!(
                    Task,
                    r#"
                    SELECT 
                        t.pub_id as "id: Id",
                        g.name as "group: Group",
                        t.name as "name: Name",
                        t.priority as "priority: Priority",
                        t.description,
                        t.completed,
                        t.due
                    FROM Tasks t
                    JOIN Groups g ON t.group_id = g.pub_id
                    WHERE t.completed = false
                "#
                )
                .fetch_all(&mut self.conn)
                .await
            }

            (FetchType::ById { id }, CompletionStatus::Any) => {
                sqlx::query_as!(
                    Task,
                    r#"
                    SELECT 
                        t.pub_id as "id: Id",
                        g.name as "group: Group",
                        t.name as "name: Name",
                        t.priority as "priority: Priority",
                        t.description,
                        t.completed,
                        t.due
                    FROM Tasks t
                    JOIN Groups g ON t.group_id = g.pub_id
                    WHERE t.pub_id = ?
                "#,
                    *id
                )
                .fetch_all(&mut self.conn)
                .await
            }
            (FetchType::ById { id }, CompletionStatus::Done) => {
                sqlx::query_as!(
                    Task,
                    r#"
                    SELECT 
                        t.pub_id as "id: Id",
                        g.name as "group: Group",
                        t.name as "name: Name",
                        t.priority as "priority: Priority",
                        t.description,
                        t.completed,
                        t.due
                    FROM Tasks t
                    JOIN Groups g ON t.group_id = g.pub_id
                    WHERE t.pub_id = ? AND t.completed = true
                "#,
                    *id
                )
                .fetch_all(&mut self.conn)
                .await
            }
            (FetchType::ById { id }, CompletionStatus::NotDone) => {
                sqlx::query_as!(
                    Task,
                    r#"
                    SELECT 
                        t.pub_id as "id: Id",
                        g.name as "group: Group",
                        t.name as "name: Name",
                        t.priority as "priority: Priority",
                        t.description,
                        t.completed,
                        t.due
                    FROM Tasks t
                    JOIN Groups g ON t.group_id = g.pub_id
                    WHERE t.pub_id = ? AND t.completed = false
                "#,
                    *id
                )
                .fetch_all(&mut self.conn)
                .await
            }
        };

        Ok(res?)
    }
}
