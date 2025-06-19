use std::time::Duration;

use common::{
    TarsClient,
    types::{Group, Priority, Task, TaskFetchOptions},
};
use tars_daemon::utils::new_test_daemon;
use tokio::time::{sleep, timeout};

#[tokio::test]
async fn task_creation() {
    let (d, addr) = new_test_daemon().await;

    let x = tokio::spawn(async move {
        timeout(Duration::from_secs(2), d.run())
            .await
            .unwrap_or_else(|_x| Ok(()))
            .unwrap();
    });

    sleep(Duration::from_secs(1)).await;

    let client = TarsClient::new(addr)
        .await
        .expect("failed to instantiate client");

    let group = Group::new(&client, "testing", None).await.unwrap();

    let task = Task::new(&client, &group, "test", Priority::Low, "nothing", None)
        .await
        .unwrap();

    let tasks = Task::fetch(&client, TaskFetchOptions::All).await.unwrap();

    assert_eq!(task, *tasks.first().unwrap());

    x.await.unwrap()
}

#[tokio::test]
async fn task_sync() {
    let (d, addr) = new_test_daemon().await;

    let x = tokio::spawn(async move {
        timeout(Duration::from_secs(2), d.run())
            .await
            .unwrap_or_else(|_x| Ok(()))
            .unwrap();
    });

    sleep(Duration::from_secs(1)).await;

    let client = TarsClient::new(addr)
        .await
        .expect("failed to instantiate client");

    let group = Group::new(&client, "testing", None).await.unwrap();

    let task = Task::new(&client, &group, "test", Priority::Low, "nothing", None)
        .await
        .unwrap();

    let mut tasks = Task::fetch(&client, TaskFetchOptions::All).await.unwrap();

    let fetched_task = tasks.first_mut().unwrap();

    assert_eq!(task, *fetched_task);

    fetched_task.name = "dont matter".to_owned().into();

    fetched_task.sync(&client).await.unwrap();

    x.await.unwrap()
}

#[tokio::test]
async fn task_delete() {
    let (d, addr) = new_test_daemon().await;

    let x = tokio::spawn(async move {
        tracing_subscriber::fmt::init();
        timeout(Duration::from_secs(2), d.run())
            .await
            .unwrap_or_else(|_x| Ok(()))
            .unwrap();
    });

    sleep(Duration::from_secs(1)).await;

    let client = TarsClient::new(addr)
        .await
        .expect("failed to instantiate client");

    let group = Group::new(&client, "testing", None).await.unwrap();

    let task = Task::new(&client, &group, "test", Priority::Low, "nothing", None)
        .await
        .unwrap();

    let mut tasks = Task::fetch(&client, TaskFetchOptions::All).await.unwrap();

    // ensure it exists
    let fetched_task = tasks.first_mut().unwrap();

    assert_eq!(task, *fetched_task);

    fetched_task.clone().delete(&client).await.unwrap();

    let tasks = Task::fetch(&client, TaskFetchOptions::All).await.unwrap();
    assert!(tasks.is_empty());

    x.await.unwrap()
}
