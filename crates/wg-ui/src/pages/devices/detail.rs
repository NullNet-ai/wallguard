use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use uuid::Uuid;
use wasm_bindgen_futures::spawn_local;

use wg_shared::types::Feature;

fn fmt_ts(ms: i64) -> String {
    let secs = ms / 1000;
    let minutes = secs / 60;
    let hours = minutes / 60;
    let days = hours / 24;

    // Julian Day Number calculation (Gregorian calendar).
    let jd = days + 2440588; // Unix epoch is JD 2440588
    let a = jd + 32044;
    let b = (4 * a + 3) / 146097;
    let c = a - (146097 * b) / 4;
    let d = (4 * c + 3) / 1461;
    let e = c - (1461 * d) / 4;
    let m = (5 * e + 2) / 153;

    let day = e - (153 * m + 2) / 5 + 1;
    let month = m + 3 - 12 * (m / 10);
    let year = 100 * b + d - 4800 + m / 10;

    let hour = hours % 24;
    let minute = minutes % 60;

    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}")
}

#[component]
pub fn DeviceDetail() -> impl IntoView {
    let params = use_params_map();
    let navigate = use_navigate();

    let device_id = move || {
        params.read().get("id")
            .and_then(|s| Uuid::parse_str(&s).ok())
    };

    let device_resource = LocalResource::new(move || async move {
        match device_id() {
            Some(id) => crate::api::devices::get(id).await,
            None => Err("Invalid device ID".to_string()),
        }
    });

    let status_resource = LocalResource::new(move || async move {
        match device_id() {
            Some(id) => crate::api::devices::status(id).await,
            None => Err("Invalid device ID".to_string()),
        }
    });

    let tunnel_error = RwSignal::new(Option::<String>::None);

    // Wrap in StoredValue so the handlers are Copy and can be captured by
    // multiple reactive/children closures without making the outer closure FnOnce.
    let open_ssh = StoredValue::new({
        let navigate = navigate.clone();
        move |_: leptos::ev::MouseEvent| {
            let Some(id) = device_id() else { return };
            let nav = navigate.clone();
            tunnel_error.set(None);
            spawn_local(async move {
                match crate::api::tunnels::open_ssh(id).await {
                    Ok(resp) => {
                        nav(
                            &format!("/devices/{id}/tunnels?session={}&type=ssh&ws={}", resp.session_id, js_sys::encode_uri_component(&resp.ws_url)),
                            Default::default(),
                        );
                    }
                    Err(e) => tunnel_error.set(Some(format!("SSH tunnel failed: {e}"))),
                }
            });
        }
    });

    let open_tty = StoredValue::new({
        let navigate = navigate.clone();
        move |_: leptos::ev::MouseEvent| {
            let Some(id) = device_id() else { return };
            let nav = navigate.clone();
            tunnel_error.set(None);
            spawn_local(async move {
                match crate::api::tunnels::open_tty(id).await {
                    Ok(resp) => {
                        nav(
                            &format!("/devices/{id}/tunnels?session={}&type=tty&ws={}", resp.session_id, js_sys::encode_uri_component(&resp.ws_url)),
                            Default::default(),
                        );
                    }
                    Err(e) => tunnel_error.set(Some(format!("TTY tunnel failed: {e}"))),
                }
            });
        }
    });

    view! {
        <div class="page">
            <header class="page-header">
                <button class="btn btn-ghost btn-sm" on:click=move |_| navigate("/devices", Default::default())>"← Devices"</button>
                <h2 class="page-title">"Device Detail"</h2>
            </header>

            <main class="page-content">
                <Suspense fallback=|| view! { <p class="loading">"Loading device..."</p> }>
                    {move || Suspend::new(async move {
                        let dev_result    = device_resource.await;
                        let status_result = status_resource.await;

                        match dev_result {
                            Err(e) => view! {
                                <div class="error-banner"><p>{e}</p></div>
                            }.into_any(),
                            Ok(device) => {
                                    let connected = status_result.ok()
                                        .map(|s| s.connected)
                                        .unwrap_or_else(|| {
                                            device.last_seen_at
                                                .map(|ts| {
                                                    let now = js_sys::Date::now() as i64;
                                                    now - ts < 120_000
                                                })
                                                .unwrap_or(false)
                                        });

                                    let id = device.id;
                                    let last_seen_str = device.last_seen_at
                                        .map(fmt_ts)
                                        .unwrap_or_else(|| "Never".to_string());
                                    let features_str = device.features
                                        .iter()
                                        .map(|f| format!("{f:?}"))
                                        .collect::<Vec<_>>()
                                        .join(", ");

                                    view! {
                                        <div class="device-detail">
                                            <div class="detail-header">
                                                <h3 class="detail-name">{device.display_name.clone()}</h3>
                                                <crate::components::StatusBadge connected=connected/>
                                            </div>

                                            <div class="detail-meta">
                                                <div class="meta-row">
                                                    <span class="meta-label">"Firewall"</span>
                                                    <span class="meta-value">
                                                        {format!("{:?}", device.firewall_kind)}
                                                    </span>
                                                </div>
                                                <div class="meta-row">
                                                    <span class="meta-label">"Agent Version"</span>
                                                    <span class="meta-value">
                                                        {device.agent_version
                                                            .clone()
                                                            .unwrap_or_else(|| "Unknown".to_string())}
                                                    </span>
                                                </div>
                                                <div class="meta-row">
                                                    <span class="meta-label">"Last Seen"</span>
                                                    <span class="meta-value">{last_seen_str}</span>
                                                </div>
                                                <div class="meta-row">
                                                    <span class="meta-label">"Features"</span>
                                                    <span class="meta-value">
                                                        {if features_str.is_empty() {
                                                            "None".to_string()
                                                        } else {
                                                            features_str
                                                        }}
                                                    </span>
                                                </div>
                                                {device.notes.clone().map(|notes| view! {
                                                    <div class="meta-row">
                                                        <span class="meta-label">"Notes"</span>
                                                        <span class="meta-value">{notes}</span>
                                                    </div>
                                                })}
                                            </div>

                                            <div class="detail-actions">
                                                {
                                                    let features = device.features.clone();
                                                    view! {
                                                        <Show
                                                            when=move || features.contains(&Feature::SshTunnel)
                                                            fallback=|| view! {}
                                                        >
                                                            <button
                                                                class="btn btn-primary"
                                                                on:click=move |e| open_ssh.update_value(|f| f(e))
                                                                disabled=move || !connected
                                                            >
                                                                "Open SSH"
                                                            </button>
                                                        </Show>
                                                    }
                                                }

                                                <Show
                                                    when=move || device.features.contains(&Feature::TtyTunnel)
                                                    fallback=|| view! {}
                                                >
                                                    <button
                                                        class="btn btn-secondary"
                                                        on:click=move |e| open_tty.update_value(|f| f(e))
                                                        disabled=move || !connected
                                                    >
                                                        "Open TTY"
                                                    </button>
                                                </Show>
                                            </div>

                                            <Show
                                                when=move || tunnel_error.get().is_some()
                                                fallback=|| view! {}
                                            >
                                                <div class="error-banner">
                                                    <p>{move || tunnel_error.get().unwrap_or_default()}</p>
                                                </div>
                                            </Show>

                                            <nav class="detail-tabs">
                                                <a class="tab-link" href=format!("/devices/{id}/failures")>
                                                    "Failures"
                                                </a>
                                                <a class="tab-link" href=format!("/devices/{id}/tunnels")>
                                                    "Tunnels"
                                                </a>
                                            </nav>
                                        </div>
                                    }.into_any()
                                }
                        }
                    })}
                </Suspense>
            </main>
        </div>
    }
}
