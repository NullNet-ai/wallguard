pub mod auth;
pub mod devices;
pub mod failures;
pub mod http_services;
pub mod sse;
pub mod static_files;
pub mod tunnels;
pub mod users;
pub mod ws;

use axum::{
    middleware as axum_middleware,
    routing::{delete, get, post},
    Router,
};

use crate::{middleware::auth::auth_middleware, AppState};

pub fn build_router(state: AppState) -> Router<AppState> {
    let protected = Router::new()
        .route("/api/v1/devices",                                    get(devices::list))
        .route("/api/v1/devices/:id",                               get(devices::get_one))
        .route("/api/v1/devices/:id/status",                        get(devices::status))
        .route("/api/v1/devices/:id/http-services",                 get(http_services::list))
        .route("/api/v1/failures",                                   get(failures::list_failures_org))
        .route("/api/v1/devices/:id/failures",                      get(failures::list_failures))
        .route("/api/v1/devices/:id/tunnels/ssh",                   post(tunnels::open_ssh))
        .route("/api/v1/devices/:id/tunnels/tty",                   post(tunnels::open_tty))
        .route("/api/v1/devices/:id/tunnels/http",                  post(tunnels::open_http))
        .route("/api/v1/devices/:id/tunnels/ssh/:session_id",       get(ws::ssh))
        .route("/api/v1/devices/:id/tunnels/tty/:session_id",       get(ws::tty))
        .route("/api/v1/users",                                     get(users::list_users).post(users::create_user))
        .route("/api/v1/users/:id",                                 delete(users::delete_user))
        .route("/api/v1/events",                                    get(sse::handler))
        .route("/api/v1/auth/logout",                               post(auth::logout))
        .route_layer(axum_middleware::from_fn_with_state(state, auth_middleware));

    let public = Router::new()
        .route("/api/v1/auth/login",   post(auth::login))
        .route("/api/v1/auth/refresh", post(auth::refresh_token));

    protected
        .merge(public)
        .fallback(static_files::handler)
}
