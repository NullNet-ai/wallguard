use std::convert::Infallible;

use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    Extension,
};
use tokio_stream::{wrappers::BroadcastStream, StreamExt as _};

use crate::{events::SseEvent, middleware::auth::RequestContext, AppState};

pub async fn handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let stream = BroadcastStream::new(state.sse_tx.subscribe())
        .filter_map(move |result| {
            let event: SseEvent = result.ok()?;
            if event.org_id != ctx.org_id {
                return None;
            }
            let kind_name = match &event.kind {
                crate::events::SseEventKind::DeviceConnected { .. }     => "device_connected",
                crate::events::SseEventKind::DeviceDisconnected { .. }  => "device_disconnected",
                crate::events::SseEventKind::NewFailure { .. }          => "new_failure",
                crate::events::SseEventKind::HttpServicesUpdated { .. } => "http_services_updated",
            };
            let data = serde_json::to_string(&event.kind).unwrap_or_default();
            Some(Ok::<_, Infallible>(Event::default().event(kind_name).data(data)))
        });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
