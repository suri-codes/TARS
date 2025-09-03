use async_recursion::async_recursion;
use axum::{
    Json, Router, debug_handler,
    extract::{Path, State},
    routing::get,
};
use common::{
    TarsError,
    types::{Id, Priority},
};
use sqlx::{Pool, Sqlite};
use tracing::instrument;

use crate::DaemonState;

pub fn score_router() -> Router<DaemonState> {
    Router::new()
        .route("/task/:id", get(calculate_task_score))
        .route("/group/:id", get(calculate_group_score))
}

#[instrument(skip(state))]
#[debug_handler]
pub async fn calculate_task_score(
    State(state): State<DaemonState>,
    Path(id): Path<Id>,
) -> Result<Json<f64>, TarsError> {
    let task = sqlx::query!(
        r#"
        SELECT
        group_id as "group_id: Id",
        priority as "priority: Priority"
        FROM Tasks
        WHERE pub_id = ?
    "#,
        *id
    )
    .fetch_one(&state.pool)
    .await?;

    let task_p_score = 1.0 / task.priority as i32 as f64;
    Ok(Json::from(
        calculate_group_p_score(&task.group_id, &state.pool).await? * task_p_score,
    ))
}

#[instrument(skip(state))]
#[debug_handler]
pub async fn calculate_group_score(
    State(state): State<DaemonState>,
    Path(id): Path<Id>,
) -> Result<Json<f64>, TarsError> {
    Ok(Json::from(calculate_group_p_score(&id, &state.pool).await?))
}

#[async_recursion]
async fn calculate_group_p_score(group_id: &Id, pool: &Pool<Sqlite>) -> Result<f64, TarsError> {
    let group = sqlx::query!(
        r#"

            SELECT
            pub_id as "id: Id",
            parent_id as "parent_id: Id",
            priority as "priority: Priority"
            FROM Groups
            WHERE pub_id = ?
        "#,
        *group_id
    )
    .fetch_one(pool)
    .await?;

    let current_p_score = 1.0 / group.priority as i32 as f64;

    if group.parent_id.is_none() {
        // we are the root
        return Ok(current_p_score);
    }

    return Ok(calculate_group_p_score(&group.parent_id.unwrap(), pool).await? * current_p_score);
}
