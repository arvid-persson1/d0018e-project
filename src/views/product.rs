use crate::Route;
use crate::database::products::product_info;
use crate::database::{Id, Product};
use crate::state::GlobalState;
use dioxus::prelude::*;

#[component]
pub fn ProductPage(id: Id<Product>) -> Element {
    let mut global_state = use_context::<Signal<GlobalState>>();
    let nav = use_navigator();
    let mut text_val = use_signal(|| String::new());
    let mut selected_rating = use_signal(|| 0u8);
    let max_chars = 300;

    // TODO(db): Skicka med inloggad kunds ID när login finns: Some(customer_id)
    let product_resource = use_resource(move || async move { product_info(None, id).await });


    let count = global_state.read().cart_count(id);
    let is_favorite = global_state.read().favorites.contains(&id);
    let heart_class = if is_favorite { "text-red-500" } else { "text-gray-400 hover:text-red-500" };

    rsx! {
        div { class: "max-w-6xl mx-auto p-4 md:p-8 bg-white",

            button {
                onclick: move |_| {
                    drop(nav.push(Route::Home {}));
                },
                class: "text-green-700 font-bold mb-4 flex items-center gap-2 hover:underline",
                i { class: "fa-solid fa-arrow-left" }
                "Tillbaka till start"
            }

            match &*product_resource.read() {
                None => rsx! {
                    p { class: "text-gray-400 text-lg py-20 text-center", "Laddar produkt..." }
                },
                Some(Err(_)) => rsx! {
                    p { class: "text-red-500 text-lg py-20 text-center", "Kunde inte ladda produkten." }
                },
                Some(Ok(product)) => {
                    let formatted_price = format!("{:.2}", product.price).replace('.', ",");
                    let avg_rating = product.rating.rating().unwrap_or(0.0);
                    let rating_count = product.rating.count();
                    let full_stars = avg_rating.round() as usize;

                    rsx! {
                        // Produkt info
                        div { class: "grid grid-cols-1 md:grid-cols-2 gap-12 mb-16",

                            // Bild
                            div { class: "flex flex-col items-center",
                                div { class: "bg-gray-50 rounded-xl p-8 w-full flex justify-center",
                                    if let Some(img) = product.gallery.first() {
                                        img {
                                            src: "{img}",
                                            class: "max-h-[400px] object-contain shadow-sm",
                                        }
                                    }
                                }
                                div { class: "mt-6 flex flex-col items-center gap-2",
                                    div { class: "flex text-yellow-400 text-xl",
                                        for i in 0..5usize {
                                            if i < full_stars {
                                                i { class: "fa-solid fa-star" }
                                            } else {
                                                i { class: "fa-regular fa-star" }
                                            }
                                        }
                                    }
                                    span { class: "text-gray-500 text-sm font-medium",
                                        "{avg_rating:.1} av 5 ({rating_count} recensioner)"
                                    }
                                }
                            }

                            // Info
                            div { class: "flex flex-col justify-start",
                                h1 { class: "text-4xl font-black text-gray-900 mb-2", "{product.name}" }
                                p { class: "text-gray-500 text-lg mb-4", "{product.description}" }

                                div { class: "border-t border-b py-6 mb-6",
                                    div { class: "text-red-600 font-black text-5xl mb-1", "{formatted_price} kr" }
                                    div { class: "text-gray-500 font-bold", "Säljs av {product.vendor_name}" }
                                }

                                // Varukorg + favorit knappar
                                div { class: "flex gap-4 items-center h-16",
                                    if count == 0 {
                                        button {
                                            class: "flex-grow h-full bg-green-700 text-white rounded-full font-black text-xl hover:bg-green-800 transition-colors shadow-md flex items-center justify-center gap-3",
                                            onclick: move |_| global_state.write().add_to_cart(id),
                                            i { class: "fa-solid fa-cart-plus" }
                                            "LÄGG I VARUKORG"
                                        }
                                    } else {
                                        div { class: "flex-grow h-full flex items-center justify-between bg-green-100 rounded-full overflow-hidden border-2 border-green-700",
                                            button {
                                                class: "px-8 h-full bg-green-700 text-white font-bold text-2xl",
                                                onclick: move |_| global_state.write().remove_from_cart(id),
                                                i { class: "fas fa-minus" }
                                            }
                                            span { class: "font-black text-2xl text-green-900", "{count}" }
                                            button {
                                                class: "px-8 h-full bg-green-700 text-white font-bold text-2xl",
                                                onclick: move |_| global_state.write().add_to_cart(id),
                                                i { class: "fas fa-plus" }
                                            }
                                        }
                                    }

                                    button {
                                        class: "h-full px-6 border-2 border-gray-200 rounded-full transition-all {heart_class}",
                                        onclick: move |_| global_state.write().toggle_favorite(id),
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
                            p { class: "text-gray-400", "Laddas när databasen är kopplad." }
                        }

                        // Recensioner
                        div { class: "max-w-3xl",
                            h2 { class: "text-2xl font-black mb-8", "Vad tycker andra kunder?" }

                            div { class: "bg-green-50 p-6 rounded-2xl mb-10 border border-green-100",
                                h3 { class: "font-bold text-lg mb-4 text-green-900", "Skriv en recension" }

                                div { class: "flex gap-2 mb-4",
                                    for n in 1u8..=5 {
                                        i {
                                            class: if selected_rating() >= n { "fa-solid fa-star text-2xl cursor-pointer text-yellow-400" } else { "fa-solid fa-star text-2xl cursor-pointer text-gray-300" },
                                            onclick: move |_| selected_rating.set(n),
                                        }
                                    }
                                }

                                textarea {
                                    class: "w-full border-2 border-white rounded-xl p-4 mb-2 focus:border-green-500 outline-none transition-all",
                                    rows: "4",
                                    maxlength: "{max_chars}",
                                    placeholder: "Berätta mer om produkten...",
                                    oninput: move |evt| text_val.set(evt.value()),
                                }

                                div { class: "flex justify-end mb-4",
                                    span { class: "text-sm text-gray-500", "{text_val().len()} / {max_chars} tecken" }
                                }

                                // TODO(db): Anropa create_review() med produkt-ID och kund-ID
                                button {
                                    class: "bg-green-700 text-white px-8 py-3 rounded-full font-bold hover:bg-green-800 transition shadow-sm disabled:opacity-30 disabled:cursor-not-allowed",
                                    disabled: selected_rating() == 0,
                                    "Skicka recension"
                                }
                            }

                            // TODO(db): Ersätt med product_reviews() från databasen
                            div { class: "space-y-6",
                                div { class: "border-b pb-6",
                                    div { class: "flex gap-1 mb-2",
                                        for _ in 0..5 {
                                            i { class: "fa-solid fa-star text-yellow-400 text-sm" }
                                        }
                                    }
                                    div { class: "flex items-center gap-2 mb-2",
                                        span { class: "font-bold text-gray-900", "Exempelrecension" }
                                        span { class: "text-gray-400 text-sm", "• Verifierat köp" }
                                    }
                                    p { class: "text-gray-600 leading-relaxed",
                                        "Recensioner visas här när databasen är kopplad."
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}