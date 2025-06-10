use axum::{Extension, Json, Router, routing::post};
use common::{
    TarsError,
    types::{Group, Id, Name, Priority, Task, TaskFetchOptions},
};
use sqlx::{Pool, Sqlite};

/// Takes in a router and appends all the handlers related to tasks.
pub fn add_task_handlers(router: Router) -> Router {
    router
        .route("/task/create", post(create_task))
        .route("/task/search", post(fetch_task))
        .route("/task/update", post(update_task))
        .route("/task/delete", post(delete_task))
}

/// Takes in a task and then writes that task to the database.
///
/// # Errors
/// TarsError
///
/// This function will return an error if
/// + Something goes wrong with sqlx.
/// + Something goes wrong turning what sqlx returns into our wrapper types.
async fn create_task(
    Extension(pool): Extension<Pool<Sqlite>>,
    Json(task): Json<Task>,
) -> Result<Json<Task>, TarsError> {
    let p = task.priority as i64;

    // first we want to check if there exists a group with the same one as
    // this task, otherwise we cant make it!
    let group_name = sqlx::query!(
        r#"
        SELECT pub_id FROM Groups WHERE name = ?
        "#,
        *task.group
    )
    .fetch_one(&pool)
    .await?
    .pub_id;

    let inserted = sqlx::query!(
        r#"
            INSERT INTO Tasks (pub_id, group_id, name, priority, description, due)
            VALUES (
                ?,
                ?,
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
    .fetch_one(&pool)
    .await?;

    let created_task = Task::with_all_fields(
        inserted.pub_id.into(),
        group_name.into(),
        inserted.name.into(),
        inserted.priority.try_into()?,
        inserted.description,
        inserted.completed,
        inserted.due,
    );

    assert_eq!(task, created_task);

    Ok(Json::from(created_task))
}

/// Takes in `TaskFetchOptions` and returns the requested Tasks.
///
/// # Errors
/// TarsError
///
/// This function will return an error if
/// + Something goes wrong with sqlx.
/// + Something goes wrong turning what sqlx returns into our wrapper types.
async fn fetch_task(
    Extension(pool): Extension<Pool<Sqlite>>,
    Json(payload): Json<TaskFetchOptions>,
) -> Result<Json<Vec<Task>>, TarsError> {
    match payload {
        TaskFetchOptions::All => {
            let all_tasks = sqlx::query_as!(
                Task,
                r#"
                    SELECT
                        t.pub_id as "id: Id",
                        t.name as "name: Name",
                        g.name as "group: Group",
                        t.priority as "priority: Priority",
                        t.description,
                        t.completed,
                        t.due
                    FROM Tasks t
                    JOIN Groups g ON t.group_id = g.pub_id
                        
                "#,
            )
            .fetch_all(&pool)
            .await?;

            Ok(Json::from(all_tasks))
        }
    }
}

/// Takes in a task, uses the id to find the old one and updates it with the new information.
///
/// # Errors
/// TarsError
///
/// This function will return an error if
/// + Something goes wrong with sqlx.
/// + Something goes wrong turning what sqlx returns into our wrapper types.
async fn update_task(
    Extension(pool): Extension<Pool<Sqlite>>,
    Json(task): Json<Task>,
) -> Result<Json<Task>, TarsError> {
    let group_pub_id = sqlx::query_scalar!("SELECT pub_id FROM Groups WHERE name = ?", *task.group)
        .fetch_one(&pool)
        .await?;

    let updated = sqlx::query_as!(
        Task,
        r#"
        UPDATE Tasks
        SET
            name = ?,
            priority = ?,
            description = ?,
            completed = ?,
            due = ?,
            group_id = ?
        WHERE pub_id = ?
        RETURNING 
            pub_id as "id: Id",
            name as "name: Name",
            (SELECT g.name FROM Groups g WHERE g.pub_id = Tasks.group_id) as "group: Group",
            priority as "priority: Priority",
            description,
            completed,
            due
        "#,
        *task.name,
        task.priority,
        task.description,
        task.completed,
        task.due,
        group_pub_id,
        *task.id
    )
    .fetch_one(&pool)
    .await?;

    // if they dont match, we have a problem!
    assert_eq!(updated, task);

    Ok(Json::from(updated))
}

/// Takes in a task `ID`, deletes it, and returns the deleted task.
///
/// # Errors
/// TarsError
///
/// This function will return an error if
/// + Something goes wrong with sqlx.
/// + Something goes wrong turning what sqlx returns into our wrapper types.
async fn delete_task(
    Extension(pool): Extension<Pool<Sqlite>>,
    Json(payload): Json<Id>,
) -> Result<Json<Task>, TarsError> {
    let mut tx = pool.begin().await?;
    let deleted_task = sqlx::query_as!(
        Task,
        r#"
            SELECT
                t.pub_id as "id: Id",
                t.name as "name: Name",
                g.name as "group: Group",
                t.priority as "priority: Priority",
                t.description,
                t.completed,
                t.due

                FROM Tasks t
                JOIN Groups g ON t.group_id = g.pub_id
                WHERE t.pub_id = ?

        "#,
        *payload
    )
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query!("DELETE FROM Tasks WHERE pub_id = ?", *payload)
        .execute(&mut *tx)
        .await?;

    assert_eq!(payload, deleted_task.id);

    tx.commit().await?;

    Ok(Json::from(deleted_task))
}
