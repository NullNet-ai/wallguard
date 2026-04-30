use axum::{
    extract::{Extension, Path, State},
    Json,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{error::AppError, middleware::auth::RequestContext, AppState};

#[derive(Serialize)]
pub struct HttpServiceRow {
    pub port:   u32,
    pub scheme: String,
    pub title:  String,
}

pub async fn list(
    State(state):    State<AppState>,
    Extension(_ctx): Extension<RequestContext>,
    Path(device_id): Path<Uuid>,
) -> Result<Json<Vec<HttpServiceRow>>, AppError> {
    let services = state.registry.get_http_services(&device_id).await;
    let rows = services
        .into_iter()
        .map(|s| HttpServiceRow { port: s.port, scheme: s.scheme, title: s.title })
        .collect();
    Ok(Json(rows))
}
