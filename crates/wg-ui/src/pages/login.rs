use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use wasm_bindgen_futures::spawn_local;

use crate::auth::{set_token, AuthSignal};

#[component]
pub fn Login() -> impl IntoView {
    let token = use_context::<AuthSignal>().expect("auth context");
    let navigate = use_navigate();

    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error_msg = RwSignal::new(Option::<String>::None);
    let loading = RwSignal::new(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let email_val = email.get();
        let password_val = password.get();
        let nav = navigate.clone();

        loading.set(true);
        error_msg.set(None);

        spawn_local(async move {
            match crate::api::auth::login(&email_val, &password_val).await {
                Ok(resp) => {
                    set_token(&resp.access_token);
                    token.set(Some(resp.access_token));
                    nav("/", Default::default());
                }
                Err(e) => {
                    error_msg.set(Some(e));
                    loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="login-page">
            <div class="login-card">
                <h1 class="login-title">"WallGuard"</h1>
                <p class="login-subtitle">"Sign in to your account"</p>

                <form on:submit=on_submit class="login-form">
                    <div class="form-group">
                        <label for="email">"Email"</label>
                        <input
                            id="email"
                            type="email"
                            placeholder="you@example.com"
                            prop:value=move || email.get()
                            on:input=move |ev| email.set(event_target_value(&ev))
                            required
                        />
                    </div>

                    <div class="form-group">
                        <label for="password">"Password"</label>
                        <input
                            id="password"
                            type="password"
                            placeholder="••••••••"
                            prop:value=move || password.get()
                            on:input=move |ev| password.set(event_target_value(&ev))
                            required
                        />
                    </div>

                    <Show
                        when=move || error_msg.get().is_some()
                        fallback=|| view! {}
                    >
                        <p class="login-error">
                            {move || error_msg.get().unwrap_or_default()}
                        </p>
                    </Show>

                    <button
                        type="submit"
                        class="btn btn-primary btn-full"
                        disabled=move || loading.get()
                    >
                        <Show
                            when=move || loading.get()
                            fallback=|| view! { "Sign In" }
                        >
                            "Signing in..."
                        </Show>
                    </button>
                </form>
            </div>
        </div>
    }
}
