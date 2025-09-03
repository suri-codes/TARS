use crate::DaemonState;
use axum::{
    Json, Router, debug_handler,
    extract::State,
    routing::{get, post},
};
use common::{Diff, DiffInner, TarsError, types::*};

use tracing::{info, instrument};

/// Returns a router with all the group specific endpoints
pub fn group_router() -> Router<DaemonState> {
    Router::new()
        .route("/", get(fetch_groups))
        .route("/create", post(create_group))
        .route("/update", post(update_group))
        .route("/delete", post(delete_group))
}

/// Takes in a `Group` and then writes that group to the database.
///
/// # Errors
/// TarsError
///
/// This function will return an error if
/// + Something goes wrong with sqlx.
/// + Something goes wrong turning what sqlx returns into our wrapper types.
#[instrument(skip(state))]
#[debug_handler]
async fn create_group(
    State(state): State<DaemonState>,
    Json(group): Json<Group>,
) -> Result<Json<Group>, TarsError> {
    // we want to recursively attach all of the parent groups
    // let new_id = Id::default();
    let inserted = sqlx::query_as!(
        Group,
        r#"
            INSERT INTO Groups (pub_id, name, parent_id, color, priority)
            VALUES (
                ?,
                ?,
                ?,
                ?,
                ?
            )
            RETURNING Groups.name as "name: Name", Groups.pub_id as "id: Id", Groups.parent_id as "parent_id: Id", Groups.color as "color: Color", Groups.priority as "priority: Priority"
        "#,
        *group.id,
        *group.name,
        group.parent_id,
        group.color,
        group.priority
    )
    .fetch_one(&state.pool)
    .await?;

    assert_eq!(group, inserted);

    info!("Created group: {:#?}", inserted);

    let _ = state
        .diff_tx
        .send(Diff::Added(DiffInner::Group(inserted.clone())));

    Ok(Json(inserted))
}

/// Fetches all groups from the database.
///
/// # Errors
/// TarsError
///
/// This function will return an error if
/// + Something goes wrong with sqlx.
/// + Something goes wrong turning what sqlx returns into our wrapper types.
#[allow(unused)]
#[instrument(skip(state))]
#[debug_handler]
async fn fetch_groups(State(state): State<DaemonState>) -> Result<Json<Vec<Group>>, TarsError> {
    let groups = sqlx::query_as!(
        Group,
        r#"
        SELECT
        pub_id as "id: Id",
        name as "name: Name",
        parent_id as "parent_id: Id",
        color as "color: Color",
        priority as "priority: Priority"
        FROM Groups
        "#
    )
    .fetch_all(&state.pool)
    .await?;

    info!("Fetched groups: {:#?}", groups);

    Ok(Json::from(groups))
}

/// Takes in a `Group` and then updates that group to the database.
///
/// # Errors
/// TarsError
///
/// This function will return an error if
/// + Something goes wrong with sqlx.
/// + Something goes wrong turning what sqlx returns into our wrapper types.
#[instrument(skip(state))]
#[debug_handler]
async fn update_group(
    State(state): State<DaemonState>,
    Json(group): Json<Group>,
) -> Result<Json<Group>, TarsError> {
    let col = group.color.as_str();

    let updated = sqlx::query_as!(
        Group,
        r#"
            UPDATE Groups
            SET
            name = ?,
            color = ?,
            priority = ?
            WHERE pub_id = ?
            RETURNING
                name as "name: Name",
                pub_id as "id: Id",
                parent_id as "parent_id: Id",
                color as "color: Color",
                priority as "priority: Priority"

        "#,
        *group.name,
        col,
        group.priority,
        *group.id
    )
    .fetch_one(&state.pool)
    .await?;

    assert_eq!(group, updated);
    info!("Updated group: {:#?}", updated);

    let _ = state
        .diff_tx
        .send(Diff::Updated(DiffInner::Group(updated.clone())));
    Ok(Json::from(updated))
}

/// Takes in a `Group` and then deletes that group in the database.
///
/// # Errors
/// TarsError
///
/// This function will return an error if
/// + Something goes wrong with sqlx.
/// + Something goes wrong turning what sqlx returns into our wrapper types.
#[instrument(skip(state))]
#[debug_handler]
async fn delete_group(
    State(state): State<DaemonState>,
    Json(group): Json<Group>,
) -> Result<Json<Group>, TarsError> {
    let deleted = sqlx::query_as!(
        Group,
        r#"
            DELETE FROM Groups
            WHERE pub_id = ?
            RETURNING
                pub_id as "id: Id",
                name as "name: Name",
                parent_id as "parent_id: Id",
                color as "color: Color",
                priority as "priority: Priority"
           
        "#,
        *group.id,
    )
    .fetch_one(&state.pool)
    .await?;

    info!("Deleted group: {:#?}", deleted);
    assert_eq!(group, deleted);

    let _ = state.diff_tx.send(Diff::Deleted(deleted.id.clone()));
    Ok(Json::from(deleted))
}
