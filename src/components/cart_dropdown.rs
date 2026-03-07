use crate::Route;
use crate::state::GlobalState;
use dioxus::prelude::*;

/// Kundvagns-dropdown som visas när man klickar på kundvagnsknappen.
#[component]
pub fn CartDropdown(on_close: EventHandler<()>) -> Element {
    let mut global_state = use_context::<Signal<GlobalState>>();
    // TODO(auth): is_logged_in ska läsas från auth context
    let is_logged_in = global_state.read().auth.user.is_some();
    let cart = global_state.read().cart.clone();
    let total = global_state.read().cart_total();
    let count_text = if cart.len() == 1 { "produkt" } else { "produkter" };

    rsx! {
        // Backdrop
        div { class: "fixed inset-0 z-40", onclick: move |_| on_close.call(()) }

        // Dropdown
        div { class: "absolute right-0 top-full mt-2 w-96 bg-white rounded-2xl shadow-xl border z-50 overflow-hidden",

            // Header
            div { class: "p-4 border-b flex justify-between items-center bg-gray-50",
                h3 { class: "font-black text-gray-900 text-lg", "Kundvagn" }
                if !cart.is_empty() {
                    span { class: "text-xs text-gray-400 font-semibold", "{cart.len()} {count_text}" }
                }
            }

            // Produktlista
            div { class: "max-h-80 overflow-y-auto",
                if cart.is_empty() {
                    div { class: "flex flex-col items-center justify-center py-12 text-gray-400",
                        i { class: "fa-solid fa-basket-shopping text-4xl mb-3" }
                        p { class: "font-semibold", "Kundvagnen är tom" }
                        if !is_logged_in {
                            p { class: "text-xs mt-1 text-center px-4", "Logga in för att handla" }
                        }
                    }
                } else {
                    for item in cart.iter() {
                        div { class: "flex items-center gap-3 p-3 border-b hover:bg-gray-50 transition",
                            // Produktbild
                            img {
                                src: "{item.image_url}",
                                class: "w-14 h-14 object-cover rounded-lg bg-gray-100 shrink-0",
                            }

                            // Namn och pris
                            div { class: "flex-grow min-w-0",
                                p { class: "font-semibold text-sm text-gray-900 truncate",
                                    "{item.name}"
                                }
                                p { class: "text-green-700 font-bold text-sm",
                                    "{item.price:.2} kr/st"
                                }
                            }

                            // Antal-kontroller
                            div { class: "flex items-center gap-1 shrink-0",
                                button {
                                    class: "w-7 h-7 rounded-full border flex items-center justify-center hover:bg-gray-100 transition text-sm font-bold",
                                    onclick: {
                                        let id = item.product_id;
                                        let qty = item.quantity;
                                        move |_| global_state.write().set_quantity(id, qty.saturating_sub(1))
                                    },
                                    "−"
                                }
                                span { class: "w-6 text-center text-sm font-bold", "{item.quantity}" }
                                button {
                                    class: "w-7 h-7 rounded-full border flex items-center justify-center hover:bg-gray-100 transition text-sm font-bold",
                                    onclick: {
                                        let id = item.product_id;
                                        let qty = item.quantity;
                                        move |_| global_state.write().set_quantity(id, qty + 1)
                                    },
                                    "+"
                                }
                            }

                            // Ta bort
                            button {
                                class: "ml-1 text-gray-300 hover:text-red-500 transition shrink-0",
                                onclick: {
                                    let id = item.product_id;
                                    move |_| global_state.write().remove_from_cart(id)
                                },
                                i { class: "fa-solid fa-trash text-sm" }
                            }
                        }
                    }
                }
            }

            // Footer med totalt och checkout
            if !cart.is_empty() {
                div { class: "p-4 border-t bg-gray-50",
                    div { class: "flex justify-between items-center mb-3",
                        span { class: "font-bold text-gray-700", "Totalt" }
                        span { class: "font-black text-xl text-gray-900", "{total:.2} kr" }
                    }

                    if is_logged_in {
                        // TODO(auth): När inloggad, koppla checkout till DB
                        Link {
                            to: Route::Home {}, // TODO: Route::Checkout när den finns
                            class: "w-full bg-green-700 text-white py-3 rounded-xl font-black text-center flex items-center justify-center gap-2 hover:bg-green-800 transition",
                            onclick: move |_| on_close.call(()),
                            i { class: "fa-solid fa-lock text-sm" }
                            "Gå till kassan"
                        }
                    } else {
                        // Ej inloggad — visa disabled knapp + uppmaning
                        div { class: "space-y-2",
                            button {
                                class: "w-full bg-gray-200 text-gray-400 py-3 rounded-xl font-black cursor-not-allowed flex items-center justify-center gap-2",
                                disabled: true,
                                i { class: "fa-solid fa-lock text-sm" }
                                "Gå till kassan"
                            }
                            p { class: "text-center text-xs text-gray-500",
                                "Du måste "
                                Link {
                                    to: Route::Login {},
                                    class: "text-green-700 font-bold hover:underline",
                                    onclick: move |_| on_close.call(()),
                                    "logga in"
                                }
                                " för att slutföra köpet"
                            }
                        }
                    }
                }
            }
        }
    }
}
