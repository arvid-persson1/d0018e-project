use crate::components::ProductCard;

use crate::database::products::newest_products;
use dioxus::prelude::*;

/// Home page..
#[allow(
    clippy::option_if_let_else,
    reason = "rsx! macro incompatible with map_or_else"
)]
#[component]
pub fn Home() -> Element {
    // TODO(db): Ersätt get_fake_products() med ett API-anrop
    // TODO(db): Lägg till paginering eller lazy-loading när produktmängd växer
    let products =
        use_resource(|| async move { newest_products(None, 20, 0).await.unwrap_or_default() });

    rsx! {
        div { class: "min-h-screen bg-gray-50",
            main { class: "container mx-auto p-4 py-8",
                div { class: "mb-6",
                    h1 { class: "text-2xl font-bold text-gray-800", "Välkommen till boop!" }
                    p { class: "text-gray-600", "Vi är definitivt inte coop" }
                }

                match &*products.read() {
                    None => rsx! {
                        div { class: "flex justify-center py-20",
                            p { class: "text-gray-400 text-lg", "Laddar produkter..." }
                        }
                    },
                    Some(items) => rsx! {
                        div { class: "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6",
                            for p in items.iter() {
                                ProductCard {
                                    id: p.id,
                                    name: p.name.clone(),
                                    price: p.price,
                                    comparison_price: p.amount_per_unit.to_string().into(),
                                    image_url: p.thumbnail.clone(),
                                }
                            }
                        }
                    },
                }
            }
        }
    }
}
