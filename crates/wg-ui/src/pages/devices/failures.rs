use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use uuid::Uuid;
use wg_shared::types::FailureSeverity;

fn fmt_ts(ms: i64) -> String {
    let secs = ms / 1000;
    let minutes = secs / 60;
    let hours = minutes / 60;
    let days = hours / 24;

    let jd = days + 2440588;
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

fn severity_class(s: FailureSeverity) -> &'static str {
    match s {
        FailureSeverity::Warning => "severity-warning",
        FailureSeverity::Error => "severity-error",
        FailureSeverity::Fatal => "severity-fatal",
    }
}

const PAGE_SIZE: u32 = 20;

#[component]
pub fn DeviceFailures() -> impl IntoView {
    let params = use_params_map();

    let device_id = move || {
        params.read().get("id")
            .and_then(|s| Uuid::parse_str(&s).ok())
    };

    let severity_filter = RwSignal::new(Option::<String>::None);
    let offset = RwSignal::new(0u32);

    let failures_resource = Resource::new(
        move || (device_id(), offset.get(), severity_filter.get()),
        |(id, off, sev)| async move {
            match id {
                Some(id) => crate::api::failures::list(id, off, sev.as_deref()).await,
                None => Err("Invalid device ID".to_string()),
            }
        },
    );

    let on_severity_change = move |ev: leptos::ev::Event| {
        let val = event_target_value(&ev);
        severity_filter.set(if val.is_empty() { None } else { Some(val) });
        offset.set(0);
    };

    view! {
        <div class="page">
            <header class="page-header">
                {move || device_id().map(|id| view! {
                    <a href=format!("/devices/{id}") class="btn btn-ghost btn-sm">"← Device"</a>
                })}
                <h2 class="page-title">"Failures"</h2>
            </header>

            <main class="page-content">
                <div class="filter-bar">
                    <label for="severity-filter">"Severity"</label>
                    <select
                        id="severity-filter"
                        on:change=on_severity_change
                    >
                        <option value="">"All"</option>
                        <option value="warning">"Warning"</option>
                        <option value="error">"Error"</option>
                        <option value="fatal">"Fatal"</option>
                    </select>
                </div>

                <Suspense fallback=|| view! { <p class="loading">"Loading failures..."</p> }>
                    {move || {
                        failures_resource.get().map(|result| {
                            match result {
                                Err(e) => view! {
                                    <div class="error-banner"><p>{e}</p></div>
                                }.into_any(),
                                Ok(resp) => {
                                    if resp.items.is_empty() {
                                        view! {
                                            <p class="empty-state">"No failures found."</p>
                                        }.into_any()
                                    } else {
                                        let total = resp.total;
                                        let items = resp.items;
                                        let current_offset = offset.get();

                                        view! {
                                            <div class="failures-list">
                                                <For
                                                    each=move || items.clone()
                                                    key=|f| f.failure_id
                                                    children=|failure| {
                                                        let sev_cls = severity_class(failure.severity);
                                                        let ts = fmt_ts(failure.occurred_at);
                                                        view! {
                                                            <div class=format!("failure-row {sev_cls}")>
                                                                <div class="failure-header">
                                                                    <span class=format!("severity-badge {sev_cls}")>
                                                                        {format!("{:?}", failure.severity)}
                                                                    </span>
                                                                    <span class="failure-category">
                                                                        {format!("{:?}", failure.category)}
                                                                    </span>
                                                                    <span class="failure-time">{ts}</span>
                                                                    <Show
                                                                        when=move || failure.is_replay
                                                                        fallback=|| view! {}
                                                                    >
                                                                        <span class="replay-badge">"replay"</span>
                                                                    </Show>
                                                                </div>
                                                                <div class="failure-message">
                                                                    {failure.message.clone()}
                                                                </div>
                                                            </div>
                                                        }
                                                    }
                                                />
                                            </div>

                                            <div class="pagination">
                                                <button
                                                    class="btn btn-sm btn-ghost"
                                                    disabled=move || current_offset == 0
                                                    on:click=move |_| {
                                                        offset.update(|o| {
                                                            *o = o.saturating_sub(PAGE_SIZE)
                                                        });
                                                    }
                                                >
                                                    "← Prev"
                                                </button>
                                                <span class="page-info">
                                                    {format!(
                                                        "{} – {} of {}",
                                                        current_offset + 1,
                                                        (current_offset + PAGE_SIZE).min(total as u32),
                                                        total,
                                                    )}
                                                </span>
                                                <button
                                                    class="btn btn-sm btn-ghost"
                                                    disabled=move || {
                                                        (current_offset + PAGE_SIZE) >= total as u32
                                                    }
                                                    on:click=move |_| {
                                                        offset.update(|o| *o += PAGE_SIZE);
                                                    }
                                                >
                                                    "Next →"
                                                </button>
                                            </div>
                                        }.into_any()
                                    }
                                }
                            }
                        })
                    }}
                </Suspense>
            </main>
        </div>
    }
}
