use leptos::prelude::*;

#[component]
pub fn RemoteDesktop(ws_url: String) -> impl IntoView {
    let _ = ws_url;
    view! {
        <div class="remote-desktop-stub">
            <p>"Remote desktop capture is not yet available."</p>
        </div>
    }
}
