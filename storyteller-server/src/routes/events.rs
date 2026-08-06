//! The change-event stream (`docs/api.md` §5).
//!
//! `GET /api/v1/events` is a Server-Sent Events stream: the [watcher](crate::watcher)
//! and the write handlers broadcast [`Event`](crate::events::Event)s, and every
//! subscriber receives them as `event:`/`data:` frames. A client that lags past
//! the broadcast buffer simply misses frames and recovers by reloading — the
//! index stays the single queryable truth, so no event is a write path.

use std::convert::Infallible;

use axum::extract::State;
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};

use crate::state::SharedState;

/// `GET /api/v1/events` — subscribe to the SSE change stream.
pub async fn stream(
    State(state): State<SharedState>,
) -> Sse<impl Stream<Item = Result<SseEvent, Infallible>>> {
    let events = BroadcastStream::new(state.subscribe()).filter_map(|result| {
        // A `Lagged` error means this subscriber fell behind; drop the marker and
        // keep streaming — the client reloads to catch up.
        let event = result.ok()?;
        SseEvent::default()
            .event(event.name())
            .json_data(event.data())
            .ok()
            .map(Ok)
    });
    Sse::new(events).keep_alive(KeepAlive::default())
}
