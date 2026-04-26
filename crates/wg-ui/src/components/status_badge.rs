use leptos::prelude::*;

#[component]
pub fn StatusBadge(connected: bool) -> impl IntoView {
    if connected {
        view! {
            <span class="status-badge connected">
                <span class="status-dot"></span>
                "Connected"
            </span>
        }
    } else {
        view! {
            <span class="status-badge disconnected">
                <span class="status-dot"></span>
                "Offline"
            </span>
        }
    }
}
