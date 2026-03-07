use dioxus::prelude::*;
use crate::database::products::ProductInfo as Product;
use crate::components::product_card::ProductCard;
use rust_decimal::prelude::ToPrimitive;

/// Home page..
#[component]
pub fn Home() -> Element {
    // TODO(db): Ersätt get_fake_products() med ett API-anrop
    // TODO(db): Lägg till paginering eller lazy-loading när produktmängd växer
    let products: Vec<Product> = vec![];

    rsx! {
        div { class: "min-h-screen bg-gray-50",
            main { class: "container mx-auto p-4 py-8",
                div { class: "mb-6",
                    h1 { class: "text-2xl font-bold text-gray-800", "Välkommen till boop!" }
                    p { class: "text-gray-600", "Vi är definitivt inte coop" }
                }

                div { class: "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6",

                    // loopa genom produkterna som läggs till
                    for p in products {
                        ProductCard {
                            id: p.id.get(),
                            name: p.name.clone(),
                            price: p.price.to_f64().unwrap_or_default(),
                            comparison_price: format!("{:.2} kr", p.price),
                            image_url: p.gallery.first().cloned().unwrap_or_default(),
                        }
                    }
                }
            }
        }
    }
}
