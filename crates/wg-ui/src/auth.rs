use leptos::prelude::*;

const TOKEN_KEY: &str = "wg_token";

pub type AuthSignal = RwSignal<Option<String>>;

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// Reads the JWT from localStorage, returning `None` if absent or on error.
pub fn get_token() -> Option<String> {
    local_storage()?.get_item(TOKEN_KEY).ok()?
}

/// Writes the JWT to localStorage. Silently ignores errors.
pub fn set_token(token: &str) {
    if let Some(storage) = local_storage() {
        let _ = storage.set_item(TOKEN_KEY, token);
    }
}

/// Removes the JWT from localStorage. Silently ignores errors.
pub fn clear_token() {
    if let Some(storage) = local_storage() {
        let _ = storage.remove_item(TOKEN_KEY);
    }
}

/// Creates a reactive `RwSignal` pre-filled from localStorage.
/// Call this once at the root of the component tree, then distribute
/// via `provide_context`.
pub fn init_auth() -> AuthSignal {
    RwSignal::new(get_token())
}
