use crate::Route;
use crate::components::ProductCard;
use crate::database::products::ProductInfo as Product;
use crate::state::GlobalState;
use dioxus::prelude::*;
use rust_decimal::prelude::ToPrimitive;

/// Favorites page.
#[component]
pub fn FavoritesPage() -> Element {
    let global_state = use_context::<Signal<GlobalState>>();

    let favorites_ids = global_state.read().favorites.clone();

    // TODO(db): Hämta riktiga produkter. Just nu en tom lista.
    let all_products: Vec<Product> = vec![];

    // Filtrera fram produkterna
    let fav_items: Vec<_> = all_products
        .iter()
        .filter(|p| favorites_ids.contains(&p.id.get()))
        .collect();
    rsx! {
        div { class: "container mx-auto p-8",
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
                // GRID LÄGE
                div { class: "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-6",
                    for p in fav_items {
                        ProductCard {
                            id: p.id.get(),
                            name: p.name.clone(),
                            price: p.price.to_f64().unwrap_or_default(),
                            comparison_price: format!("{:.2} kr/{}", p.price, p.description),
                            image_url: p.gallery.first().cloned().unwrap_or_default(),
                        }
                    }
                }
            }
        }
    }
}
