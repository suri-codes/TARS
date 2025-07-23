use crate::DaemonState;

use axum::{
    Router,
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    routing::get,
};
use futures_util::stream::Stream;
use std::convert::Infallible;
use tokio_stream::{StreamExt as _, wrappers::BroadcastStream};

pub fn subscribe_router() -> Router<DaemonState> {
    Router::new().route("/", get(diff_handler))
}

async fn diff_handler(
    State(state): State<DaemonState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // A `Stream` that repeats an event every second
    let rx = state.diff_tx.subscribe();

    let stream = BroadcastStream::new(rx)
        .map(|res| {
            match res {
                Ok(data) => {
                    // Convert your data to JSON or whatever format you need
                    let json = serde_json::to_string(&data).unwrap_or_default();

                    Event::default().data(json)
                }
                Err(_) => {
                    // Handle broadcast errors (like lagged receiver)
                    Event::default().data("error")
                }
            }
        })
        .map(Ok);

    Sse::new(stream).keep_alive(KeepAlive::default())
}
