use super::{ORM, ORMError};
use crate::types::{Group, Id, Task};

/// Adds todo entry
impl ORM {
    pub async fn insert_task(&mut self, task: Task) -> Result<(), ORMError> {
        let p = task.priority as i64;

        let record= sqlx::query!(
            r#"
                INSERT INTO Tasks (pub_id, group_id, name, priority, description, due)
                VALUES (
                    ?,
                    (SELECT pub_id FROM Groups WHERE name = ?),
                    ?,
                    ?,
                    ?,
                    ?
                )
                RETURNING Tasks.pub_id, Tasks.name, Tasks.priority as "priority: i64", Tasks.description, Tasks.due, Tasks.group_id, Tasks.completed
            "#,
            *task.id,
            *task.group,
            *task.name,
            p,
            task.description,
            task.due
        )
        .fetch_one(&mut self.conn)
        .await?;

        let group_name =
            sqlx::query_scalar!("SELECT name FROM Groups WHERE id = $1", record.group_id)
                .fetch_one(&mut self.conn)
                .await?;

        let created_task = Task::with_all_fields(
            record.pub_id.try_into()?,
            record.name.as_str().try_into()?,
            group_name.as_str().try_into()?,
            record.priority.try_into()?,
            record.description,
            record.completed,
            record.due,
        );

        assert_eq!(task, created_task);

        Ok(())
    }

    pub async fn insert_group(&mut self, group: Group) -> Result<(), ORMError> {
        let id = Id::default();
        let record = sqlx::query_scalar!(
            r#"
                INSERT INTO Groups (pub_id, name)
                VALUES (
                    ?,
                    ?
                )
                RETURNING Groups.name as "0"
            "#,
            *id,
            *group
        )
        .fetch_one(&mut self.conn)
        .await?;

        todo!()
    }
}
