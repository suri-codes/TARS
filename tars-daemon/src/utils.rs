use crate::{DaemonState, Db, TarsDaemon};

/// Returns a new `TarsDaemon`, with a temporary DB and a open port, perfect for testing.
/// Ensure you use the returned String as the url to communicate with the daemon.
pub async fn new_test_daemon() -> (TarsDaemon, String) {
    let db = Db::new(true).await.unwrap();
    let availible_port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    // use common::logging;
    // logging::init(format!("tars-d-{availible_port}.log").as_str(), true).unwrap();

    let port_str = format!("127.0.0.1:{availible_port}");

    let state = DaemonState::new(db, &port_str);

    let daemon = TarsDaemon::init(state).await;

    (daemon, format!("http://127.0.0.1:{availible_port}"))
}
