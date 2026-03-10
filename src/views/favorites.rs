use crate::Route;
use crate::components::product_card::ProductCard;
use crate::database::products::favorites;
use crate::state::GlobalState;
use dioxus::prelude::*;
use rust_decimal::prelude::ToPrimitive;

/// Favorites page.
#[component]
pub fn FavoritesPage() -> Element {
    let global_state = use_context::<Signal<GlobalState>>();
    let login = global_state.read().login.clone();

    // Hämta customer ID från login
    let customer_id = match login.as_ref().and_then(|l| {
        if let crate::database::LoginId::Customer(id) = l.id {
            Some(id)
        } else {
            None
        }
    }) {
        Some(id) => id,
        None => {
            return rsx! {
                div { class: "container mx-auto p-8",
                    Link {
                        to: Route::Home {},
                        class: "text-green-700 hover:text-green-900 font-bold flex items-center gap-2 mb-4 transition-colors",
                        i { class: "fa-solid fa-arrow-left" }
                        "Tillbaka till start"
                    }
                    h1 { class: "text-3xl font-black mb-8", "Mina Favoriter" }
                    div { class: "text-center py-20 bg-white rounded-2xl shadow-sm border border-gray-100",
                        i { class: "fa-regular fa-heart text-6xl text-gray-200 mb-4" }
                        p { class: "text-gray-500 text-xl mb-4",
                            "Du måste vara inloggad för att se dina favoriter."
                        }
                        Link {
                            to: Route::Login {},
                            class: "bg-green-700 text-white font-black px-6 py-3 rounded-full",
                            "Logga in"
                        }
                    }
                }
            };
        }
    };

    let fav_resource = use_resource(move || async move { favorites(customer_id, 100, 0).await });

    rsx! {
        div { class: "container mx-auto p-8",
            Link {
                to: Route::Home {},
                class: "text-green-700 hover:text-green-900 font-bold flex items-center gap-2 mb-4 transition-colors",
                i { class: "fa-solid fa-arrow-left" }
                "Tillbaka till start"
            }
            h1 { class: "text-3xl font-black mb-8", "Mina Favoriter" }

            match &*fav_resource.read() {
                None => rsx! {
                    p { class: "text-gray-400 animate-pulse", "Laddar favoriter..." }
                },
                Some(Err(e)) => rsx! {
                    p { class: "text-red-400 text-sm", "Fel: {e}" }
                },
                Some(Ok(products)) if products.is_empty() => rsx! {
                    div { class: "text-center py-20 bg-white rounded-2xl shadow-sm border border-gray-100",
                        i { class: "fa-regular fa-heart text-6xl text-gray-200 mb-4" }
                        p { class: "text-gray-500 text-xl", "Du har inga sparade produkter här!" }
                    }
                },
                Some(Ok(products)) => rsx! {
                    div { class: "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-6",
                        for p in products.iter() {
                            ProductCard {
                                id: p.id.get(),
                                name: p.name.clone(),
                                price: p.price.to_f64().unwrap_or_default(),
                                comparison_price: format!("{:.2} kr", p.price),
                                image_url: p.thumbnail.to_string(),
                            }
                        }
                    }
                },
            }
        }
    }
}