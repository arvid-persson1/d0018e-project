use crate::Route;
use crate::components::ProductCard;
use crate::fake_data::get_fake_products;
use crate::state::GlobalState;
use dioxus::prelude::*;

/// Favorites page.
#[component]
pub fn FavoritesPage() -> Element {
    let global_state = use_context::<Signal<GlobalState>>();
    // TODO(db): Ersätt get_fake_products() med ett API-anrop
    let products = get_fake_products();

    // Filtrera data, de tillagda i favriter är de som syns!
    // TODO(db): Favorit-ID:n ska sparas i databasen per användare istället för GlobalState
    let fav_items: Vec<_> = products
        .into_iter()
        .filter(|p| global_state.read().favorites.contains(&p.id))
        .collect();

    rsx! {
        div { class: "container mx-auto p-8",

            // Knapp för att gå tillbaka till main page
            Link {
                to: Route::Home {},
                class: "text-green-700 hover:text-green-900 font-bold flex items-center gap-2 mb-4 transition-colors",
                i { class: "fa-solid fa-arrow-left" }
                "Tillbaka till start"
            }
            h1 { class: "text-3xl font-black mb-8", "Mina Favoriter" }

            if fav_items.is_empty() {
                div { class: "text-center py-20 bg-white rounded-2xl shadow-sm border border-gray-100",
                    i { class: "fa-regular fa-heart text-6xl text-gray-200 mb-4" }
                    p { class: "text-gray-500 text-xl", "Du har inga sparade produkter här!" }
                }
            } else {
                div { class: "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-6",
                    for p in fav_items {
                        // TODO(db): ProductCard är samma, bara datan ändras
                        ProductCard {
                            id: p.id,
                            name: p.name,
                            price: p.price,
                            image_url: p.image_url,
                            comparison_price: p.comparison_price,
                        }
                    }
                }
            }
        }
    }
}
