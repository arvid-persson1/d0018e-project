use crate::Route;
use crate::state::GlobalState;
use dioxus::prelude::*;

/// Favorites page.
#[component]
pub fn FavoritesPage() -> Element {
    let global_state = use_context::<Signal<GlobalState>>();

    // TODO(db): När inloggning finns, använd inloggad kunds ID istället för None
    // Just nu visas favoriter från GlobalState eftersom vi inte har login än

    // TODO(db): Byt ut detta mot favorites(customer_id, 50, 0) när login finns

    rsx! {
        div { class: "container mx-auto p-8",
            Link {
                to: Route::Home {},
                class: "text-green-700 hover:text-green-900 font-bold flex items-center gap-2 mb-4 transition-colors",
                i { class: "fa-solid fa-arrow-left" }
                "Tillbaka till start"
            }
            h1 { class: "text-3xl font-black mb-8", "Mina Favoriter" }

            if global_state.read().favorites.is_empty() {
                div { class: "text-center py-20 bg-white rounded-2xl shadow-sm border border-gray-100",
                    i { class: "fa-regular fa-heart text-6xl text-gray-200 mb-4" }
                    p { class: "text-gray-500 text-xl", "Du har inga sparade produkter här!" }
                }
            } else {
                // TODO(db): När login finns, ersätt med use_resource som kallar favorites(customer_id, 50, 0)
                // och loopa över ProductOverviewFavorited istället
                p { class: "text-gray-500", "Dina favoriter visas här när databasen är kopplad." }
            }
        }
    }
}
