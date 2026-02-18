use dioxus::prelude::*;

use crate::components::product_card::ProductCard;
use crate::fake_data::get_fake_products;

// Class for the home page
#[component]
pub fn Home() -> Element {
    // TODO(db): Ersätt get_fake_products() med ett API-anrop
    // TODO(db): Lägg till paginering eller lazy-loading när produktmängd växer
    let products = get_fake_products();

    rsx! {
        div { class: "min-h-screen bg-gray-50",
            main { class: "container mx-auto p-4 py-8",
                div { class: "mb-6",
                    h1 { class: "text-2xl font-bold text-gray-800", "Välkommen till boop!" }
                    p { class: "text-gray-600", "Vi är definitivt inte coop" }
                }

                div { class: "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6",

                    // loopa genom produkterna som läggs till
                    for p in products.iter() {
                        // TODO(db): ProductCard är samma, bara datan ändras
                        ProductCard {
                            id: p.id,
                            name: p.name.clone(),
                            price: p.price,
                            comparison_price: p.comparison_price.clone(),
                            image_url: p.image_url.clone(),
                        }
                    }
                }
            }
        }
    }
}
