use crate::DaemonState;
use axum::{Json, Router, debug_handler, extract::State, routing::post};
use common::{
    TarsError,
    types::{Group, Id, Name},
};
use tracing::{info, instrument};

/// Returns a router with all the group specific endpoints
pub fn group_router() -> Router<DaemonState> {
    Router::new()
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
            INSERT INTO Groups (pub_id, name, parent_id)
            VALUES (
                ?,
                ?,
                ?
            )
            RETURNING Groups.name as "name: Name", Groups.pub_id as "id: Id", Groups.parent_id as "parent_id: Option<Id>"
        "#,
        *group.id,
        *group.name,
        group.parent_id
    )
    .fetch_one(&state.pool)
    .await?;

    assert_eq!(group, inserted);

    info!("Created group: {:#?}", inserted);
    Ok(Json(group))
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
        parent_id as "parent_id: Option<Id>"
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
    let updated = sqlx::query_as!(
        Group,
        r#"
            UPDATE Groups
            SET
            name = ?
            WHERE pub_id = ?
            RETURNING
                name as "name: Name",
                pub_id as "id: Id",
                parent_id as "parent_id: Option<Id>"

        "#,
        *group.name,
        *group.id
    )
    .fetch_one(&state.pool)
    .await?;

    assert_eq!(group, updated);
    info!("Updated group: {:#?}", updated);

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
                parent_id as "parent_id: Option<Id>"
           
        "#,
        *group.id,
    )
    .fetch_one(&state.pool)
    .await?;

    info!("Deleted group: {:#?}", deleted);
    assert_eq!(group, deleted);

    Ok(Json::from(deleted))
}
