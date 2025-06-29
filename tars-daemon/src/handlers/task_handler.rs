use async_recursion::async_recursion;
use axum::{Json, Router, debug_handler, extract::State, routing::post};

use common::{
    TarsError,
    types::{Color, Group, Id, Name, Priority, Task, TaskFetchOptions},
};
use criterion::async_executor;
use sqlx::{Pool, Sqlite};
use tracing::{error, info, instrument};

use crate::DaemonState;

/// Returns a router with all the task specific endpoints
pub fn task_router() -> Router<DaemonState> {
    Router::new()
        .route("/create", post(create_task))
        .route("/fetch", post(fetch_task))
        .route("/update", post(update_task))
        .route("/delete", post(delete_task))
}

/// Takes in a task and then writes that task to the database.
///
/// # Errors
/// TarsError
///
/// This function will return an error if
/// + Something goes wrong with sqlx.
/// + Something goes wrong turning what sqlx returns into our wrapper types.
#[instrument(skip(state))]
#[debug_handler]
pub async fn create_task(
    State(state): State<DaemonState>,
    Json(task): Json<Task>,
) -> Result<Json<Task>, TarsError> {
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
            RETURNING Tasks.pub_id, Tasks.name, Tasks.priority as "priority: Priority", Tasks.description, Tasks.due, Tasks.group_id, Tasks.completed
            
        "#,
        *task.id,
        *task.group.id,
        *task.name,
        task.priority,
        task.description,
        task.due
    )
    .fetch_one(&state.pool)
    .await.inspect_err(|e|error!("{:?}", e))?;

    let group = sqlx::query_as!(
        Group,
        r#"
        SELECT name as "name: Name", pub_id as "id: Id", parent_id as "parent_id: Id", color as "color: Color" FROM Groups WHERE pub_id = ?
        "#,
        inserted.group_id
    )
    .fetch_one(&state.pool)
    .await.inspect_err(|e|error!("{:?}", e))?;

    let created_task = Task::with_all_fields(
        inserted.pub_id,
        group,
        inserted.name,
        inserted.priority,
        inserted.description,
        inserted.completed,
        inserted.due,
    );

    assert_eq!(task, created_task);
    info!("Created task: {:#?}", created_task);

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
#[instrument(skip(state))]
#[debug_handler]
async fn fetch_task(
    State(state): State<DaemonState>,
    Json(task_fetch_opts): Json<TaskFetchOptions>,
) -> Result<Json<Vec<Task>>, TarsError> {
    match task_fetch_opts {
        TaskFetchOptions::All => {
            let records = sqlx::query!(
                r#"
                    SELECT
                        t.pub_id as task_pub_id,
                        t.name as task_name,
                        g.name  as group_name,
                        g.pub_id as group_pub_id ,
                        g.parent_id as "group_parent_id: Id",
                        g.color as "group_color: Color",
                        t.priority as "priority: Priority",
                        t.description,
                        t.completed,
                        t.due
                    FROM Tasks t
                    JOIN Groups g ON t.group_id = g.pub_id
                        
                "#,
            )
            .fetch_all(&state.pool)
            .await?;

            let tasks: Vec<Task> = records
                .into_iter()
                .map(|row| {
                    Task::with_all_fields(
                        row.task_pub_id,
                        Group::with_all_fields(
                            row.group_pub_id,
                            row.group_name,
                            row.group_parent_id,
                            row.group_color,
                        ),
                        row.task_name,
                        row.priority,
                        row.description,
                        row.completed,
                        row.due,
                    )
                })
                .collect();
            info!("Fetched tasks: {:#?}", &tasks);

            Ok(Json::from(tasks))
        }
        TaskFetchOptions::ByGroup { group } => {
            let mut tasks: Vec<Task> = Vec::new();

            recurse_group_fetch(&mut tasks, group, &state.pool).await?;

            info!("Fetched tasks: {:#?}", &state.pool);

            Ok(Json::from(tasks))
        }
    }
}

