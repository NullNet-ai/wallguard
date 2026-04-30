use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "wgTerminal"])]
    fn open(element_id: &str, ws_url: &str);

    #[wasm_bindgen(js_namespace = ["window", "wgTerminal"])]
    fn dispose(element_id: &str);
}

const TERMINAL_ID: &str = "wg-xterm";

#[component]
pub fn Terminal(ws_url: String) -> impl IntoView {
    Effect::new(move |_| {
        open(TERMINAL_ID, &ws_url);
    });

    on_cleanup(|| {
        dispose(TERMINAL_ID);
    });

    view! {
        <div id=TERMINAL_ID class="xterm-container"></div>
    }
}
