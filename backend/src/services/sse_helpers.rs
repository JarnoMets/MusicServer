// Shared SSE (Server-Sent Events) response builder.
//
// Multiple route handlers create SSE streams with the same boilerplate.
// This module provides a reusable helper.

use actix_web::{web, HttpResponse};
use serde::Serialize;
use tokio::sync::broadcast;

/// Create an SSE HttpResponse from a broadcast receiver.
///
/// - `rx`: The broadcast receiver to read events from.
/// - `is_done`: A predicate that returns `true` when the stream should close.
pub fn sse_response<T>(
    mut rx: broadcast::Receiver<T>,
    is_done: impl Fn(&T) -> bool + Send + 'static,
) -> HttpResponse
where
    T: Serialize + Clone + Send + 'static,
{
    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let data = serde_json::to_string(&event).unwrap_or_default();
                    yield Ok::<_, actix_web::Error>(web::Bytes::from(format!("data: {}\n\n", data)));
                    if is_done(&event) {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Closed) => break,
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
            }
        }
    };

    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/event-stream"))
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("Connection", "keep-alive"))
        .streaming(stream)
}
