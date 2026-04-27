use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api::install_codes::InstallationCodeRow;

fn format_expiry(expires_at_ms: i64) -> String {
    let now_ms = js_sys::Date::now() as i64;
    let diff_ms = expires_at_ms - now_ms;
    if diff_ms <= 0 {
        return "expired".to_string();
    }
    let hours = diff_ms / 3_600_000;
    if hours < 1 {
        let mins = diff_ms / 60_000;
        format!("expires in {mins}m")
    } else if hours < 48 {
        format!("expires in {hours}h")
    } else {
        format!("expires in {}d", hours / 24)
    }
}

#[component]
pub fn InstallCodesPage() -> impl IntoView {
    let codes_resource = LocalResource::new(
        || async { crate::api::install_codes::list().await },
    );

    let ttl_hours   = RwSignal::new("24".to_string());
    let new_code    = RwSignal::new(Option::<InstallationCodeRow>::None);
    let form_error  = RwSignal::new(Option::<String>::None);
    let generating  = RwSignal::new(false);

    let on_generate = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let ttl: Option<i64> = ttl_hours.get().parse().ok();
        form_error.set(None);
        generating.set(true);
        new_code.set(None);

        spawn_local(async move {
            match crate::api::install_codes::create(ttl).await {
                Ok(row) => {
                    new_code.set(Some(row));
                    codes_resource.refetch();
                }
                Err(e) => form_error.set(Some(e)),
            }
            generating.set(false);
        });
    };

    view! {
        <div class="page">
            <header class="page-header">
                <a href="/" class="btn btn-ghost btn-sm">"← Dashboard"</a>
                <h2 class="page-title">"Installation Codes"</h2>
                <a href="/settings/users" class="btn btn-ghost btn-sm">"Users"</a>
            </header>

            <main class="page-content">

                // ── Generate new code ───────────────────────────────────
                <section class="settings-section">
                    <h3 class="section-subtitle">"Generate a New Code"</h3>
                    <p class="text-muted">
                        "Each code can be used once to enroll a device. Pass it to "
                        <code>"wg-cli enroll --install-code <CODE>"</code>
                        " on the device."
                    </p>

                    <form class="install-code-form" on:submit=on_generate>
                        <div class="form-row">
                            <div class="form-group">
                                <label for="ttl">"Expires after (hours)"</label>
                                <input
                                    id="ttl"
                                    type="number"
                                    min="1"
                                    max="720"
                                    prop:value=move || ttl_hours.get()
                                    on:input=move |ev| ttl_hours.set(event_target_value(&ev))
                                />
                            </div>
                            <button
                                type="submit"
                                class="btn btn-primary"
                                disabled=move || generating.get()
                            >
                                <Show
                                    when=move || generating.get()
                                    fallback=|| view! { "Generate" }
                                >
                                    "Generating…"
                                </Show>
                            </button>
                        </div>

                        <Show
                            when=move || form_error.get().is_some()
                            fallback=|| view! {}
                        >
                            <p class="form-error">{move || form_error.get().unwrap_or_default()}</p>
                        </Show>
                    </form>

                    // Show newly created code prominently
                    <Show
                        when=move || new_code.get().is_some()
                        fallback=|| view! {}
                    >
                        {move || new_code.get().map(|row| view! {
                            <div class="code-result">
                                <p class="code-result-label">"New installation code:"</p>
                                <div class="code-display">
                                    <code class="code-value">{row.code.clone()}</code>
                                    <button
                                        class="btn btn-sm btn-secondary"
                                        on:click={
                                            let code = row.code.clone();
                                            move |_| {
                                                let script = format!(
                                                    "navigator.clipboard.writeText('{}')",
                                                    code
                                                );
                                                let _ = js_sys::eval(&script);
                                            }
                                        }
                                    >
                                        "Copy"
                                    </button>
                                </div>
                                <p class="text-muted">{format_expiry(row.expires_at)}</p>
                            </div>
                        })}
                    </Show>
                </section>

                // ── Active codes list ────────────────────────────────────
                <section class="settings-section">
                    <h3 class="section-subtitle">"Active Codes"</h3>

                    <Suspense fallback=|| view! { <p class="loading">"Loading…"</p> }>
                        {move || Suspend::new(async move {
                            match codes_resource.await {
                                Err(e) => view! {
                                    <div class="error-banner"><p>{e}</p></div>
                                }.into_any(),
                                Ok(resp) => {
                                    if resp.items.is_empty() {
                                        view! {
                                            <p class="empty-state">"No active codes."</p>
                                        }.into_any()
                                    } else {
                                        let items = resp.items;
                                        view! {
                                            <table class="codes-table">
                                                <thead>
                                                    <tr>
                                                        <th>"Code"</th>
                                                        <th>"Status"</th>
                                                        <th>"Expires"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    <For
                                                        each=move || items.clone()
                                                        key=|c| c.code.clone()
                                                        children=|row| view! {
                                                            <tr class="code-row">
                                                                <td>
                                                                    <code class="code-mono">{row.code.clone()}</code>
                                                                </td>
                                                                <td>
                                                                    <Show
                                                                        when=move || row.used
                                                                        fallback=|| view! {
                                                                            <span class="badge badge-success">"Unused"</span>
                                                                        }
                                                                    >
                                                                        <span class="badge badge-muted">"Used"</span>
                                                                    </Show>
                                                                </td>
                                                                <td class="text-muted">
                                                                    {format_expiry(row.expires_at)}
                                                                </td>
                                                            </tr>
                                                        }
                                                    />
                                                </tbody>
                                            </table>
                                        }.into_any()
                                    }
                                }
                            }
                        })}
                    </Suspense>
                </section>

            </main>
        </div>
    }
}
