use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "wgRemoteDesktop"])]
    fn open(canvas_id: &str, ws_url: &str, width: u32, height: u32);

    #[wasm_bindgen(js_namespace = ["window", "wgRemoteDesktop"])]
    fn dispose(canvas_id: &str);

    #[wasm_bindgen(js_namespace = ["window", "wgRemoteDesktop"], js_name = "sendPli")]
    fn send_pli(canvas_id: &str);
}

const CANVAS_ID: &str = "wg-rdp-canvas";

#[component]
pub fn RemoteDesktop(ws_url: String, width: u32, height: u32) -> impl IntoView {
    Effect::new(move |_| {
        open(CANVAS_ID, &ws_url, width, height);
    });

    on_cleanup(|| {
        dispose(CANVAS_ID);
    });

    view! {
        <div class="rdp-container">
            <canvas
                id=CANVAS_ID
                class="rdp-canvas"
                style=format!("display:block;width:100%;aspect-ratio:{}/{}", width, height)
            />
            <div class="rdp-toolbar">
                <button
                    class="btn btn-sm btn-ghost"
                    on:click=move |_| send_pli(CANVAS_ID)
                >
                    "Request keyframe"
                </button>
            </div>
        </div>
    }
}
