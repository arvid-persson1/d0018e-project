use crate::Route;
use crate::state::GlobalState;
use dioxus::prelude::*;
 
/// Kundvagns-dropdown som visas när man klickar på kundvagnsknappen.
#[component]
pub fn CartDropdown(on_close: EventHandler<()>) -> Element {
    let mut global_state = use_context::<Signal<GlobalState>>();
    let cart        = global_state.read().cart.clone();
    let total       = global_state.read().cart_total();
    let is_customer = global_state.read().login.as_ref()
        .is_some_and(|l| matches!(l.id, crate::database::LoginId::Customer(_)));
    let customer_id = global_state.read().customer_id();
 
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
                    }
                } else {
                    for item in cart.iter() {
                        div { class: "flex items-center gap-3 p-3 border-b hover:bg-gray-50 transition",
                            img {
                                src: "{item.image_url}",
                                class: "w-14 h-14 object-cover rounded-lg bg-gray-100 shrink-0",
                            }
                            div { class: "flex-grow min-w-0",
                                p { class: "font-semibold text-sm text-gray-900 truncate",
                                    "{item.name}"
                                }
                                p { class: "text-green-700 font-bold text-sm",
                                    "{item.price:.2} kr/st"
                                }
                            }
                            div { class: "flex items-center gap-1 shrink-0",
                                button {
                                    class: "w-7 h-7 rounded-full border flex items-center justify-center hover:bg-gray-100 transition text-sm font-bold text-green-700",
                                    onclick: {
                                        let id = item.product_id;
                                        let qty = item.quantity;
                                        move |_| {
                                            let new_qty = qty.saturating_sub(1);
                                            global_state.write().set_quantity(id, new_qty);
                                            if let Some(cid) = customer_id {
                                                #[allow(unused_results)]
                                                spawn(async move {
                                                    use crate::database::{Id, Product};
                                                    drop(
                                                        crate::database::cart::set_in_shopping_cart(
                                                                cid,
                                                                Id::<Product>::from(id),
                                                                new_qty,
                                                            )
                                                            .await,
                                                    );
                                                });
                                            }
                                        }
                                    },
                                    "−"
                                }
                                span { class: "w-6 text-center text-sm font-bold text-green-600",
                                    "{item.quantity}"
                                }
                                button {
                                    class: "w-7 h-7 rounded-full border flex items-center justify-center hover:bg-gray-100 transition text-sm font-bold text-green-700",
                                    onclick: {
                                        let id = item.product_id;
                                        let qty = item.quantity;
                                        move |_| {
                                            let new_qty = qty + 1;
                                            global_state.write().set_quantity(id, new_qty);
                                            if let Some(cid) = customer_id {
                                                #[allow(unused_results)]
                                                spawn(async move {
                                                    use crate::database::{Id, Product};
                                                    drop(
                                                        crate::database::cart::set_in_shopping_cart(
                                                                cid,
                                                                Id::<Product>::from(id),
                                                                new_qty,
                                                            )
                                                            .await,
                                                    );
                                                });
                                            }
                                        }
                                    },
                                    "+"
                                }
                            }
                            button {
                                class: "ml-1 text-gray-300 hover:text-red-500 transition shrink-0",
                                onclick: {
                                    let id = item.product_id;
                                    move |_| {
                                        global_state.write().remove_from_cart(id);
                                        if let Some(cid) = customer_id {
                                            #[allow(unused_results)]
                                            spawn(async move {
                                                use crate::database::{Id, Product};
                                                drop(
                                                    crate::database::cart::set_in_shopping_cart(
                                                            cid,
                                                            Id::<Product>::from(id),
                                                            0,
                                                        )
                                                        .await,
                                                );
                                            });
                                        }
                                    }
                                },
                                i { class: "fa-solid fa-trash text-sm" }
                            }
                        }
                    }
                }
            }

            // Footer
            if !cart.is_empty() {
                div { class: "p-4 border-t bg-gray-50",
                    div { class: "flex justify-between items-center mb-3",
                        span { class: "font-bold text-gray-700", "Totalt" }
                        span { class: "font-black text-xl text-gray-900", "{total:.2} kr" }
                    }

                    if is_customer {
                        Link {
                            to: Route::Cart {},
                            class: "w-full bg-green-700 text-white py-3 rounded-xl font-black text-center flex items-center justify-center gap-2 hover:bg-green-800 transition",
                            onclick: move |_| on_close.call(()),
                            i { class: "fa-solid fa-lock text-sm" }
                            "Gå till kassan"
                        }
                    } else {
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