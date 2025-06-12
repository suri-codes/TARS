use axum::{
    Json,
    extract::State,
    routing::{get, post},
};
use common::{
    TarsError,
    types::{Group, Id, Name},
};
use sqlx::{Pool, Sqlite};

use crate::TarsRouter;

/// Takes in a router and appends all handlers related to groups
// pub fn add_group_handlers(router: TarsRouter) -> TarsRouter {
//     router
//         .route("/group", get(fetch_groups))
//         .route("/group/create", post(create_group))
//         .route("/group/update", post(update_group))
//         .route("/group/delete", post(delete_group))
// }

/// Takes in a `Group` and then writes that group to the database.
///
/// # Errors
/// TarsError
///
/// This function will return an error if
/// + Something goes wrong with sqlx.
/// + Something goes wrong turning what sqlx returns into our wrapper types.
async fn create_group(
    State(pool): State<Pool<Sqlite>>,
    Json(group): Json<Group>,
) -> Result<Json<Group>, TarsError> {
    // let new_id = Id::default();
    let inserted = sqlx::query_as!(
        Group,
        r#"
            INSERT INTO Groups (pub_id, name)
            VALUES (
                ?,
                ?
            )
            RETURNING Groups.name as "name: Name", Groups.pub_id as "id: Id" 
        "#,
        *group.id,
        *group.name,
    )
    .fetch_one(&pool)
    .await?;

    assert_eq!(group, inserted);

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
async fn fetch_groups(State(pool): State<Pool<Sqlite>>) -> Result<Json<Vec<Group>>, TarsError> {
    let groups = sqlx::query_as!(
        Group,
        r#"
        SELECT
        pub_id as "id: Id",
        name as "name: Name"
        FROM Groups
        "#
    )
    .fetch_all(&pool)
    .await?;

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
async fn update_group(
    State(pool): State<Pool<Sqlite>>,
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
                pub_id as "id: Id"

        "#,
        *group.name,
        *group.id
    )
    .fetch_one(&pool)
    .await?;

    assert_eq!(group, updated);

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
async fn delete_group(
    State(pool): State<Pool<Sqlite>>,
    Json(group): Json<Group>,
) -> Result<Json<Group>, TarsError> {
    let deleted = sqlx::query_as!(
        Group,
        r#"
            DELETE FROM Groups
            WHERE pub_id = ?
            RETURNING
                pub_id as "id: Id",
                name as "name: Name"
           
        "#,
        *group.id,
    )
    .fetch_one(&pool)
    .await?;

    assert_eq!(group, deleted);

    Ok(Json::from(deleted))
}
