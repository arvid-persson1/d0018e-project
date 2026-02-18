use crate::Route;
use crate::state::GlobalState;
use dioxus::prelude::*;

// class for a product card
#[derive(Props, Clone, PartialEq)]
pub struct ProductProps {
    pub id: i32,
    pub name: String,
    pub price: f64,
    pub image_url: String,
    pub comparison_price: String,
}

#[component]
pub fn ProductCard(props: ProductProps) -> Element {
    let mut global_state = use_context::<Signal<GlobalState>>();

    // kollar favorit
    let is_favorite = global_state.read().favorites.contains(&props.id);

    let product_id = props.id;

    // Pris format
    let formatted_price = format!("{:.2}", props.price).replace('.', ",");
    let formatted_comparison = props.comparison_price.replace('.', ",");

    let heart_class = if is_favorite {
        "text-red-500"
    } else {
        "text-gray-400 hover:text-red-500"
    };

    rsx! {
        div { class: "bg-white border border-gray-200 rounded-lg shadow-sm hover:shadow-md transition p-4 flex flex-col gap-3 relative",

            // länk i bilden för produktsidan
            Link { to: Route::Product { id: props.id },
                img {
                    src: "{props.image_url}",
                    class: "w-full h-60 object-contain mb-2 cursor-pointer hover:opacity-80 transition",
                }
            }

            div { class: "flex flex-col gap-0.5",
                // länk i namnet för produktsidan
                Link { to: Route::Product { id: props.id },
                    h3 { class: "font-bold text-lg text-gray-800 hover:text-green-700 cursor-pointer",
                        "{props.name}"
                    }
                }

                p { class: "text-2xl font-black text-black-600", "{formatted_price} kr" }
                p { class: "text-gray-500 text-xs font-medium", "Jfr pris {formatted_comparison}" }
            }

            // Köpknapp och favoritknapp
            div { class: "flex items-center gap-2 mt-auto",
                if global_state.read().cart_items.iter().filter(|&&id| id == props.id).count() == 0 {
                    button {
                        class: "flex-grow bg-green-700 text-white font-bold py-2 rounded-full hover:bg-green-800 transition flex justify-center items-center gap-2",
                        onclick: move |_| {
                            global_state.write().cart_items.push(product_id);
                        },
                        i { class: "fas fa-shopping-cart" }
                    }
                } else {
                    div { class: "flex-grow flex items-center justify-between bg-green-100 rounded-full overflow-hidden",
                        button {
                            class: "px-4 py-2 bg-green-700 text-white font-bold",
                            onclick: move |_| {
                                let mut state = global_state.write();
                                if let Some(pos) = state.cart_items.iter().position(|&x| x == product_id) {
                                    state.cart_items.remove(pos);
                                }
                            },
                            i { class: "fas fa-minus" }
                        }
                        span { class: "font-bold text-green-900",
                            "{global_state.read().cart_items.iter().filter(|&&id| id == props.id).count()}"
                        }
                        button {
                            class: "px-4 py-2 bg-green-700 text-white font-bold",
                            onclick: move |_| {
                                global_state.write().cart_items.push(product_id);
                            },
                            i { class: "fas fa-plus" }
                        }
                    }
                }

                // Favoritknapp
                button {
                    class: "p-2 transition-colors {heart_class} text-xl",
                    onclick: move |_| {
                        let mut state = global_state.write();
                        if state.favorites.contains(&product_id) {
                            state.favorites.retain(|&x| x != product_id);
                        } else {
                            state.favorites.push(product_id);
                        }
                    },
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
