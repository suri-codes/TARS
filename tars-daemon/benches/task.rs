use std::{process::Command, thread, time::Duration};

use common::{
    TarsClient,
    types::{Group, Priority, Task},
};
use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use tars_daemon::utils::new_test_daemon;
use tokio::time::timeout;

// This is a struct that tells Criterion.rs to use the "futures" crate's current-thread executor
use tokio::runtime::Runtime;

// Here we have an async function to benchmark
async fn task_creation(client: &TarsClient, group: &Group) {
    let _x = Task::new(
        client,
        group,
        "bench",
        Priority::Asap,
        "better go fast",
        None,
    )
    .await
    .unwrap();
}

async fn task_sync(client: &TarsClient, task: Task) {
    task.sync(client).await.unwrap();
}

async fn task_delete(client: &TarsClient, task: Task) {
    task.delete(client).await.unwrap();
}

fn bench_tasks(c: &mut Criterion) {
    let b_rt = Runtime::new().unwrap();
    let (d, addr) = b_rt.block_on(new_test_daemon());

    let _daemon_handle = thread::spawn(move || {
        let d_rt = Runtime::new().unwrap();
        tracing_subscriber::fmt::init();
        let _ = d_rt.block_on(async { timeout(Duration::from_secs(40), d.run()).await });

        // d_rt.block_on(d.run()).unwrap();
    });

    // need to sleep so daemon can get up to speed
    thread::sleep(Duration::from_secs(1));

    let client = b_rt.block_on(TarsClient::new(addr)).unwrap();

    let group = b_rt.block_on(Group::new(&client, "bench", None)).unwrap();

    c.bench_function("task creation", |b| {
        let rt = Runtime::new().unwrap();
        b.to_async(rt).iter(|| task_creation(&client, &group));
    });

    let task = b_rt
        .block_on(Task::new(
            &client,
            &group,
            "bench",
            Priority::Asap,
            "better go fast",
            None,
        ))
        .unwrap();

    c.bench_function("task sync", |b| {
        let rt = Runtime::new().unwrap();
        b.to_async(rt).iter(|| task_sync(&client, task.clone()));
    });

    c.bench_function("task creation + delete", |b| {
        let rt = Runtime::new().unwrap();
        b.to_async(rt).iter(|| async {
            let x = Task::new(
                &client,
                &group,
                "bench",
                Priority::Asap,
                "better go fast",
                None,
            )
            .await
            .unwrap();

            task_delete(&client, x).await
        });
    });
}

criterion_group!(benches, bench_tasks);
criterion_main!(benches);
