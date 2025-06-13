use common::TarsClient;
use utils::new_test_daemon;

mod utils;

#[tokio::test]
async fn task_tests() {
    // let (d, addr) = new_test_daemon().await;
    // let _x = tokio::spawn(async move {
    //     d.run().await;
    // });

    // // now we can start using the client to test?
    // let _client = TarsClient::new(addr)
    //     .await
    //     .expect("failed to instantiate client");

    //TODO: finish up the client implementation so we can actually write these tests
}
