use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, WebSocket};

#[component]
pub fn Terminal(ws_url: String) -> impl IntoView {
    // Accumulated terminal output — text is appended as it arrives.
    let lines: RwSignal<String> = RwSignal::new(String::new());
    // Current value of the input box.
    let input_val: RwSignal<String> = RwSignal::new(String::new());
    // The live WebSocket handle; stored as a non-reactive owned value so we
    // can call .send_with_str() from the keydown handler.
    let ws_handle: StoredValue<Option<WebSocket>> = StoredValue::new(None);

    // Open the WebSocket and wire up callbacks once the component is mounted.
    let ws_url_owned = ws_url.clone();
    Effect::new(move |_| {
        let ws = match WebSocket::new(&ws_url_owned) {
            Ok(w)  => w,
            Err(_) => {
                lines.update(|s| s.push_str("[error] Could not open WebSocket\n"));
                return;
            }
        };

        // --- onopen ---
        let lines_open = lines;
        let onopen = Closure::<dyn FnMut()>::new(move || {
            lines_open.update(|s| s.push_str("[connected]\n"));
        });
        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();

        // --- onmessage ---
        let lines_msg = lines;
        let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
            let text = e.data().as_string().unwrap_or_default();
            lines_msg.update(|s| s.push_str(&text));
        });
        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        // --- onerror ---
        let lines_err = lines;
        let onerror = Closure::<dyn FnMut()>::new(move || {
            lines_err.update(|s| s.push_str("[error] WebSocket error\n"));
        });
        ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        onerror.forget();

        // --- onclose ---
        let lines_close = lines;
        let onclose = Closure::<dyn FnMut()>::new(move || {
            lines_close.update(|s| s.push_str("\n[disconnected]\n"));
        });
        ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
        onclose.forget();

        ws_handle.set_value(Some(ws));
    });

    // Send the current input value over the WebSocket on Enter.
    let on_keydown = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Enter" {
            ev.prevent_default();
            let text = input_val.get();
            if text.is_empty() {
                return;
            }
            let to_send = format!("{}\n", text);
            ws_handle.with_value(|opt| {
                if let Some(ws) = opt {
                    let _ = ws.send_with_str(&to_send);
                }
            });
            // Echo locally so the user sees what was typed.
            lines.update(|s| {
                s.push_str("> ");
                s.push_str(&text);
                s.push('\n');
            });
            input_val.set(String::new());
        }
    };

    view! {
        <div class="terminal-wrapper">
            <div class="terminal">
                <pre>{move || lines.get()}</pre>
            </div>
            <div class="terminal-input-row">
                <span class="terminal-prompt">"$ "</span>
                <input
                    type="text"
                    class="terminal-input"
                    placeholder="Type command and press Enter…"
                    prop:value=move || input_val.get()
                    on:input=move |ev| input_val.set(event_target_value(&ev))
                    on:keydown=on_keydown
                    autocomplete="off"
                    spellcheck="false"
                />
            </div>
        </div>
    }
}
