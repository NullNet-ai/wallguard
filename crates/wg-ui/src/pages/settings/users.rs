use leptos::prelude::*;
use uuid::Uuid;
use wasm_bindgen_futures::spawn_local;
use wg_shared::types::Role;

use crate::api::users::CreateUserRequest;

#[component]
pub fn UsersPage() -> impl IntoView {
    let users_resource = LocalResource::new(
        || async { crate::api::users::list().await },
    );

    let form_email    = RwSignal::new(String::new());
    let form_name     = RwSignal::new(String::new());
    let form_role     = RwSignal::new("viewer".to_string());
    let form_password = RwSignal::new(String::new());
    let form_error    = RwSignal::new(Option::<String>::None);
    let form_loading  = RwSignal::new(false);

    let on_create = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let email    = form_email.get();
        let name     = form_name.get();
        let password = form_password.get();
        let role = match form_role.get().as_str() {
            "owner"    => Role::Owner,
            "admin"    => Role::Admin,
            "operator" => Role::Operator,
            _          => Role::Viewer,
        };

        form_error.set(None);
        form_loading.set(true);

        spawn_local(async move {
            let req = CreateUserRequest { email, name, password, role };
            match crate::api::users::create(req).await {
                Ok(_) => {
                    form_email.set(String::new());
                    form_name.set(String::new());
                    form_password.set(String::new());
                    form_role.set("viewer".to_string());
                    form_loading.set(false);
                    users_resource.refetch();
                }
                Err(e) => {
                    form_error.set(Some(e));
                    form_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="page">
            <header class="page-header">
                <a href="/" class="btn btn-ghost btn-sm">"← Dashboard"</a>
                <h2 class="page-title">"User Management"</h2>
            </header>

            <main class="page-content">
                <section class="settings-section">
                    <h3 class="section-subtitle">"Users"</h3>

                    <Suspense fallback=|| view! { <p class="loading">"Loading users..."</p> }>
                        {move || Suspend::new(async move {
                            match users_resource.await {
                                Err(e) => view! {
                                    <div class="error-banner"><p>{e}</p></div>
                                }.into_any(),
                                Ok(resp) => {
                                    if resp.items.is_empty() {
                                        view! {
                                            <p class="empty-state">"No users found."</p>
                                        }.into_any()
                                    } else {
                                        let items = resp.items;
                                        view! {
                                            <table class="users-table">
                                                <thead>
                                                    <tr>
                                                        <th>"Name"</th>
                                                        <th>"Email"</th>
                                                        <th>"Role"</th>
                                                        <th>"Actions"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    <For
                                                        each=move || items.clone()
                                                        key=|u| u.id
                                                        children=move |user| {
                                                            let user_id: Uuid = user.id;
                                                            view! {
                                                                <tr class="user-row">
                                                                    <td>{user.name.clone()}</td>
                                                                    <td>{user.email.clone()}</td>
                                                                    <td>
                                                                        <span class=format!(
                                                                            "role-badge role-{}",
                                                                            format!("{:?}", user.role).to_lowercase()
                                                                        )>
                                                                            {format!("{:?}", user.role)}
                                                                        </span>
                                                                    </td>
                                                                    <td>
                                                                        <button
                                                                            class="btn btn-sm btn-danger"
                                                                            on:click=move |_| {
                                                                                spawn_local(async move {
                                                                                    let _ = crate::api::users::delete(user_id).await;
                                                                                    users_resource.refetch();
                                                                                });
                                                                            }
                                                                        >
                                                                            "Delete"
                                                                        </button>
                                                                    </td>
                                                                </tr>
                                                            }
                                                        }
                                                    />
                                                </tbody>
                                            </table>
                                        }.into_any()
                                    }
                                }
                            }
                        })}
                    </Suspense>
                </section>

                <section class="settings-section">
                    <h3 class="section-subtitle">"Add User"</h3>

                    <form class="create-user-form" on:submit=on_create>
                        <div class="form-grid">
                            <div class="form-group">
                                <label for="new-email">"Email"</label>
                                <input
                                    id="new-email"
                                    type="email"
                                    placeholder="user@example.com"
                                    prop:value=move || form_email.get()
                                    on:input=move |ev| form_email.set(event_target_value(&ev))
                                    required
                                />
                            </div>
                            <div class="form-group">
                                <label for="new-name">"Name"</label>
                                <input
                                    id="new-name"
                                    type="text"
                                    placeholder="Display name"
                                    prop:value=move || form_name.get()
                                    on:input=move |ev| form_name.set(event_target_value(&ev))
                                    required
                                />
                            </div>
                            <div class="form-group">
                                <label for="new-role">"Role"</label>
                                <select
                                    id="new-role"
                                    on:change=move |ev| form_role.set(event_target_value(&ev))
                                >
                                    <option value="viewer">"Viewer"</option>
                                    <option value="operator">"Operator"</option>
                                    <option value="admin">"Admin"</option>
                                    <option value="owner">"Owner"</option>
                                </select>
                            </div>
                            <div class="form-group">
                                <label for="new-password">"Password"</label>
                                <input
                                    id="new-password"
                                    type="password"
                                    placeholder="••••••••"
                                    prop:value=move || form_password.get()
                                    on:input=move |ev| form_password.set(event_target_value(&ev))
                                    required
                                />
                            </div>
                        </div>

                        <Show
                            when=move || form_error.get().is_some()
                            fallback=|| view! {}
                        >
                            <p class="form-error">
                                {move || form_error.get().unwrap_or_default()}
                            </p>
                        </Show>

                        <button
                            type="submit"
                            class="btn btn-primary"
                            disabled=move || form_loading.get()
                        >
                            <Show
                                when=move || form_loading.get()
                                fallback=|| view! { "Add User" }
                            >
                                "Adding..."
                            </Show>
                        </button>
                    </form>
                </section>
            </main>
        </div>
    }
}
