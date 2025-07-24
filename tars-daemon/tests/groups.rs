use std::time::Duration;

use common::{TarsClient, types::Group};
use tars_daemon::utils::new_test_daemon;
use tokio::time::{sleep, timeout};

#[tokio::test]
async fn group_creation() {
    let (d, addr) = new_test_daemon().await;

    let x = tokio::spawn(async move {
        timeout(Duration::from_secs(3), d.run())
            .await
            .unwrap_or_else(|_x| Ok(()))
            .unwrap();
    });

    sleep(Duration::from_secs(1)).await;
    let client = TarsClient::new(addr)
        .await
        .expect("failed to instantiate client");

    let group = Group::new(&client, "testing", None, Default::default())
        .await
        .unwrap();

    let child_group = Group::new(&client, "child", Some(group.id.clone()), Default::default())
        .await
        .unwrap();

    let mut created = vec![group, child_group];
    created.sort();

    let mut fetched = Group::fetch_all(&client).await.unwrap();
    fetched.sort();

    assert_eq!(created, fetched);
    x.await.unwrap()
}

#[tokio::test]
async fn group_sync() {
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

    let mut group = Group::new(&client, "testing", None, Default::default())
        .await
        .unwrap();
    group.name = "testing_2".to_owned().into();

    group.sync(&client).await.unwrap();

    x.await.unwrap()
}

#[tokio::test]
async fn group_delete() {
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

    let group = Group::new(&client, "testing", None, Default::default())
        .await
        .unwrap();

    group.delete(&client).await.unwrap();

    let fetched = Group::fetch_all(&client).await.unwrap();

    assert!(fetched.is_empty());

    x.await.unwrap();
}
