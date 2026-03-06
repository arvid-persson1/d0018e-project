use crate::Route;
use crate::state::{GlobalState, UserRole};
use dioxus::prelude::*;

/// Administration page.
/// TODO(auth): Kontrollera att inloggad användare har rollen Administrator innan sidan visas.
#[allow(clippy::same_name_method, reason = "Dioxus macro limitation")]
#[component]
pub fn Administration() -> Element {
    let global_state = use_context::<Signal<GlobalState>>();

    // TODO(auth): Ersätt med riktig admin-rollkontroll mot DB
    let is_admin = false; // global_state.read().auth.user.as_ref().map_or(false, |u| u.role == UserRole::Administrator)

    rsx! {
        div { class: "min-h-screen bg-gray-50 p-6",
            div { class: "max-w-4xl mx-auto",
                if is_admin {
                    // TODO(auth): Admin-innehåll visas här
                    h1 { class: "text-3xl font-black text-gray-900 mb-6", "Administration" }
                    p { class: "text-gray-400 italic",
                        "Adminpanel laddas när databasen och auth är kopplade."
                    }
                } else {
                    div { class: "bg-white rounded-2xl shadow-sm p-8 text-center",
                        i { class: "fa-solid fa-lock text-5xl text-gray-300 mb-4 block" }
                        h1 { class: "text-2xl font-black text-gray-900 mb-2", "Åtkomst nekad" }
                        p { class: "text-gray-500 mb-6",
                            "Den här sidan kräver administratörsbehörighet."
                        }
                        Link {
                            to: Route::Home {},
                            class: "bg-green-700 text-white px-8 py-3 rounded-full font-black hover:bg-green-800 transition",
                            "Tillbaka till start"
                        }
                    }
                }
            }
        }
    }
}