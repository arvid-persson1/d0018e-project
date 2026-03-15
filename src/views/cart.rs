#![allow(non_snake_case)]
use crate::Route;
use crate::database::cart::{cart_products, checkout, set_in_shopping_cart};
use crate::state::GlobalState;
use dioxus::prelude::*;
 
/// Kundvagnssida med checkout.
#[component]
pub fn CartPage() -> Element {
    let mut global_state = use_context::<Signal<GlobalState>>();
 
    let mut checkout_error = use_signal(|| None::<String>);
    let mut checkout_done  = use_signal(|| false);
    let mut checking_out   = use_signal(|| false);
 
    // Läs reaktivt från global state
    let auth_loading = global_state.read().auth_loading;
    let customer_id  = global_state.read().customer_id();
 
    // Hämta cart från DB — triggas om när customer_id eller auth_loading ändras
    let mut cart_resource = use_resource(move || async move {
        let auth_loading = global_state.read().auth_loading;
        let cid = global_state.read().customer_id();
        if !auth_loading {
            if let Some(cid) = cid {
                cart_products(cid).await.ok()
            } else {
                None
            }
        } else {
            None
        }
    });
 
    let cart_read  = cart_resource.read();
    let cart_loading = cart_read.is_none() || auth_loading;
    let cart_tuple: Option<(Box<[crate::database::cart::CartProduct]>, time::PrimitiveDateTime)> =
        cart_read.as_ref().and_then(|r| r.clone());
    let cart_empty = cart_tuple.as_ref().map(|(p, _)| p.is_empty()).unwrap_or(false);
 
    rsx! {
        div { class: "min-h-screen bg-gray-50",
            div { class: "max-w-4xl mx-auto p-6",
                Link {
                    to: Route::Home {},
                    class: "text-green-700 hover:text-green-900 font-bold flex items-center gap-2 mb-6 transition-colors",
                    i { class: "fa-solid fa-arrow-left" }
                    "Fortsätt handla"
                }
                h1 { class: "text-3xl font-black text-gray-900 mb-8",
                    i { class: "fa-solid fa-basket-shopping text-green-700 mr-3" }
                    "Kundvagn"
                }

                if checkout_done() {
                    div { class: "bg-green-50 border border-green-200 rounded-2xl p-8 text-center",
                        i { class: "fa-solid fa-circle-check text-5xl text-green-600 mb-4" }
                        h2 { class: "text-2xl font-black text-green-900 mb-2",
                            "Tack för din beställning!"
                        }
                        p { class: "text-green-700 mb-6", "Din order har lagts." }
                        Link {
                            to: Route::Home {},
                            class: "bg-green-700 text-white font-black px-8 py-3 rounded-full hover:bg-green-800 transition",
                            "Fortsätt handla"
                        }
                    }
                } else if cart_loading {
                    div { class: "flex items-center justify-center py-20",
                        i { class: "fa-solid fa-spinner fa-spin text-3xl text-green-600" }
                    }
                } else if let Some(cid) = customer_id {
                    if cart_empty {
                        div { class: "text-center py-20 bg-white rounded-2xl shadow-sm border border-gray-100",
                            i { class: "fa-solid fa-basket-shopping text-6xl text-gray-200 mb-4" }
                            p { class: "text-gray-500 text-xl", "Kundvagnen är tom" }
                            Link {
                                to: Route::Home {},
                                class: "inline-block mt-6 bg-green-700 text-white font-black px-8 py-3 rounded-full hover:bg-green-800 transition",
                                "Börja handla"
                            }
                        }
                    } else if let Some((products, cart_time)) = cart_tuple {
                        {
                            let total: rust_decimal::Decimal = products
                                .iter()
                                .map(|p| {
                                    let ep = p
                                        .special_offer_deal
                                        .as_ref()
                                        .map(|d| d.average_discount(p.price))
                                        .unwrap_or(p.price);
                                    ep * rust_decimal::Decimal::from(p.count.get())
                                })
                                .sum();
                            rsx! {
                                div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6",
                                    div { class: "lg:col-span-2 space-y-3",
                                        for product in products.iter() {
                                            div { class: "bg-white rounded-2xl shadow-sm p-4 flex items-center gap-4",
                                                Link {
                                                    to: Route::Product {
                                                        id: product.id.into(),
                                                    },
                                                    img {
                                                        src: "{product.thumbnail}",
                                                        class: "w-20 h-20 object-cover rounded-xl bg-gray-100 shrink-0",
                                                        alt: "{product.name}",
                                                    }
                                                }
                                                div { class: "flex-grow min-w-0",
                                                    Link {
                                                        to: Route::Product {
                                                            id: product.id.into(),
                                                        },
                                                        p { class: "font-bold text-gray-900 hover:text-green-700 transition truncate",
                                                            "{product.name}"
                                                        }
                                                    }
                                                    if let Some(deal) = &product.special_offer_deal {
                                                        p { class: "text-green-600 font-black text-sm",
                                                            "{deal.average_discount(product.price):.2} kr"
                                                            span { class: "line-through text-gray-400 ml-1 text-xs font-normal",
                                                                "{product.price:.2} kr"
                                                            }
                                                        }
                                                    } else {
                                                        p { class: "text-green-700 font-black text-sm", "{product.price:.2} kr/st" }
                                                    }
                                                    if product.in_stock < 5 && product.in_stock > 0 {
                                                        p { class: "text-orange-500 text-xs mt-1", "Endast {product.in_stock} kvar!" }
                                                    }
                                                }
                                                div { class: "flex items-center gap-2 shrink-0",
                                                    button {
                                                        class: "w-8 h-8 rounded-full border-2 border-gray-200 flex items-center justify-center hover:border-green-500 hover:text-green-700 transition font-bold",
                                                        onclick: {
                                                            let pid = product.id;
                                                            let count = product.count.get();
                                                            move |_| {
                                                                let new_count = count.saturating_sub(1);
                                                                global_state.write().set_quantity(pid.get(), new_count);
                                                                #[allow(unused_results)]
                                                                spawn(async move {
                                                                    drop(set_in_shopping_cart(cid, pid, new_count).await);
                                                                    cart_resource.restart();
                                                                });
                                                            }
                                                        },
                                                        "−"
                                                    }
                                                    span { class: "w-8 text-center font-black text-gray-900", "{product.count}" }
                                                    button {
                                                        class: "w-8 h-8 rounded-full border-2 border-gray-200 flex items-center justify-center hover:border-green-500 hover:text-green-700 transition font-bold",
                                                        onclick: {
                                                            let pid = product.id;
                                                            let count = product.count.get();
                                                            move |_| {
                                                                let new_count = count + 1;
                                                                global_state.write().set_quantity(pid.get(), new_count);
                                                                #[allow(unused_results)]
                                                                spawn(async move {
                                                                    drop(set_in_shopping_cart(cid, pid, new_count).await);
                                                                    cart_resource.restart();
                                                                });
                                                            }
                                                        },
                                                        "+"
                                                    }
                                                }
                                                button {
                                                    class: "text-gray-300 hover:text-red-500 transition ml-2",
                                                    onclick: {
                                                        let pid = product.id;
                                                        move |_| {
                                                            global_state.write().remove_from_cart(pid.get());
                                                            #[allow(unused_results)]
                                                            spawn(async move {
                                                                drop(set_in_shopping_cart(cid, pid, 0).await);
                                                                cart_resource.restart();
                                                            });
                                                        }
                                                    },
                                                    i { class: "fa-solid fa-trash" }
                                                }
                                            }
                                        }
                                    }

                                    div { class: "bg-white rounded-2xl shadow-sm p-6 h-fit",
                                        h2 { class: "font-black text-gray-900 text-lg mb-4", "Ordersammanfattning" }
                                        div { class: "space-y-2 mb-4",
                                            for product in products.iter() {
                                                div { class: "flex justify-between text-sm text-gray-600",
                                                    span { class: "truncate mr-2", "{product.name} × {product.count}" }
                                                    span { class: "shrink-0 font-semibold",
                                                        {
                                                            let p = product
                                                                .special_offer_deal
                                                                .as_ref()
                                                                .map(|d| d.average_discount(product.price))
                                                                .unwrap_or(product.price);
                                                            let line = p * rust_decimal::Decimal::from(product.count.get());
                                                            format!("{line:.2} kr")
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        div { class: "border-t pt-4 flex justify-between items-center mb-6",
                                            span { class: "font-bold text-gray-700", "Totalt" }
                                            span { class: "font-black text-2xl text-gray-900", "{total:.2} kr" }
                                        }
                                        if let Some(err) = checkout_error() {
                                            p { class: "text-red-500 text-sm mb-3 text-center bg-red-50 p-2 rounded-lg",
                                                "{err}"
                                            }
                                        }
                                        button {
                                            class: if checking_out() { "w-full bg-gray-300 text-gray-500 py-4 rounded-xl font-black text-lg cursor-not-allowed flex items-center justify-center gap-2" } else { "w-full bg-green-700 text-white py-4 rounded-xl font-black text-lg hover:bg-green-800 transition flex items-center justify-center gap-2" },
                                            disabled: checking_out(),
                                            onclick: move |_| {
                                                checking_out.set(true);
                                                checkout_error.set(None);
                                                let items: Vec<crate::database::cart::CheckoutItem> = products
                                                    .iter()
                                                    .map(|p| crate::database::cart::CheckoutItem {
                                                        product: p.id,
                                                        number: p.count,
                                                        special_offer: p.special_offer_id,
                                                        expected_price: p
                                                            .special_offer_deal
                                                            .as_ref()
                                                            .map(|d| {
                                                                d.discounted_price(
                                                                        p.count,
                                                                        p.price,
                                                                        p.special_offer_remaining_uses,
                                                                    )
                                                                    .0
                                                            })
                                                            .unwrap_or(p.price * rust_decimal::Decimal::from(p.count.get())),
                                                    })
                                                    .collect();
                                                #[allow(unused_results)]
                                                spawn(async move {
                                                    match checkout(cid, items, cart_time).await {
                                                        Ok(()) => {
                                                            checkout_done.set(true);
                                                            global_state.write().cart.clear();
                                                        }
                                                        Err(e) => {
                                                            checkout_error.set(Some(e.to_string()));
                                                            checking_out.set(false);
                                                        }
                                                    }
                                                });
                                            },
                                            if checking_out() {
                                                i { class: "fa-solid fa-spinner fa-spin text-sm" }
                                                "Behandlar..."
                                            } else {
                                                i { class: "fa-solid fa-lock text-sm" }
                                                "Genomför köp"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Ej inloggad — visa local state cart
                    {
                        let local_cart = global_state.read().cart.clone();
                        let total = global_state.read().cart_total();
                        if local_cart.is_empty() {
                            rsx! {
                                div { class: "text-center py-20 bg-white rounded-2xl shadow-sm border border-gray-100",
                                    i { class: "fa-solid fa-basket-shopping text-6xl text-gray-200 mb-4" }
                                    p { class: "text-gray-500 text-xl", "Kundvagnen är tom" }
                                }
                            }
                        } else {
                            rsx! {
                                div { class: "space-y-3 mb-6",
                                    for item in local_cart.iter() {
                                        div { class: "bg-white rounded-2xl shadow-sm p-4 flex items-center gap-4",
                                            img {
                                                src: "{item.image_url}",
                                                class: "w-16 h-16 object-cover rounded-xl",
                                            }
                                            div { class: "flex-grow",
                                                p { class: "font-bold text-gray-900", "{item.name}" }
                                                p { class: "text-green-700 font-black text-sm", "{item.price:.2} kr" }
                                            }
                                            span { class: "font-bold text-gray-700", "× {item.quantity}" }
                                        }
                                    }
                                }
                                div { class: "bg-white rounded-2xl shadow-sm p-6",
                                    div { class: "flex justify-between mb-6",
                                        span { class: "font-bold text-gray-700", "Totalt" }
                                        span { class: "font-black text-2xl", "{total:.2} kr" }
                                    }
                                    Link {
                                        to: Route::Login {},
                                        class: "block w-full bg-green-700 text-white py-4 rounded-xl font-black text-lg text-center hover:bg-green-800 transition",
                                        "Logga in för att handla"
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