use crate::Route;
use crate::database::{Id, Product};
use crate::state::GlobalState;
use dioxus::prelude::*;
use rust_decimal::Decimal;

// class for a product card
#[allow(missing_docs)]
#[derive(Props, Debug, Clone, PartialEq)]

/// props for productCard
pub struct ProductCardProps {
    /// The product ID
    pub id: Id<Product>,
    /// The product name
    pub name: Box<str>,
    /// The base price
    pub price: Decimal,
    /// URL to the product image
    pub image_url: Box<str>,
    /// Comparison price string
    pub comparison_price: Box<str>,
}

/// Product card component
#[component]
pub fn ProductCard(props: ProductCardProps) -> Element {
    let mut global_state = use_context::<Signal<GlobalState>>();

    let id = props.id;
    let count = global_state.read().cart_count(id);
    let is_favorite = global_state.read().favorites.contains(&id);

    let formatted_price = format!("{:.2}", props.price).replace('.', ",");
    let formatted_comparison = props.comparison_price.replace('.', ",");

    let heart_class = if is_favorite {
        "text-red-500"
    } else {
        "text-gray-400 hover:text-red-500"
    };

    rsx! {
        div { class: "bg-white border border-gray-200 rounded-lg shadow-sm hover:shadow-md transition p-4 flex flex-col gap-3 relative",

            Link { to: Route::Product { id },
                img {
                    src: "{props.image_url}",
                    class: "w-full h-60 object-contain mb-2 cursor-pointer hover:opacity-80 transition",
                }
            }

            div { class: "flex flex-col gap-0.5",
                Link { to: Route::Product { id },
                    h3 { class: "font-bold text-lg text-gray-800 hover:text-green-700 cursor-pointer",
                        "{props.name}"
                    }
                }
                p { class: "text-2xl font-black text-black-600", "{formatted_price} kr" }
                p { class: "text-gray-500 text-xs font-medium", "Jfr pris {formatted_comparison}" }
            }

            div { class: "flex items-center gap-2 mt-auto",
                if count == 0 {
                    button {
                        class: "flex-grow bg-green-700 text-white font-bold py-2 rounded-full hover:bg-green-800 transition flex justify-center items-center gap-2",
                        onclick: move |_| global_state.write().add_to_cart(id),
                        i { class: "fas fa-shopping-cart" }
                    }
                } else {
                    div { class: "flex-grow flex items-center justify-between bg-green-100 rounded-full overflow-hidden",
                        button {
                            class: "px-4 py-2 bg-green-700 text-white font-bold",
                            onclick: move |_| global_state.write().remove_from_cart(id),
                            i { class: "fas fa-minus" }
                        }
                        span { class: "font-bold text-green-900", "{count}" }
                        button {
                            class: "px-4 py-2 bg-green-700 text-white font-bold",
                            onclick: move |_| global_state.write().add_to_cart(id),
                            i { class: "fas fa-plus" }
                        }
                    }
                }

                button {
                    class: "p-2 transition-colors {heart_class} text-xl",
                    onclick: move |_| global_state.write().toggle_favorite(id),
                    if is_favorite {
                        i { class: "fa-solid fa-heart" }
                    } else {
                        i { class: "fa-regular fa-heart" }
                    }
                }
            }
        }
    }
}
