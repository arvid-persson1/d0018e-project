use dioxus::prelude::*;

use crate::components::productCard::ProductCard; 

#[component]
pub fn Home() -> Element {
    rsx! {
        div { class: "min-h-screen bg-gray-50",

            main { class: "container mx-auto p-4 py-8",
                div { class: "mb-6",
                    h1 { class: "text-2xl font-bold text-gray-800", "Välkommen till boop!" }
                    p { class: "text-gray-600", "Vi är definitivt inte coop" }
                }

                div { class: "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6",

                    ProductCard {
                        name: "test produkt".to_string(),
                        price: 24.90,
                        comparison_price: "30 kr/kg".to_string(),
                        image_url: "https://via.placeholder.com".to_string(),
                    }
                }
            }
        }
    }
}