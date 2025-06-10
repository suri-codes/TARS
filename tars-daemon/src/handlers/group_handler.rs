use axum::{Extension, Json, Router, routing::post};
use common::{
    TarsError,
    types::{Group, Id, Name},
};
use sqlx::{Pool, Sqlite};

pub fn add_group_handlers(router: Router) -> Router {
    router
        .route("/group/create", post(create_group))
        .route("/group/search", post(fetch_groups))
        .route("/group/update", post(update_group))
        .route("/group/delete", post(delete_group))
}

async fn create_group(
    Extension(pool): Extension<Pool<Sqlite>>,
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
async fn fetch_groups(
    Extension(pool): Extension<Pool<Sqlite>>,
    // Json(group): Json<Group>,
) -> Result<Json<Vec<Group>>, TarsError> {
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
async fn update_group(
    Extension(pool): Extension<Pool<Sqlite>>,
    Json(group): Json<Group>,
) -> Result<Json<Group>, TarsError> {
    // uh, how do we do this lmao, i think groups need to have an id no?
    todo!()
}
async fn delete_group(
    Extension(pool): Extension<Pool<Sqlite>>,
    Json(group): Json<Group>,
) -> Result<Json<Group>, TarsError> {
    todo!()
}
