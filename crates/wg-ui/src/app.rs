use leptos::prelude::*;
use leptos_router::components::{Redirect, Route, Router, Routes};
use leptos_router::path;

use crate::auth::{self, AuthSignal};
use crate::pages::{
    dashboard::Dashboard,
    devices::{detail::DeviceDetail, list::DeviceList},
    login::Login,
    settings::{install_codes::InstallCodesPage, users::UsersPage},
};

/// Root application component.
///
/// Sets up the router and provides `AuthSignal` via context so every descendant
/// component can read or mutate the current JWT without prop-drilling.
#[component]
pub fn App() -> impl IntoView {
    let token: AuthSignal = auth::init_auth();
    provide_context(token);

    view! {
        <Router>
            <Routes fallback=|| view! { <p class="not-found">"404 — Page not found"</p> }>

                // Public route — login page
                <Route path=path!("/login") view=Login/>

                // Dashboard (auth-gated)
                <Route path=path!("/") view=move || {
                    if token.get().is_none() {
                        view! { <Redirect path="/login"/> }.into_any()
                    } else {
                        view! { <Dashboard/> }.into_any()
                    }
                }/>

                // Device list (auth-gated)
                <Route path=path!("/devices") view=move || {
                    if token.get().is_none() {
                        view! { <Redirect path="/login"/> }.into_any()
                    } else {
                        view! { <DeviceList/> }.into_any()
                    }
                }/>

                // Device detail (auth-gated)
                <Route path=path!("/devices/:id") view=move || {
                    if token.get().is_none() {
                        view! { <Redirect path="/login"/> }.into_any()
                    } else {
                        view! { <DeviceDetail/> }.into_any()
                    }
                }/>

                // Device failures sub-page (auth-gated)
                <Route path=path!("/devices/:id/failures") view=move || {
                    if token.get().is_none() {
                        view! { <Redirect path="/login"/> }.into_any()
                    } else {
                        view! { <crate::pages::devices::failures::DeviceFailures/> }.into_any()
                    }
                }/>

                // Device tunnels sub-page (auth-gated)
                <Route path=path!("/devices/:id/tunnels") view=move || {
                    if token.get().is_none() {
                        view! { <Redirect path="/login"/> }.into_any()
                    } else {
                        view! { <crate::pages::devices::tunnels::DeviceTunnels/> }.into_any()
                    }
                }/>

                // User management settings (auth-gated)
                <Route path=path!("/settings/users") view=move || {
                    if token.get().is_none() {
                        view! { <Redirect path="/login"/> }.into_any()
                    } else {
                        view! { <UsersPage/> }.into_any()
                    }
                }/>

                // Installation codes (auth-gated)
                <Route path=path!("/settings/install-codes") view=move || {
                    if token.get().is_none() {
                        view! { <Redirect path="/login"/> }.into_any()
                    } else {
                        view! { <InstallCodesPage/> }.into_any()
                    }
                }/>

            </Routes>
        </Router>
    }
}
