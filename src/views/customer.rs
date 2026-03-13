#![allow(non_snake_case)]
use crate::Route;
use crate::database::{
    products::{favorites, orders},
    reviews::customer_reviews,
};
use crate::state::GlobalState;
use dioxus::prelude::*;

/// Kundprofil; visar favoriter, köphistorik och recensioner.
/// Visar orderstatus-badge med färg baserat på status-sträng.
/// När order_status läggs till i databasen ersätts "placeholder" med riktigt värde.
#[component]
fn OrderStatusBadge(status: String) -> Element {
    let (bg, text, icon) = match status.as_str() {
        "delivered"  => ("bg-green-100 text-green-800 border-green-200",  "Levererad",      "fa-solid fa-circle-check"),
        "shipped"    => ("bg-blue-100 text-blue-800 border-blue-200",     "Skickad",         "fa-solid fa-truck"),
        "processing" => ("bg-yellow-100 text-yellow-800 border-yellow-200","Behandlas",     "fa-solid fa-gear fa-spin"),
        "cancelled"  => ("bg-red-100 text-red-800 border-red-200",        "Avbruten",        "fa-solid fa-xmark"),
        _            => ("bg-gray-100 text-gray-600 border-gray-200",     "Lagd",            "fa-solid fa-clock"),
    };
    rsx! {
        span { class: "inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-bold border {bg}",
            i { class: "{icon} text-[10px]" }
            "{text}"
        }
    }
}
 
/// Visar betalningsstatus-badge.
/// När payment_status läggs till i databasen ersätts "placeholder" med riktigt värde.
#[component]
fn PaymentStatusBadge(status: String) -> Element {
    let (bg, text, icon) = match status.as_str() {
        "paid"    => ("bg-green-50 text-green-700 border-green-200", "Betald",     "fa-solid fa-check"),
        "pending" => ("bg-amber-50 text-amber-700 border-amber-200", "Väntar",     "fa-solid fa-hourglass-half"),
        "refunded"=> ("bg-purple-50 text-purple-700 border-purple-200","Återbetald","fa-solid fa-rotate-left"),
        _         => ("bg-gray-50 text-gray-600 border-gray-200",    "Okänd",      "fa-solid fa-circle-question"),
    };
    rsx! {
        span { class: "inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-semibold border {bg}",
            i { class: "{icon} text-[10px]" }
            "{text}"
        }
    }
}
 
