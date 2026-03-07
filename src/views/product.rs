use crate::Route;
use crate::database::products::product_info;
use crate::database::{Id, Product as DbProduct};
use crate::state::GlobalState;
use dioxus::prelude::*;

// Class for product page
/// Product page
/// # Arguments
/// * `id` - The product ID to display.
#[allow(clippy::same_name_method, non_snake_case)]
#[component]
pub fn Product(id: i32) -> Element {
    let mut global_state = use_context::<Signal<GlobalState>>();
    let nav = use_navigator();

    let db_id = Id::<DbProduct>::from(id);

    // Form-signaler för recensioner
    let mut text_val = use_signal(String::new);
    let mut selected_rating = use_signal(|| 0_u8);
    let max_chars = 300;

    // TODO(auth): Skicka med inloggad kunds ID: Some(customer_id)
    let product_resource = use_resource(move || async move {
        product_info(None, db_id).await
    });

    let is_favorite = global_state.read().favorites.contains(&id);
    let quantity = global_state.read().cart.iter()
        .find(|i| i.product_id == id)
        .map(|i| i.quantity)
        .unwrap_or(0);

    let heart_class = if is_favorite { "text-red-500" } else { "text-gray-400 hover:text-red-500" };


    // Klass för betyg stjärnor
    let s1 = if selected_rating() >= 1 { "text-yellow-400" } else { "text-gray-300" };
    let s2 = if selected_rating() >= 2 { "text-yellow-400" } else { "text-gray-300" };
    let s3 = if selected_rating() >= 3 { "text-yellow-400" } else { "text-gray-300" };
    let s4 = if selected_rating() >= 4 { "text-yellow-400" } else { "text-gray-300" };
    let s5 = if selected_rating() >= 5 { "text-yellow-400" } else { "text-gray-300" };

    rsx! {
        div { class: "max-w-6xl mx-auto p-4 md:p-8 bg-white",

            // Tillbaka knapp
            button {
                onclick: move |_| {
                    let _unused = nav.push(Route::Home {});
                },
                class: "text-green-700 font-bold mb-4 flex items-center gap-2 hover:underline",
                i { class: "fa-solid fa-arrow-left" }
                "Tillbaka till start"
            }

            // Hantering av databas-resursen
            match &*product_resource.read_unchecked() {
                None => rsx! {
                    div { class: "flex justify-center items-center py-20",
                        p { class: "text-xl font-bold text-gray-400 animate-pulse", "Hämtar produkt från databasen..." }
                    }
                },
                Some(Err(e)) => rsx! {
                    div { class: "text-center p-20",
                        h2 { class: "text-red-500 text-2xl font-black", "Ett fel uppstod" }
                        p { class: "text-gray-500", "{e}" }
                    }
                },
                Some(Ok(product)) => {
                    let formatted_price = format!("{:.2}", product.price).replace('.', ",");
                    let avg_rating = product.rating.rating().unwrap_or(0.0);
                    let rating_count = product.rating.count();
                    let full_stars = avg_rating.round() as usize;

                    // Klona värden för closures
                    let pname = product.name.to_string();
                    let pprice = product.price.to_string().parse::<f64>().unwrap_or(0.0);
                    let pimage = product
                        .gallery
                        .first()
                        .map(|u| u.to_string())
                        .unwrap_or_default();
                    let pname2 = pname.clone();
                    let pimage2 = pimage.clone();
                    rsx! {
                        div { class: "grid grid-cols-1 md:grid-cols-2 gap-12 mb-16",

                            // Bild-sidan
                            div { class: "flex flex-col items-center",
                                div { class: "bg-gray-50 rounded-xl p-8 w-full flex justify-center",
                                    if let Some(img) = product.gallery.first() {
                                        img {
                                            src: "{img}",
                                            class: "max-h-[400px] object-contain shadow-sm",
                                        }
                                    }
                                }

                                // Betyg under bilden
                                div { class: "mt-6 flex flex-col items-center gap-2",
                                    div { class: "flex text-yellow-400 text-xl",
                                        for i in 0..5_usize {
                                            if i < full_stars {
                                                i { class: "fa-solid fa-star" } // TODO(db): Ersätt med set_in_shopping_cart(customer_id, product_id, 1)
                                            } else {
                                                i { class: "fa-regular fa-star" }
                                            }
                                        }
                                    }
                                    span { class: "text-gray-500 text-sm font-medium",
                                        "{avg_rating:.1} av 5 ({rating_count} recensioner)"
                                    }
                                }
                            } // TODO(db): Ersätt med set_in_shopping_cart(customer_id, product_id, quantity+1)

                            // Info-sidan
                            div { class: "flex flex-col justify-start",
                                h1 { class: "text-4xl font-black text-gray-900 mb-2", "{product.name}" }
                                p { class: "text-gray-500 text-lg mb-4", "{product.description}" } // TODO(db): Ersätt med set_in_shopping_cart(customer_id, product_id, quantity+1)

                                // Pris
                                div { class: "border-t border-b py-6 mb-6",
                                    div { class: "text-red-600 font-black text-5xl mb-1", "{formatted_price} kr" } // TODO(db): Ersätt med set_favorite(customer_id, product_id, !is_favorite)
                                    div { class: "text-gray-500 font-bold", "Säljs av {product.vendor_name}" }
                                }

                                // Köp/Favorit knappar
                                div { class: "flex gap-4 items-center h-16",
                                    if quantity == 0 {
                                        button {
                                            class: "flex-grow h-full bg-green-700 text-white rounded-full font-black text-xl hover:bg-green-800 transition-colors shadow-md flex items-center justify-center gap-3",
                                            onclick: move |_| {
                                                global_state.write().add_to_cart(id, pname.clone(), pprice, pimage.clone());
                                            },
                                            i { class: "fa-solid fa-cart-plus" }
                                            "LÄGG I VARUKORG"
                                        }
                                    } else {
                                        div { class: "flex-grow h-full flex items-center justify-between bg-green-100 rounded-full overflow-hidden border-2 border-green-700",
                                            button {
                                                class: "px-8 h-full bg-green-700 text-white font-bold text-2xl",
                                                onclick: move |_| {
                                                    // TODO(db): Ersätt med set_in_shopping_cart(customer_id, product_id, quantity-1)
                                                    global_state.write().set_quantity(id, quantity - 1);
                                                },
                                                i { class: "fas fa-minus" }
                                            }
                                            span { class: "font-black text-2xl text-green-900", "{quantity}" }
                                            button { // TODO(db): Ersätt med set_favorite(customer_id, product_id, !is_favorite) // TODO(db): Ersätt med set_favorite(customer_id, product_id, !is_favorite) // TODO(db): Ersätt med set_favorite(customer_id, product_id, !is_favorite)  TODO(db): Ersätt med set_favorite(customer_id, product_id, !is_favorite) // TODO(db): Ersätt med set_favorite(customer_id, product_id, !is_favorite)  TODO(db): Ersätt med set_favorite(customer_id, product_id, !is_favorite)  TODO(db): Ersätt med set_favorite(customer_id, product_id, !is_favorite)  TODO(db): Ersätt med set_favorite(customer_id, product_id, !is_favorite)
                                                class: "px-8 h-full bg-green-700 text-white font-bold text-2xl",
                                                onclick: move |_| {
                                                    // TODO(db): Ersätt med set_in_shopping_cart(customer_id, product_id, quantity+1)
                                                    global_state.write().set_quantity(id, quantity + 1);
                                                },
                                                i { class: "fas fa-plus" }
                                            }
                                        }
                                    }
                                    button {
                                        class: "h-full px-6 border-2 border-gray-200 rounded-full transition-all {heart_class}",
                                        onclick: move |_| {
                                            // TODO(db): Ersätt med set_favorite(customer_id, product_id, !is_favorite)
                                            let mut s = global_state.write();
                                            if s.favorites.contains(&id) {
                                                s.favorites.retain(|&x| x != id);
                                            } else {
                                                s.favorites.push(id);
                                            }
                                        },
                                        i { class: if is_favorite { "fa-solid fa-heart text-2xl" } else { "fa-regular fa-heart text-2xl" } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Liknande produkter
            div { class: "border-t pt-16 mb-16",
                h2 { class: "text-3xl font-black mb-8 text-gray-900", "Liknande produkter" }
                p { class: "text-gray-400", "Laddas när databasen är kopplad." }
            }

            // Recensions-sektion
            div { class: "max-w-3xl",
                h2 { class: "text-2xl font-black mb-8", "Vad tycker andra kunder?" }
                div { class: "bg-green-50 p-6 rounded-2xl mb-10 border border-green-100",
                    h3 { class: "font-bold text-lg mb-4 text-green-900", "Skriv en recension" }

                    // Stjärnor med dina s1-s5 klasser
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

                    textarea {
                        class: "w-full border-2 border-white rounded-xl p-4 mb-2 focus:border-green-500 outline-none transition-all",
                        rows: "4",
                        maxlength: "{max_chars}",
                        placeholder: "Berätta mer om produkten...",
                        oninput: move |evt| text_val.set(evt.value()),
                    }

                    div { class: "flex justify-between items-center mb-4",
                        span { class: "text-sm text-gray-500", "{text_val().len()} / {max_chars} tecken" }
                        button {
                            class: "bg-green-700 text-white px-8 py-3 rounded-full font-bold hover:bg-green-800 transition shadow-sm disabled:opacity-30 disabled:cursor-not-allowed",
                            disabled: selected_rating() == 0,
                            onclick: move |_| {
                                // TODO(db): Ersätt med create_review(customer_id, product_id, rating, text)
                                println!("Betyg: {}", selected_rating());
                            },
                            "Skicka recension"
                        }
                    }
                }

                // Exempelrecension
                div { class: "space-y-6",
                    div { class: "border-b pb-6",
                        div { class: "flex gap-1 mb-2",
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
