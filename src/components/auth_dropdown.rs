use crate::Route;
use dioxus::prelude::*;

// TODO(auth): Replace this entire component's logic with real auth state.
// When auth:
// Add AuthState to GlobalState (or as its own context):
// pub struct AuthState { pub user: Option<LoggedInUser> }
// pub struct LoggedInUser { pub id: i32, pub username: String, pub role: UserRole }
// pub enum UserRole { Customer, Vendor, Administrator }
//
// App start:
// let session = use_resource(|| async { check_session().await });
//
// Login: POST credentials, get session token, store in AuthState
// Logout: DELETE session, clear AuthState

/// Dropdown shown when clicking the profile icon in the navbar
#[component]
pub fn AuthDropdown(on_close: EventHandler<()>) -> Element {
    // TODO(auth): Replace with: let auth = use_context::<Signal<AuthState>>();
    // Then: if auth.read().user.is_some() { show logged in view } else { show login/register }
    let is_logged_in = false; // TODO(auth): read from auth context

    rsx! {
        // Backdrop
        div { class: "fixed inset-0 z-40", onclick: move |_| on_close.call(()) }

        // Dropdown
        div { class: "absolute right-0 top-full mt-2 w-56 bg-white rounded-2xl shadow-xl border z-50 overflow-hidden",
            if is_logged_in {
                // TODO(auth): Show logged-in user's name here
                // let user = auth.read().user.as_ref().unwrap();
                div { class: "p-4 border-b",
                    p { class: "font-bold text-gray-900", "Anna Andersson" } // TODO(auth)
                    p { class: "text-xs text-gray-400", "Kund" } // TODO(auth): show role
                }
                div { class: "p-2",
                    Link {
                        to: Route::CustomerProfile {},
                        class: "flex items-center gap-3 px-3 py-2 rounded-xl hover:bg-gray-50 text-sm text-gray-700 transition",
                        onclick: move |_| on_close.call(()),
                        i { class: "fa-solid fa-user w-4" }
                        "Min profil"
                    }
                    // TODO(auth): Show vendor link only if role == Vendor
                    // if user.role == UserRole::Vendor {
                    //     Link { to: Route::VendorProfile { id: user.id }, ... }
                    // }
                    div { class: "border-t my-2" }
                    button {
                        class: "flex items-center gap-3 px-3 py-2 rounded-xl hover:bg-red-50 text-sm text-red-600 transition w-full",
                        onclick: move |_| {
                            // TODO(auth): logout logic here
                            // logout().await;
                            // auth.write().user = None;
                            on_close.call(());
                        },
                        i { class: "fa-solid fa-right-from-bracket w-4" }
                        "Logga ut"
                    }
                }
            } else {
                div { class: "p-2",
                    p { class: "text-xs text-gray-400 px-3 py-2 font-semibold uppercase tracking-wide",
                        "Kund"
                    }
                    Link {
                        to: Route::Login {}, // TODO: Add Login route
                        class: "flex items-center gap-3 px-3 py-2 rounded-xl hover:bg-gray-50 text-sm text-gray-700 transition",
                        onclick: move |_| on_close.call(()),
                        i { class: "fa-solid fa-right-to-bracket w-4" }
                        "Logga in"
                    }
                    Link {
                        to: Route::Register {}, // TODO: Add Register route
                        class: "flex items-center gap-3 px-3 py-2 rounded-xl hover:bg-green-50 text-sm text-green-700 transition font-semibold",
                        onclick: move |_| on_close.call(()),
                        i { class: "fa-solid fa-user-plus w-4" }
                        "Skapa konto"
                    }
                    div { class: "border-t my-2" }
                    p { class: "text-xs text-gray-400 px-3 py-2 font-semibold uppercase tracking-wide",
                        "Företag"
                    }
                    Link {
                        to: Route::VendorLogin {}, // TODO: Add VendorLogin route
                        class: "flex items-center gap-3 px-3 py-2 rounded-xl hover:bg-gray-50 text-sm text-gray-700 transition",
                        onclick: move |_| on_close.call(()),
                        i { class: "fa-solid fa-store w-4" }
                        "Logga in som företag"
                    }
                    Link {
                        to: Route::VendorRegister {}, // TODO: Add VendorRegister route
                        class: "flex items-center gap-3 px-3 py-2 rounded-xl hover:bg-green-50 text-sm text-green-700 transition font-semibold",
                        onclick: move |_| on_close.call(()),
                        i { class: "fa-solid fa-building w-4" }
                        "Registrera företag"
                    }
                }
            }
        }
    }
}
