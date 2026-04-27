use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::auth::{clear_token, AuthSignal};

#[component]
pub fn Dashboard() -> impl IntoView {
    let token = use_context::<AuthSignal>().expect("auth context");
    let navigate = use_navigate();

    let devices_resource = LocalResource::new(
        || async { crate::api::devices::list().await },
    );

    let failures_resource = LocalResource::new(
        || async {
            crate::api::get::<crate::api::failures::FailuresResponse>(
                "/api/v1/failures?offset=0&limit=20",
            )
            .await
        },
    );

    let sse_messages = RwSignal::new(Vec::<String>::new());

    // SSE subscription via web_sys::EventSource + Closure (no futures dep).
    {
        let sse_messages = sse_messages;
        if let Ok(es) = web_sys::EventSource::new("/api/v1/events") {
            let closure = Closure::<dyn Fn(web_sys::MessageEvent)>::new(
                move |ev: web_sys::MessageEvent| {
                    let data = ev.data().as_string().unwrap_or_default();
                    sse_messages.update(|v| {
                        if v.len() >= 50 {
                            v.remove(0);
                        }
                        v.push(data);
                    });
                },
            );
            es.set_onmessage(Some(closure.as_ref().unchecked_ref()));
            // Leak the closure so it lives for the duration of the page.
            closure.forget();
            // Keep EventSource alive by leaking it too.
            std::mem::forget(es);
        }
    }

    let on_logout = {
        let nav = navigate.clone();
        move |_| {
            clear_token();
            token.set(None);
            nav("/login", Default::default());
        }
    };

    view! {
        <div class="page">
            <header class="page-header">
                <h1 class="page-title">"WallGuard"</h1>
                <nav class="page-nav">
                    <a href="/devices">"Devices"</a>
                    <a href="/settings/users">"Settings"</a>
                </nav>
                <button class="btn btn-ghost" on:click=on_logout>
                    "Logout"
                </button>
            </header>

            <main class="page-content">
                <h2 class="section-title">"Dashboard"</h2>

                <div class="stats-grid">
                    <div class="stat-card">
                        <div class="stat-label">"Connected Devices"</div>
                        <Suspense fallback=|| view! { <div class="stat-value">"—"</div> }>
                            {move || Suspend::new(async move {
                                match devices_resource.await {
                                    Ok(resp) => {
                                        let connected = resp.items.iter()
                                            .filter(|d| d.last_seen_at
                                                .map(|ts| js_sys::Date::now() as i64 - ts < 120_000)
                                                .unwrap_or(false))
                                            .count();
                                        let total = resp.items.len();
                                        view! {
                                            <div class="stat-value">{format!("{connected} / {total}")}</div>
                                        }.into_any()
                                    }
                                    Err(e) => view! {
                                        <div class="stat-value stat-error">{e}</div>
                                    }.into_any(),
                                }
                            })}
                        </Suspense>
                    </div>

                    <div class="stat-card">
                        <div class="stat-label">"Recent Failures"</div>
                        <Suspense fallback=|| view! { <div class="stat-value">"—"</div> }>
                            {move || Suspend::new(async move {
                                match failures_resource.await {
                                    Ok(resp) => view! {
                                        <div class="stat-value">{format!("{}", resp.total)}</div>
                                    }.into_any(),
                                    Err(e) => view! {
                                        <div class="stat-value stat-error">{e}</div>
                                    }.into_any(),
                                }
                            })}
                        </Suspense>
                    </div>

                    <div class="stat-card">
                        <div class="stat-label">"Live Events"</div>
                        <div class="stat-value">
                            {move || sse_messages.get().len().to_string()}
                        </div>
                    </div>
                </div>

                <section class="dashboard-section">
                    <h3 class="section-subtitle">"Recent Live Events"</h3>
                    <div class="event-log">
                        <Show
                            when=move || !sse_messages.get().is_empty()
                            fallback=|| view! { <p class="empty-state">"No live events yet."</p> }
                        >
                            <ul class="event-list">
                                <For
                                    each=move || {
                                        let mut msgs = sse_messages.get();
                                        msgs.reverse();
                                        msgs.into_iter().take(20).enumerate().collect::<Vec<_>>()
                                    }
                                    key=|(i, _)| *i
                                    children=|(_, msg)| view! {
                                        <li class="event-item">
                                            <span class="event-data">{msg}</span>
                                        </li>
                                    }
                                />
                            </ul>
                        </Show>
                    </div>
                </section>

                <section class="dashboard-section">
                    <div class="section-header">
                        <h3 class="section-subtitle">"Devices"</h3>
                        <a href="/devices" class="btn btn-sm btn-secondary">"View All"</a>
                    </div>

                    <Suspense fallback=|| view! { <p>"Loading devices..."</p> }>
                        {move || Suspend::new(async move {
                            match devices_resource.await {
                                Ok(resp) => {
                                    let items = resp.items.into_iter().take(5).collect::<Vec<_>>();
                                    view! {
                                        <ul class="device-summary-list">
                                            <For
                                                each=move || items.clone()
                                                key=|d| d.id
                                                children=|device| {
                                                    let connected = device.last_seen_at
                                                        .map(|ts| js_sys::Date::now() as i64 - ts < 120_000)
                                                        .unwrap_or(false);
                                                    let id = device.id;
                                                    view! {
                                                        <li class="device-summary-item">
                                                            <span class="device-name">
                                                                {device.display_name.clone()}
                                                            </span>
                                                            <crate::components::StatusBadge connected=connected/>
                                                            <a href=format!("/devices/{id}")>"Details"</a>
                                                        </li>
                                                    }
                                                }
                                            />
                                        </ul>
                                    }.into_any()
                                }
                                Err(e) => view! {
                                    <p class="error-text">{e}</p>
                                }.into_any(),
                            }
                        })}
                    </Suspense>
                </section>
            </main>
        </div>
    }
}
