use tars_daemon::{Db, TarsDaemon};

pub async fn new_test_daemon() -> (TarsDaemon, String) {
    let db = Db::new(true).await;
    let daemon = TarsDaemon::init(db).await;

    let availible_port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();

    (daemon, format!("127.0.0.1:{}", availible_port))
}
