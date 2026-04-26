mod api;
mod app;
mod auth;
mod components;
mod pages;

use app::App;

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    leptos::mount::mount_to_body(App);
}
