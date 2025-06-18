use std::time::Duration;

use common::{TarsClient, types::Group};
use tokio::time::{sleep, timeout};
use utils::new_test_daemon;

mod utils;

#[tokio::test]
async fn group_creation() {
    let (d, addr) = new_test_daemon().await;

    let _x = tokio::spawn(async move {
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

    let _child_group = Group::new(&client, "child", Some(group.id)).await.unwrap();

    // let created = vec![group, child_group].sort();

    // let groups = Group::fetch_all(&client).await.unwrap().sort();
}