#[async_recursion]
async fn recurse_group_fetch(
    tasks: &mut Vec<Task>,
    group: Group,
    pool: &Pool<Sqlite>,
) -> Result<(), TarsError> {
    // first we add the tasks pertinent to the passed in group

    let records = sqlx::query!(
        r#"
                    SELECT
                        t.pub_id as task_pub_id,
                        t.name as task_name,
                        g.name  as group_name,
                        g.pub_id as group_pub_id ,
                        g.parent_id as "group_parent_id: Id",
                        g.color as "group_color: Color",
                        t.priority as "priority: Priority",
                        t.description,
                        t.completed,
                        t.due
                    FROM Tasks t
                    JOIN Groups g ON t.group_id = g.pub_id
                    WHERE g.pub_id = ?
                        
                "#,
        group.id
    )
    .fetch_all(pool)
    .await?;

    for row in records {
        let task = Task::with_all_fields(
            row.task_pub_id,
            Group::with_all_fields(
                row.group_pub_id,
                row.group_name,
                row.group_parent_id,
                row.group_color,
            ),
            row.task_name,
            row.priority,
            row.description,
            row.completed,
            row.due,
        );

        tasks.push(task)
    }

    // now lets look at children groups
    let children = sqlx::query_as!(
        Group,
        r#"
        SELECT pub_id as "id: Id", name as "name: Name", color as "color: Color" , parent_id as "parent_id: Id"
        FROM Groups
        WHERE parent_id = ?
        "#,
        group.id
    ).fetch_all(pool).await?;

    for child in children {
        recurse_group_fetch(tasks, child, pool).await?;
    }

    Ok(())

    //
    //
    //
    // then we take whatever groups are the children of this one, and add those.
}
/// Takes in a task, uses the id to find the old one and updates it with the new information.
///
/// # Errors
/// TarsError
///
/// This function will return an error if
/// + Something goes wrong with sqlx.
/// + Something goes wrong turning what sqlx returns into our wrapper types.
#[instrument(skip(state))]
#[debug_handler]
async fn update_task(
    State(state): State<DaemonState>,
    Json(task): Json<Task>,
) -> Result<Json<Task>, TarsError> {
    let row = sqlx::query!(
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
            pub_id as task_pub_id,
            name as task_name,
            group_id,
            (SELECT g.name FROM Groups g WHERE g.pub_id = Tasks.group_id) as group_name,
            (SELECT g.parent_id FROM Groups g WHERE g.pub_id = Tasks.group_id) as "group_parent_id: Id",
            (SELECT g.color FROM Groups g WHERE g.pub_id = Tasks.group_id) as "group_color: Color",
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
        *task.group.id,
        *task.id
    )
    .fetch_one(&state.pool)
    .await?;

    let updated_task = Task::with_all_fields(
        row.task_pub_id,
        Group::with_all_fields(
            row.group_id,
            row.group_name,
            row.group_parent_id,
            row.group_color,
        ),
        row.task_name,
        row.priority,
        row.description,
        row.completed,
        row.due,
    );

    // if they dont match, we have a problem!
    assert_eq!(updated_task, task);

    info!("Updated task: {:#?}", updated_task);
    Ok(Json::from(updated_task))
}

/// Takes in a task `ID`, deletes it, and returns the deleted task.
///
/// # Errors
/// TarsError
///
/// This function will return an error if
/// + Something goes wrong with sqlx.
/// + Something goes wrong turning what sqlx returns into our wrapper types.
#[instrument(skip(state))]
#[debug_handler]
async fn delete_task(
    State(state): State<DaemonState>,
    Json(deletion_id): Json<Id>,
) -> Result<Json<Task>, TarsError> {
    let mut tx = state.pool.begin().await?;
    let row = sqlx::query!(
        r#"
            SELECT
                t.pub_id as task_id,
                t.name as task_name,
                g.name as group_name,
                g.parent_id as "group_parent_id: Id",
                g.color as "group_color: Color",

                t.group_id,
                t.priority as "priority: Priority",
                t.description,
                t.completed,
                t.due
                FROM Tasks t
                JOIN Groups g ON t.group_id = g.pub_id
                WHERE t.pub_id = ?

        "#,
        *deletion_id
    )
    .fetch_one(&mut *tx)
    .await?;

    let deleted_task = Task::with_all_fields(
        row.task_id,
        Group::with_all_fields(
            row.group_id,
            row.group_name,
            row.group_parent_id,
            row.group_color,
        ),
        row.task_name,
        row.priority,
        row.description,
        row.completed,
        row.due,
    );

    sqlx::query!("DELETE FROM Tasks WHERE pub_id = ?", *deletion_id)
        .execute(&mut *tx)
        .await?;

    assert_eq!(deletion_id, deleted_task.id);

    tx.commit().await?;
    info!("Deleted task: {:#?}", deleted_task);

    Ok(Json::from(deleted_task))
}
