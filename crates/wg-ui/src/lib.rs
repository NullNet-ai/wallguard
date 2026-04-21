// wg-ui — WallGuard web UI (Leptos / WASM).
// Phase 0 stub. Full UI implemented in Phase 9b.
// Build with: trunk build --release
// Do not build with plain cargo — this crate targets wasm32.

use leptos::*;

#[component]
pub fn App() -> impl IntoView {
    view! { <p>"WallGuard UI (stub)"</p> }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    leptos::mount_to_body(App);
}
