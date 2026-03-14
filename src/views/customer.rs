#![allow(non_snake_case)]
use crate::Route;
use crate::database::{
    products::{customer_orders, favorites},
    reviews::customer_reviews,
};
use crate::state::GlobalState;
use dioxus::prelude::*;

/// Kundprofil; visar favoriter, köphistorik och recensioner.
#[component]
pub fn CustomerProfile() -> Element {
    let global_state = use_context::<Signal<GlobalState>>();
    let login = global_state.read().login.clone();

    // Hämta customer ID från login
    let customer_id = match login.as_ref().and_then(|l| {
        if let crate::database::LoginId::Customer(id) = l.id {
            Some(id)
        } else {
            None
        }
    }) {
        Some(id) => id,
        None => {
            return rsx! {
                div { class: "min-h-screen bg-gray-50 flex items-center justify-center",
                    div { class: "text-center",
                        p { class: "text-gray-500 mb-4", "Du måste vara inloggad som kund." }
                        Link {
                            to: Route::Login {},
                            class: "bg-green-700 text-white font-black px-6 py-3 rounded-full",
                            "Logga in"
                        }
                    }
                }
            };
        },
    };

    let username = login
        .as_ref()
        .map(|l| l.username.to_string())
        .unwrap_or_default();

    let fav_resource = use_resource(move || async move { favorites(customer_id, 20, 0).await });
    let orders_resource =
        use_resource(move || async move { customer_orders(customer_id, 20, 0).await });
    let reviews_resource =
        use_resource(move || async move { customer_reviews(customer_id, 20, 0).await });

    let mut active_tab = use_signal(|| 0_u8);

    rsx! {
        div { class: "min-h-screen bg-gray-50",
            div { class: "max-w-5xl mx-auto p-6",
                Link {
                    to: Route::Home {},
                    class: "text-green-700 hover:text-green-900 font-bold flex items-center gap-2 mb-4 transition-colors",
                    i { class: "fa-solid fa-arrow-left" }
                    "Tillbaka till start"
                }

                // Header
                div { class: "flex items-center gap-4 mb-8",
                    div { class: "w-20 h-20 rounded-full bg-green-100 flex items-center justify-center",
                        i { class: "fa-solid fa-user text-3xl text-green-700" }
                    }
                    div {
                        h1 { class: "text-3xl font-black text-gray-900", "{username}" }
                        p { class: "text-gray-500 text-sm", "Kund" }
                    }
                }

                // Flikar
                div { class: "flex gap-2 mb-6 border-b",
                    button {
                        class: if active_tab() == 0 { "px-4 py-2 font-bold text-green-700 border-b-2 border-green-700" } else { "px-4 py-2 text-gray-500 hover:text-gray-700" },
                        onclick: move |_| active_tab.set(0),
                        i { class: "fa-solid fa-heart mr-2" }
                        "Favoriter"
                    }
                    button {
                        class: if active_tab() == 1 { "px-4 py-2 font-bold text-green-700 border-b-2 border-green-700" } else { "px-4 py-2 text-gray-500 hover:text-gray-700" },
                        onclick: move |_| active_tab.set(1),
                        i { class: "fa-solid fa-bag-shopping mr-2" }
                        "Köphistorik"
                    }
                    button {
                        class: if active_tab() == 2 { "px-4 py-2 font-bold text-green-700 border-b-2 border-green-700" } else { "px-4 py-2 text-gray-500 hover:text-gray-700" },
                        onclick: move |_| active_tab.set(2),
                        i { class: "fa-solid fa-star mr-2" }
                        "Mina recensioner"
                    }
                }

                // Favoriter
                if active_tab() == 0 {
                    match &*fav_resource.read() {
                        None => rsx! {
                            p { class: "text-gray-400 animate-pulse", "Laddar..." }
                        },
                        Some(Err(e)) => rsx! {
                            p { class: "text-red-400 text-sm", "Fel: {e}" }
                        },
                        Some(Ok(products)) if products.is_empty() => rsx! {
                            p { class: "text-gray-400 text-sm", "Inga favoriter ännu." }
                        },
                        Some(Ok(products)) => rsx! {
                            div { class: "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4",
                                for p in products.iter() {
                                    Link {
                                        to: Route::Product { id: p.id.into() },
                                        class: "bg-white rounded-2xl shadow-sm p-4 hover:shadow-md transition",
                                        img {
                                            src: "{p.thumbnail}",
                                            class: "w-full h-32 object-cover rounded-xl mb-3",
                                            alt: "{p.name}",
                                        }
                                        p { class: "font-bold text-gray-900 text-sm", "{p.name}" }
                                        p { class: "text-green-700 font-black text-sm", "{p.price:.2} kr" }
                                        p { class: "text-gray-400 text-xs", "{p.vendor_name}" }
                                    }
                                }
                            }
                        },
                    }
                }

                // Köphistorik
                if active_tab() == 1 {
                    match &*orders_resource.read() {
                        None => rsx! {
                            p { class: "text-gray-400 animate-pulse", "Laddar..." }
                        },
                        Some(Err(e)) => rsx! {
                            p { class: "text-red-400 text-sm", "Fel: {e}" }
                        },
                        Some(Ok(orders)) if orders.is_empty() => rsx! {
                            p { class: "text-gray-400 text-sm", "Inga köp ännu." }
                        },
                        Some(Ok(orders)) => rsx! {
                            div { class: "space-y-4",
                                for order in orders.iter() {
                                    div { class: "bg-white rounded-2xl shadow-sm p-4",
                                        p { class: "text-xs text-gray-400 mb-3 font-semibold", "{order.time}" }
                                        div { class: "space-y-3",
                                            for purchase in order.purchases.iter() {
                                                div { class: "flex items-center gap-3",
                                                    img {
                                                        src: "{purchase.thumbnail}",
                                                        class: "w-12 h-12 rounded-xl object-cover",
                                                        alt: "{purchase.product_name}",
                                                    }
                                                    div { class: "flex-1",
                                                        p { class: "font-bold text-sm text-gray-900", "{purchase.product_name}" }
                                                        p { class: "text-xs text-gray-500",
                                                            "{purchase.vendor_name} · {purchase.number} st"
                                                        }
                                                    }
                                                    p { class: "font-black text-green-700 text-sm", "{purchase.paid:.2} kr" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                    }
                }

                // Recensioner
                if active_tab() == 2 {
                    match &*reviews_resource.read() {
                        None => rsx! {
                            p { class: "text-gray-400 animate-pulse", "Laddar..." }
                        },
                        Some(Err(e)) => rsx! {
                            p { class: "text-red-400 text-sm", "Fel: {e}" }
                        },
                        Some(Ok(reviews)) if reviews.is_empty() => rsx! {
                            p { class: "text-gray-400 text-sm", "Inga recensioner ännu." }
                        },
                        Some(Ok(reviews)) => rsx! {
                            div { class: "space-y-4",
                                for review in reviews.iter() {
                                    div { class: "bg-white rounded-2xl shadow-sm p-4",
                                        div { class: "flex items-center gap-3 mb-3",
                                            img {
                                                src: "{review.thumbnail}",
                                                class: "w-12 h-12 rounded-xl object-cover",
                                                alt: "{review.product_name}",
                                            }
                                            div {
                                                p { class: "font-bold text-sm text-gray-900", "{review.product_name}" }
                                                div { class: "flex gap-0.5 mt-0.5",
                                                    for i in 0..5_u8 {
                                                        i { class: if i < review.rating.get().get() { "fa-solid fa-star text-yellow-400 text-xs" } else { "fa-regular fa-star text-yellow-400 text-xs" } }
                                                    }
                                                }
                                            }
                                        }
                                        p { class: "font-bold text-gray-900 text-sm mb-1", "{review.title}" }
                                        p { class: "text-gray-600 text-sm", "{review.content}" }
                                    }
                                }
                            }
                        },
                    }
                }
            }
        }
    }
}

