use crate::Route;
use crate::database::{LoginId, log_out};
use crate::state::GlobalState;
use dioxus::prelude::*;

/// Dropdown shown when clicking the profile icon in the navbar.
#[component]
pub fn AuthDropdown(on_close: EventHandler<()>) -> Element {
let global_state = use_context::<Signal<GlobalState>>();
let login = global_state.read().login.clone();
let nav = use_navigator();

rsx! {
    div { class: "fixed inset-0 z-40", onclick: move |_| on_close.call(()) }
    div { class: "absolute right-0 top-full mt-2 w-56 bg-white rounded-2xl shadow-xl border z-50 overflow-hidden",
        if let Some(ref user) = login {
            div { class: "p-4 border-b",
                p { class: "font-bold text-gray-900", "{user.username}" }
                p { class: "text-xs text-gray-400",
                    match user.id {
                        LoginId::Vendor(_) => "Företag",
                        LoginId::Administrator(_) => "Admin",
                        _ => "Kund",
                    }
                }
            }

            div { class: "p-2",
                Link {
                    to: match user.id {
                        LoginId::Vendor(id) => Route::Vendor { id },
                        _ => Route::CustomerProfile {},
                    },
                    class: "flex items-center gap-3 px-3 py-2 rounded-xl hover:bg-gray-50 text-sm text-gray-700 transition",
                    onclick: move |_| on_close.call(()),
                    i { class: "fa-solid fa-user w-4" }
                    match user.id {
                        LoginId::Vendor(_) => "Min butik",
                        _ => "Min profil",
                    }
                }

                div { class: "border-t my-2" }
                button {
                    class: "flex items-center gap-3 px-3 py-2 rounded-xl hover:bg-red-50 text-sm text-red-600 transition w-full",
                    onclick: move |_| {
                        let mut gs = global_state;
                        let on_close = on_close.clone();
                        let nav = nav.clone();
                        let _task = spawn(async move {
                            drop(log_out().await);
                            gs.write().login = None;
                            on_close.call(());
                            let _unused = nav.push(Route::Home {});
                        });
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