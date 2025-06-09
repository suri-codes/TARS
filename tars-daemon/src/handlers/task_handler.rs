use axum::{Extension, Json, Router, extract::State, http::StatusCode, routing::post};
use common::types::{Id, Task, TaskFetchOptions, UpdateTask};
use sqlx::{Pool, Sqlite};

pub fn add_task_handlers(router: Router) -> Router {
    router
        .route("/task/create", post(create_task))
        .route("/task/search", post(fetch_task))
        .route("/task/update", post(update_task))
        .route("/task/delete", post(delete_task))
}

async fn create_task(
    Extension(pool): Extension<Pool<Sqlite>>,
    Json(payload): Json<Task>,
) -> (StatusCode, Json<Task>) {
    // async fn add_data(
    //     State(pool): State<Pool<Sqlite>>,
    //     Json(payload): Json<AddData>,
    // ) -> (StatusCode, Json<Data>) {
    todo!()
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
