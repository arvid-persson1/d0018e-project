use crate::Route;
use crate::components::product_card::ProductCard;
use crate::fake_data::get_fake_products;
use crate::state::GlobalState;
use dioxus::prelude::*;

// Class for product page
#[component]
pub fn Product(id: i32) -> Element {
    let mut global_state = use_context::<Signal<GlobalState>>();
    let is_favorite = global_state.read().favorites.contains(&id);
    let product_id = id;
    let nav = use_navigator();

    let cart_count = global_state
        .read()
        .cart_items
        .iter()
        .filter(|&&item_id| item_id == id)
        .count();

    // Recension signal
    let mut text_val = use_signal(|| "".to_string());
    let mut selected_rating = use_signal(|| 0);
    let max_chars = 300;

    // Hämta in produkt data
    let products = get_fake_products();
    let product = products
        .iter()
        .find(|p| p.id == id)
        .cloned()
        .unwrap_or(products[0].clone());

    // Prisformatering
    let formatted_price = format!("{:.2}", product.price).replace('.', ",");
    let formatted_jfr = product.comparison_price.replace('.', ",");
    let heart_class = if is_favorite {
        "text-red-500"
    } else {
        "text-gray-400 hover:text-red-500"
    };

    // Klass för betyg stjärnor
    let s1 = if selected_rating() >= 1 {
        "text-yellow-400"
    } else {
        "text-gray-300"
    };
    let s2 = if selected_rating() >= 2 {
        "text-yellow-400"
    } else {
        "text-gray-300"
    };
    let s3 = if selected_rating() >= 3 {
        "text-yellow-400"
    } else {
        "text-gray-300"
    };
    let s4 = if selected_rating() >= 4 {
        "text-yellow-400"
    } else {
        "text-gray-300"
    };
    let s5 = if selected_rating() >= 5 {
        "text-yellow-400"
    } else {
        "text-gray-300"
    };

    rsx! {
        div { class: "max-w-6xl mx-auto p-4 md:p-8 bg-white",

            // Tillbaka knapp
            button {
                onclick: move |_| {
                    nav.push(Route::Home {});
                },
                class: "text-green-700 font-bold mb-4 flex items-center gap-2 hover:underline",
                i { class: "fa-solid fa-arrow-left" }
                "Tillbaka till start"
            }

            div { class: "grid grid-cols-1 md:grid-cols-2 gap-12 mb-16",

                // Bild-sidan
                div { class: "flex flex-col items-center",
                    div { class: "bg-gray-50 rounded-xl p-8 w-full flex justify-center",
                        img {
                            src: "{product.image_url}",
                            class: "max-h-[400px] object-contain shadow-sm",
                        }
                    }
                    // Betyg under bilden
                    div { class: "mt-6 flex flex-col items-center gap-2",
                        div { class: "flex text-yellow-400 text-xl",
                            for _ in 0..4 {
                                i { class: "fa-solid fa-star" }
                            }
                            i { class: "fa-regular fa-star" }
                        }
                        // x av 5 där x är medel-betyget
                        span { class: "text-gray-500 text-sm font-medium", "4 av 5 (x recensioner)" }
                    }
                }

                // Info
                div { class: "flex flex-col justify-start",
                    h1 { class: "text-4xl font-black text-gray-900 mb-2", "{product.name}" }
                    p { class: "text-gray-500 text-lg mb-4", "{product.description}" }

                    // pris
                    div { class: "border-t border-b py-6 mb-6",
                        div { class: "text-red-600 font-black text-5xl mb-1", "{formatted_price} kr" }
                        div { class: "text-gray-500 font-bold", "Jfr pris {formatted_jfr}" }
                    }

                    // Varukorg knappen
                    div { class: "flex gap-4 items-center h-16",
                        if cart_count == 0 {
                            button {
                                class: "flex-grow h-full bg-green-700 text-white rounded-full font-black text-xl hover:bg-green-800 transition-colors shadow-md flex items-center justify-center gap-3",
                                onclick: move |_| {
                                    global_state.write().cart_items.push(id);
                                },
                                i { class: "fa-solid fa-cart-plus" }
                                "LÄGG I VARUKORG"
                            }
                        } else {
                            div { class: "flex-grow h-full flex items-center justify-between bg-green-100 rounded-full overflow-hidden border-2 border-green-700",
                                button {
                                    class: "px-8 h-full bg-green-700 text-white font-bold text-2xl",
                                    onclick: move |_| {
                                        let mut state = global_state.write();
                                        if let Some(pos) = state.cart_items.iter().position(|&x| x == id) {
                                            state.cart_items.remove(pos);
                                        }
                                    },
                                    i { class: "fas fa-minus" }
                                }

                                span { class: "font-black text-2xl text-green-900", "{cart_count}" }

                                button {
                                    class: "px-8 h-full bg-green-700 text-white font-bold text-2xl",
                                    onclick: move |_| {
                                        global_state.write().cart_items.push(id);
                                    },
                                    i { class: "fas fa-plus" }
                                }
                            }
                        }

                        // favorit knapp
                        button {
                            class: "h-full px-6 border-2 border-gray-200 rounded-full transition-all {heart_class}",
                            onclick: move |_| {
                                let mut state = global_state.write();
                                if state.favorites.contains(&product_id) {
                                    state.favorites.retain(|&x| x != product_id);
                                } else {
                                    state.favorites.push(product_id);
                                }
                            },
                            if is_favorite {
                                i { class: "fa-solid fa-heart text-2xl" }
                            } else {
                                i { class: "fa-regular fa-heart text-2xl" }
                            }
                        }
                    }
                }
            }

            // Liknande produkter
            div { class: "border-t pt-16 mb-16",
                h2 { class: "text-3xl font-black mb-8 text-gray-900", "Liknande produkter" }
                div { class: "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-6",
                    for p in products.iter().filter(|p| p.id != id).take(4) {
                        ProductCard {
                            id: p.id,
                            name: p.name.clone(),
                            price: p.price,
                            image_url: p.image_url.clone(),
                            comparison_price: p.comparison_price.clone(),
                        }
                    }
                }
            }

            // Recensioner
            div { class: "max-w-3xl",
                h2 { class: "text-2xl font-black mb-8", "Vad tycker andra kunder?" }
                div { class: "bg-green-50 p-6 rounded-2xl mb-10 border border-green-100",
                    h3 { class: "font-bold text-lg mb-4 text-green-900", "Skriv en recension" }

                    // Stjärnor för betyg
                    div { class: "flex gap-2 mb-4",
                        i {
                            class: "fa-solid fa-star text-2xl cursor-pointer {s1}",
                            onclick: move |_| selected_rating.set(1),
                        }
                        i {
                            class: "fa-solid fa-star text-2xl cursor-pointer {s2}",
                            onclick: move |_| selected_rating.set(2),
                        }
                        i {
                            class: "fa-solid fa-star text-2xl cursor-pointer {s3}",
                            onclick: move |_| selected_rating.set(3),
                        }
                        i {
                            class: "fa-solid fa-star text-2xl cursor-pointer {s4}",
                            onclick: move |_| selected_rating.set(4),
                        }
                        i {
                            class: "fa-solid fa-star text-2xl cursor-pointer {s5}",
                            onclick: move |_| selected_rating.set(5),
                        }
                    }

                    // textruta
                    textarea {
                        class: "w-full border-2 border-white rounded-xl p-4 mb-2 focus:border-green-500 outline-none transition-all",
                        rows: "4",
                        maxlength: "{max_chars}",
                        placeholder: "Berätta mer om produkten...",
                        oninput: move |evt| text_val.set(evt.value()),
                    }

                    // max tecken
                    div { class: "flex justify-end mb-4",
                        span { class: "text-sm text-gray-500", "{text_val().len()} / {max_chars} tecken" }
                    }

                    // Skicka in recension knapp, aktiveras inte föränn man har satt betyg!
                    button {
                        class: "bg-green-700 text-white px-8 py-3 rounded-full font-bold hover:bg-green-800 transition shadow-sm disabled:opacity-30 disabled:cursor-not-allowed",
                        disabled: selected_rating() == 0,
                        onclick: move |_| {
                            println!("Betyg: {}", selected_rating());
                        },
                        "Skicka recension"
                    }
                }
                // EXEMPEL
                div { class: "space-y-6",
                    div { class: "border-b pb-6",
                        div { class: "flex gap-1 mb-2",
                            // Hur mycket betyg har getts... i detta fall 5/5
                            for _ in 0..5 {
                                i { class: "fa-solid fa-star text-yellow-400 text-sm" }
                            }
                        }
                        div { class: "flex items-center gap-2 mb-2",
                            span { class: "font-bold text-gray-900", "Namn Efternamn" }
                            span { class: "text-gray-400 text-sm", "• Verifierat köp" }
                        }
                        p { class: "text-gray-600 leading-relaxed",
                            "Här är en exempel recension för att veta hur den ska se ut :))"
                        }
                    }
                }
            }
        }
    }
}
