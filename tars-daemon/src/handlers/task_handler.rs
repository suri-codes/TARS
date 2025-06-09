use axum::{Extension, Json, Router, extract::State, http::StatusCode, routing::post};
use common::{
    TarsError,
    types::{Id, Task, TaskFetchOptions, UpdateTask},
};
use sqlx::{Pool, Sqlite};
use tracing::error;

/// Takes in a router and appends all the handlers related to tasks.
pub fn add_task_handlers(router: Router) -> Router {
    router
        .route("/task/create", post(create_task))
        .route("/task/search", post(fetch_task))
        .route("/task/update", post(update_task))
        .route("/task/delete", post(delete_task))
}

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

async fn fetch_task(
    Extension(pool): Extension<Pool<Sqlite>>,
    Json(payload): Json<TaskFetchOptions>,
) -> (StatusCode, Json<Vec<Task>>) {
    // Implementation here
    todo!()
}

async fn update_task(
    Extension(pool): Extension<Pool<Sqlite>>,
    Json(payload): Json<UpdateTask>,
) -> (StatusCode, Json<Task>) {
    todo!()
}
async fn delete_task(
    Extension(pool): Extension<Pool<Sqlite>>,
    Json(payload): Json<Id>,
) -> (StatusCode, Json<Task>) {
    todo!()
}
