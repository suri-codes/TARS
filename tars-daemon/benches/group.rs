use std::{thread, time::Duration};

use common::{TarsClient, types::Group};
use criterion::{Criterion, criterion_group, criterion_main};
use tars_daemon::utils::new_test_daemon;
use tokio::{runtime::Runtime, time::timeout};

async fn group_creation(client: &TarsClient) {
    let g = Group::new(client, "lol", None).await.unwrap();
}

async fn group_sync(client: &TarsClient, group: Group) {
    group.sync(client).await.unwrap();
}

async fn group_delete(client: &TarsClient, group: Group) {
    group.delete(client).await.unwrap();
}

fn bench_groups(c: &mut Criterion) {
    let b_rt = Runtime::new().unwrap();
    let (d, addr) = b_rt.block_on(new_test_daemon());
    let _ = thread::spawn(move || {
        let d_rt = Runtime::new().unwrap();
        tracing_subscriber::fmt::init();
        let _ = d_rt.block_on(async { timeout(Duration::from_secs(40), d.run()).await });
    });
    thread::sleep(Duration::from_secs(1));
    let client = b_rt.block_on(TarsClient::new(addr)).unwrap();

    c.bench_function("group creation", |b| {
        let rt = Runtime::new().unwrap();
        b.to_async(rt).iter(|| group_creation(&client));
    });

    let g = b_rt.block_on(Group::new(&client, "sync", None)).unwrap();

    c.bench_function("group sync", |b| {
        let rt = Runtime::new().unwrap();
        b.to_async(rt).iter(|| group_sync(&client, g.clone()));
    });

    c.bench_function("group creation + delete", |b| {
        let rt = Runtime::new().unwrap();
        b.to_async(rt).iter(|| async {
            let g = Group::new(&client, "delete", None).await.unwrap();
            group_delete(&client, g).await;
        });
    });
}
criterion_group!(benches, bench_groups);
criterion_main!(benches);
