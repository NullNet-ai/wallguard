use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map, use_query_map};
use uuid::Uuid;
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn DeviceTunnels() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let navigate = use_navigate();

    let device_id = move || {
        params.read().get("id")
            .and_then(|s| Uuid::parse_str(&s).ok())
    };

    // Read active session from query params (set by DeviceDetail when opening a tunnel).
    let ws_url = move || {
        let raw = query.read().get("ws").unwrap_or_default();
        if raw.is_empty() {
            return None;
        }
        let decoded = js_sys::decode_uri_component(&raw)
            .ok()
            .and_then(|js| js.as_string())
            .unwrap_or(raw);
        Some(decoded)
    };

    let session_id = move || {
        query.read().get("session")
            .filter(|s| !s.is_empty())
    };

    let tunnel_type = move || {
        query.read().get("type").unwrap_or_default()
    };

    let tunnel_error = RwSignal::new(Option::<String>::None);

    let open_ssh = {
        let navigate = navigate.clone();
        move |_| {
            let Some(id) = device_id() else { return };
            let nav = navigate.clone();
            tunnel_error.set(None);
            spawn_local(async move {
                match crate::api::tunnels::open_ssh(id).await {
                    Ok(resp) => {
                        let ws_url = match crate::auth::get_token() {
                            Some(t) => format!("{}?token={}", resp.ws_url, t),
                            None    => resp.ws_url.clone(),
                        };
                        nav(
                            &format!(
                                "/devices/{id}/tunnels?session={}&type=ssh&ws={}",
                                resp.session_id,
                                js_sys::encode_uri_component(&ws_url),
                            ),
                            Default::default(),
                        );
                    }
                    Err(e) => tunnel_error.set(Some(format!("SSH tunnel failed: {e}"))),
                }
            });
        }
    };

    let open_tty = {
        let navigate = navigate.clone();
        move |_| {
            let Some(id) = device_id() else { return };
            let nav = navigate.clone();
            tunnel_error.set(None);
            spawn_local(async move {
                match crate::api::tunnels::open_tty(id).await {
                    Ok(resp) => {
                        let ws_url = match crate::auth::get_token() {
                            Some(t) => format!("{}?token={}", resp.ws_url, t),
                            None    => resp.ws_url.clone(),
                        };
                        nav(
                            &format!(
                                "/devices/{id}/tunnels?session={}&type=tty&ws={}",
                                resp.session_id,
                                js_sys::encode_uri_component(&ws_url),
                            ),
                            Default::default(),
                        );
                    }
                    Err(e) => tunnel_error.set(Some(format!("TTY tunnel failed: {e}"))),
                }
            });
        }
    };

    view! {
        <div class="page">
            <header class="page-header">
                {move || device_id().map(|id| view! {
                    <a href=format!("/devices/{id}") class="btn btn-ghost btn-sm">"← Device"</a>
                })}
                <h2 class="page-title">"Tunnels"</h2>
            </header>

            <main class="page-content">
                <Show
                    when=move || ws_url().is_some()
                    fallback=move || view! {
                        <div class="tunnel-open-panel">
                            <p class="empty-state">"No active tunnel session."</p>

                            <Show
                                when=move || tunnel_error.get().is_some()
                                fallback=|| view! {}
                            >
                                <div class="error-banner">
                                    <p>{move || tunnel_error.get().unwrap_or_default()}</p>
                                </div>
                            </Show>

                            <div class="tunnel-actions">
                                <button
                                    class="btn btn-primary"
                                    on:click=open_ssh.clone()
                                >
                                    "Open SSH Tunnel"
                                </button>
                                <button
                                    class="btn btn-secondary"
                                    on:click=open_tty.clone()
                                >
                                    "Open TTY Tunnel"
                                </button>
                            </div>
                        </div>
                    }
                >
                    <div class="tunnel-session">
                        <div class="session-info">
                            <span class="session-label">"Session ID:"</span>
                            <code class="session-id">
                                {move || session_id().unwrap_or_default()}
                            </code>
                            <span class="session-type-label">"Type:"</span>
                            <span class="session-type">
                                {move || tunnel_type().to_uppercase()}
                            </span>
                        </div>

                        {move || ws_url().map(|url| view! {
                            <div class="terminal-wrapper">
                                <crate::components::Terminal ws_url=url/>
                            </div>
                        })}

                        <div class="tunnel-footer">
                            {move || device_id().map(|id| view! {
                                <a
                                    href=format!("/devices/{id}/tunnels")
                                    class="btn btn-ghost btn-sm"
                                >
                                    "Close Session"
                                </a>
                            })}
                        </div>
                    </div>
                </Show>
            </main>
        </div>
    }
}
