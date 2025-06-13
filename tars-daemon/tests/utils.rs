use tars_daemon::{DaemonState, Db, TarsDaemon};

#[expect(dead_code)]
pub async fn new_test_daemon() -> (TarsDaemon, String) {
    let db = Db::new(true).await;
    let availible_port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();

    let port_str = format!("127.0.0.1:{}", availible_port);

    let state = DaemonState::new(db, &port_str);

    let daemon = TarsDaemon::init(state).await;

    (daemon, format!("127.0.0.1:{}", availible_port))
}
