use leptos::prelude::*;

#[component]
pub fn DeviceList() -> impl IntoView {
    let devices_resource = LocalResource::new(
        || async { crate::api::devices::list().await },
    );

    view! {
        <div class="page">
            <header class="page-header">
                <a href="/" class="btn btn-ghost btn-sm">"← Dashboard"</a>
                <h2 class="page-title">"Devices"</h2>
            </header>

            <main class="page-content">
                <Suspense fallback=|| view! { <p class="loading">"Loading devices..."</p> }>
                    {move || Suspend::new(async move {
                        match devices_resource.await {
                            Err(e) => view! {
                                <div class="error-banner">
                                    <p>{format!("Failed to load devices: {e}")}</p>
                                    <button
                                        class="btn btn-sm btn-secondary"
                                        on:click=move |_| devices_resource.refetch()
                                    >
                                        "Retry"
                                    </button>
                                </div>
                            }.into_any(),
                            Ok(resp) => {
                                if resp.items.is_empty() {
                                    view! {
                                        <p class="empty-state">"No devices enrolled yet."</p>
                                    }.into_any()
                                } else {
                                    let items = resp.items;
                                    view! {
                                        <div class="device-grid">
                                            <For
                                                each=move || items.clone()
                                                key=|d| d.id
                                                children=|device| {
                                                    let connected = device.last_seen_at
                                                        .map(|ts| {
                                                            let now = js_sys::Date::now() as i64;
                                                            now - ts < 120_000
                                                        })
                                                        .unwrap_or(false);
                                                    view! {
                                                        <crate::components::DeviceCard
                                                            device=device
                                                            connected=connected
                                                        />
                                                    }
                                                }
                                            />
                                        </div>
                                    }.into_any()
                                }
                            }
                        }
                    })}
                </Suspense>
            </main>
        </div>
    }
}