/// Kundprofil; visar favoriter, köphistorik och recensioner.
#[component]
pub fn CustomerProfile() -> Element {
    let global_state = use_context::<Signal<GlobalState>>();
    let login = global_state.read().login.clone();
 
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
        }
    };
 
    let username = login.as_ref().map(|l| l.username.to_string()).unwrap_or_default();
 
    let fav_resource     = use_resource(move || async move { favorites(customer_id, 20, 0).await });
    let orders_resource  = use_resource(move || async move { orders(customer_id, 20, 0).await });
    let reviews_resource = use_resource(move || async move { customer_reviews(customer_id, 20, 0).await });
 
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
                div { class: "flex gap-2 mb-6 border-b overflow-x-auto",
                    button {
                        class: if active_tab() == 0 { "px-4 py-2 font-bold text-green-700 border-b-2 border-green-700 whitespace-nowrap" } else { "px-4 py-2 text-gray-500 hover:text-gray-700 whitespace-nowrap" },
                        onclick: move |_| active_tab.set(0),
                        i { class: "fa-solid fa-heart mr-2" }
                        "Favoriter"
                    }
                    button {
                        class: if active_tab() == 1 { "px-4 py-2 font-bold text-green-700 border-b-2 border-green-700 whitespace-nowrap" } else { "px-4 py-2 text-gray-500 hover:text-gray-700 whitespace-nowrap" },
                        onclick: move |_| active_tab.set(1),
                        i { class: "fa-solid fa-bag-shopping mr-2" }
                        "Mina ordrar"
                    }
                    button {
                        class: if active_tab() == 2 { "px-4 py-2 font-bold text-green-700 border-b-2 border-green-700 whitespace-nowrap" } else { "px-4 py-2 text-gray-500 hover:text-gray-700 whitespace-nowrap" },
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

                // Ordrar med status
                if active_tab() == 1 {
                    // Info-banner om placeholder-status
                    div { class: "mb-4 bg-amber-50 border border-amber-200 rounded-xl p-3 flex items-start gap-2 text-sm text-amber-800",
                        i { class: "fa-solid fa-circle-info mt-0.5 shrink-0" }
                        span {
                            "Orderstatus och betalningsstatus är placeholders tills databasen är uppdaterad med dessa kolumner."
                        }
                    }
                    match &*orders_resource.read() {
                        None => rsx! {
                            p { class: "text-gray-400 animate-pulse", "Laddar..." }
                        },
                        Some(Err(e)) => rsx! {
                            p { class: "text-red-400 text-sm", "Fel: {e}" }
                        },
                        Some(Ok(orders)) if orders.is_empty() => rsx! {
                            div { class: "text-center py-16 bg-white rounded-2xl",
                                i { class: "fa-solid fa-bag-shopping text-4xl text-gray-200 mb-3" }
                                p { class: "text-gray-400 text-sm font-semibold", "Inga köp ännu." }
                            }
                        },
                        Some(Ok(orders)) => rsx! {
                            div { class: "space-y-4",
                                for (order_idx , order) in orders.iter().enumerate() {
                                    div { class: "bg-white rounded-2xl shadow-sm overflow-hidden border border-gray-100",
                                        // Order header med status
                                        div { class: "flex items-center justify-between px-5 py-3 bg-gray-50 border-b border-gray-100",
                                            div { class: "flex items-center gap-3",
                                                span { class: "text-xs text-gray-500 font-semibold", "Order #{order_idx + 1}" }
                                                span { class: "text-gray-300", "•" }
                                                span { class: "text-xs text-gray-400", "{order.time}" }
                                            }
                                            // Status-badges — placeholder tills DB har kolumnerna
                                            div { class: "flex items-center gap-2",
                                                // TODO: Ersätt "placed" med order.order_status när DB-kolumn finns
                                                OrderStatusBadge { status: "placed".to_string() }
                                                // TODO: Ersätt "paid" med order.payment_status när DB-kolumn finns
                                                PaymentStatusBadge { status: "paid".to_string() }
                                            }
                                        }
                                        // Köpta produkter
                                        div { class: "divide-y divide-gray-50",
                                            for purchase in order.purchases.iter() {
                                                div { class: "flex items-center gap-4 px-5 py-4",
                                                    img {
                                                        src: "{purchase.thumbnail}",
                                                        class: "w-14 h-14 rounded-xl object-cover shrink-0 bg-gray-100",
                                                        alt: "{purchase.product_name}",
                                                    }
                                                    div { class: "flex-1 min-w-0",
                                                        p { class: "font-bold text-sm text-gray-900 truncate",
                                                            "{purchase.product_name}"
                                                        }
                                                        p { class: "text-xs text-gray-500 mt-0.5",
                                                            "{purchase.vendor_name}"
                                                            span { class: "mx-1 text-gray-300", "·" }
                                                            "{purchase.number} st"
                                                            if purchase.special_offer_used {
                                                                span { class: "ml-2 text-green-700 font-semibold",
                                                                    i { class: "fa-solid fa-tag text-[10px] mr-1" } // Order-footer med total
                                                                    "Erbjudande använt"
                                                                }
                                                            }
                                                        }
                                                    }
                                                    p { class: "font-black text-green-700 text-sm shrink-0",
                                                        "{purchase.paid:.2} kr"
                                                    }
                                                }
                                            }
                                        }
                                        // Order-footer med total
                                        div { class: "px-5 py-3 bg-gray-50 border-t border-gray-100 flex justify-between items-center",
                                            span { class: "text-xs text-gray-500", "{order.purchases.len()} produkt(er)" }
                                            span { class: "font-black text-gray-900 text-sm",
                                                "Totalt: "
                                                {
                                                    let total: rust_decimal::Decimal = order.purchases.iter().map(|p| p.paid).sum();
                                                    format!("{total:.2} kr")
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