use dioxus::prelude::*;
use crate::state::GlobalState;
// class for5 a product card
#[derive(Props, Clone, PartialEq)]
pub struct ProductProps {
    pub name: String,
    pub price: f64,
    pub image_url: String,
    pub comparison_price: String,
}

#[component]
pub fn ProductCard(props: ProductProps) -> Element {
    let mut count = use_signal(|| 0);
    let mut is_favorite = use_signal(|| false);
    let mut global_state = use_context::<Signal<GlobalState>>();

    let heart_class = if is_favorite() { "text-red-500" } else { "text-gray-400 hover:text-red-500" };

    rsx! {
        div { class: "bg-white border border-gray-200 rounded-lg shadow-sm hover:shadow-md transition p-4 flex flex-col gap-3 relative",

            img {
                src: "{props.image_url}",
                class: "w-full h-60 w-30 object-contain mb-2",
            }

            div { class: "flex flex-col gap-0.5",
                h3 { class: "font-bold text-lg text-gray-800", "{props.name}" }

                // för specialerbjudande
                //p { class: "text-2xl font-black text-red-600", "{props.price} kr" }
                p { class: "text-2xl font-black text-black-600", "{props.price} kr" }

                p { class: "text-gray-500 text-xs font-medium", "Jfr pris {props.comparison_price}" }
            }

            div { class: "flex items-center gap-2 mt-auto",
                if count() == 0 {
                    button {
                        class: "flex-grow bg-green-700 text-white font-bold py-2 rounded-full hover:bg-green-800 transition flex justify-center items-center gap-2",
                        onclick: move |_| {
                            count += 1;
                            global_state.write().cart_count += 1;
                        },
                        i { class: "fas fa-shopping-cart" }
                    
                    }
                } else {
                    div { class: "flex-grow flex items-center justify-between bg-green-100 rounded-full overflow-hidden",
                        button {
                            onclick: move |_| {
                                count -= 1;
                                global_state.write().cart_count -= 1;
                            },
                            class: "px-4 py-2 bg-green-700 text-white font-bold",
                            i { class: "fas fa-minus" }
                        }
                        span { class: "font-bold text-green-900", "{count}" }
                        button {
                            onclick: move |_| {
                                count += 1;
                                global_state.write().cart_count += 1;
                            },
                            class: "px-4 py-2 bg-green-700 text-white font-bold",
                            i { class: "fas fa-plus" }
                        }
                    }
                }

                button {
                    class: "p-2 transition-colors {heart_class} text-xl",
                    onclick: move |_| {
                        is_favorite.toggle();
                        if is_favorite() {
                            global_state.write().fav_count += 1;
                        } else {
                            global_state.write().fav_count -= 1;
                        }
                    },
                    if is_favorite() {
                        i { class: "fa-solid fa-heart" }
                    } else {
                        i { class: "fa-regular fa-heart" }
                    }
                }
            }
        }
    }
}