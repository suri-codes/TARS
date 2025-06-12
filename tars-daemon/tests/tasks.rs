use common::TarsClient;
use utils::new_test_daemon;

mod utils;

#[tokio::test]
async fn task_tests() {
    let (d, addr) = new_test_daemon().await;
    let d_addr = addr.clone();
    let _x = tokio::spawn(async move {
        d.run(d_addr.clone().as_str()).await;
    });

    // now we can start using the client to test?
    let _client = TarsClient::new(addr)
        .await
        .expect("failed to instantiate client");

    //TODO: finish up the client implementation so we can actually write these tests
}
