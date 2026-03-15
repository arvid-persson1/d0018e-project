#![allow(non_snake_case)]
use crate::Route;
use crate::components::product_card::ProductCard;
use crate::database::products::{customer_orders, favorites, set_status, OrderInfo, OrderStatus};
use crate::state::GlobalState;
use dioxus::prelude::*;
use rust_decimal::prelude::ToPrimitive;
 
// Order status badge
 
#[component]
fn OrderStatusBadge(status: OrderStatus) -> Element {
    let (bg, label, icon) = match status {
        OrderStatus::Pending  => ("bg-amber-100 text-amber-800 border-amber-200",  "Väntar på avsändning", "fa-solid fa-clock"),
        OrderStatus::Shipped  => ("bg-blue-100 text-blue-800 border-blue-200",     "Skickad",              "fa-solid fa-truck"),
        OrderStatus::Received => ("bg-green-100 text-green-800 border-green-200",  "Mottagen",             "fa-solid fa-circle-check"),
    };
    rsx! {
        span { class: "inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-bold border {bg}",
            i { class: "{icon} text-[10px]" }
            "{label}"
        }
    }
}
 
// Customer profile
 
/// Kundprofil; visar ordrar och recensioner för inloggad kund
#[allow(clippy::same_name_method, reason = "Dioxus macro limitation")]
#[component]
pub fn CustomerProfile() -> Element {
    let global_state = use_context::<Signal<GlobalState>>();
    let login = global_state.read().login.clone();
 
    let customer_id = match login.as_ref().and_then(|l| {
        if let crate::database::LoginId::Customer(id) = l.id { Some(id) } else { None }
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
 
    let orders_resource = use_resource(move || async move {
        customer_orders(customer_id, 50, 0).await
    });
    let reviews_resource = use_resource(move || async move {
        crate::database::reviews::customer_reviews(customer_id, 50, 0).await
    });
    let fav_resource = use_resource(move || async move {
        favorites(customer_id, 100, 0).await
    });
 
    let mut active_tab = use_signal(|| 0_u8);
    let mut status_msg: Signal<Option<String>> = use_signal(|| None);
 
    let orders_read = orders_resource.read();
    let ord_loading = orders_read.is_none();
    let ord_err: Option<String> = orders_read.as_ref()
        .and_then(|r| r.as_ref().err().map(|e| e.to_string()));
    let orders_list: Option<Vec<OrderInfo>> = orders_read.as_ref()
        .and_then(|r| r.as_ref().ok())
        .map(|v| v.to_vec());
    let ord_empty = orders_list.as_ref().map(|v| v.is_empty()).unwrap_or(false);
 
    let rev_read    = reviews_resource.read();
    let rev_loading = rev_read.is_none();
    let rev_err: Option<String> = rev_read.as_ref()
        .and_then(|r| r.as_ref().err().map(|e| e.to_string()));
    let reviews_list = rev_read.as_ref()
        .and_then(|r| r.as_ref().ok())
        .map(|v: &Box<[_]>| v.to_vec());
    let rev_empty = reviews_list.as_ref().map(|v: &Vec<_>| v.is_empty()).unwrap_or(false);
 
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

                // Status feedback
                if let Some(msg) = status_msg() {
                    div { class: "mb-4 bg-green-50 border border-green-200 rounded-xl p-3 text-sm text-green-800 flex items-center gap-2",
                        i { class: "fa-solid fa-check" }
                        "{msg}"
                    }
                }

                // Flikar
                div { class: "flex gap-2 mb-6 border-b overflow-x-auto",
                    button {
                        class: if active_tab() == 0 { "px-4 py-2 font-bold text-green-700 border-b-2 border-green-700 whitespace-nowrap" } else { "px-4 py-2 text-gray-500 hover:text-gray-700 whitespace-nowrap" },
                        onclick: move |_| active_tab.set(0),
                        i { class: "fa-solid fa-bag-shopping mr-2" }
                        "Mina ordrar"
                    }
                    button {
                        class: if active_tab() == 1 { "px-4 py-2 font-bold text-green-700 border-b-2 border-green-700 whitespace-nowrap" } else { "px-4 py-2 text-gray-500 hover:text-gray-700 whitespace-nowrap" },
                        onclick: move |_| active_tab.set(1),
                        i { class: "fa-solid fa-star mr-2" }
                        "Mina recensioner"
                    }
                    button {
                        class: if active_tab() == 2 { "px-4 py-2 font-bold text-green-700 border-b-2 border-green-700 whitespace-nowrap" } else { "px-4 py-2 text-gray-500 hover:text-gray-700 whitespace-nowrap" },
                        onclick: move |_| active_tab.set(2),
                        i { class: "fa-solid fa-heart mr-2" }
                        "Mina favoriter"
                    }
                }

                // Orders tab
                if active_tab() == 0 {
                    if ord_loading {
                        p { class: "text-gray-400 animate-pulse", "Laddar..." }
                    } else if let Some(err) = ord_err {
                        p { class: "text-red-400 text-sm", "Fel: {err}" }
                    } else if ord_empty {
                        div { class: "text-center py-16 bg-white rounded-2xl",
                            i { class: "fa-solid fa-bag-shopping text-4xl text-gray-200 mb-3 block" }
                            p { class: "text-gray-400 text-sm font-semibold", "Inga köp ännu." }
                        }
                    } else if let Some(orders) = orders_list {
                        div { class: "space-y-4",
                            for (order_idx , order) in orders.iter().enumerate() {
                                {
                                    let t = order.time;
                                    let time_str = format!(
                                        "{:04}-{:02}-{:02} {:02}:{:02}",
                                        t.year(),
                                        t.month() as u8,
                                        t.day(),
                                        t.hour(),
                                        t.minute(),
                                    );
                                    let total: rust_decimal::Decimal = order.purchases.iter().map(|p| p.paid).sum();
                                    rsx! {
                                        div { class: "bg-white rounded-2xl shadow-sm overflow-hidden border border-gray-100",
                                            div { class: "flex items-center justify-between px-5 py-3 bg-gray-50 border-b border-gray-100",
                                                div { class: "flex items-center gap-3",
                                                    span { class: "text-xs font-semibold text-gray-500", "Order #{order_idx + 1}" }
                                                    span { class: "text-gray-300", "•" }
                                                    span { class: "text-xs text-gray-400", "{time_str}" }
                                                }
                                            }

                                            div { class: "divide-y divide-gray-50",
                                                for purchase in order.purchases.iter() {
                                                    {
                                                        let ps = purchase.status;
                                                        let changed = purchase.product_changed;
                                                        rsx! {
                                                            div { class: "flex items-center gap-4 px-5 py-4",
                                                                img {
                                                                    src: "{purchase.thumbnail}",
                                                                    class: "w-14 h-14 rounded-xl object-cover shrink-0 bg-gray-100",
                                                                    alt: "{purchase.product_name}",
                                                                }
                                                                div { class: "flex-1 min-w-0",
                                                                    p { class: "font-bold text-sm text-gray-900 truncate", "{purchase.product_name}" }
                                                                    p { class: "text-xs text-gray-500 mt-0.5",
                                                                        "{purchase.vendor_name}"
                                                                        span { class: "mx-1 text-gray-300", "·" }
                                                                        "{purchase.number} st"
                                                                    }
                                                                    if changed {
                                                                        p { class: "text-xs text-amber-600 mt-0.5",
                                                                            i { class: "fa-solid fa-circle-exclamation mr-1" }
                                                                            "Produkten har ändrats sedan köpet"
                                                                        }
                                                                    }
                                                                }
                                                                div { class: "flex flex-col items-end gap-2 shrink-0",
                                                                    p { class: "font-black text-green-700 text-sm", "{purchase.paid:.2} kr" }
                                                                    OrderStatusBadge { status: ps }
                                                                    span { class: "inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-bold border bg-green-50 text-green-700 border-green-200",
                                                                        i { class: "fa-solid fa-check text-[10px]" }
                                                                        "Betald"
                                                                    }

                                                                    if ps == OrderStatus::Shipped {
                                                                        {
                                                                            let purchase_id = purchase.id;
                                                                            rsx! {
                                                                                button {
                                                                                    class: "text-xs bg-green-700 text-white font-bold px-3 py-1 rounded-lg hover:bg-green-800 transition",
                                                                                    onclick: move |_| {
                                                                                        let mut sm = status_msg;
                                                                                        let mut r = orders_resource;
                                                                                        #[allow(unused_results)]
                                                                                        spawn(async move {
                                                                                            match set_status(purchase_id, OrderStatus::Received).await {
                                                                                                Ok(()) => {
                                                                                                    sm.set(Some("Order markerad som mottagen.".into()));
                                                                                                    r.restart();
                                                                                                }
                                                                                                Err(e) => sm.set(Some(format!("Fel: {e}"))),
                                                                                            }
                                                                                        });
                                                                                    },
                                                                                    i { class: "fa-solid fa-box-open mr-1" }
                                                                                    "Markera mottagen"
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

                                            div { class: "px-5 py-3 bg-gray-50 border-t border-gray-100 flex justify-between items-center",
                                                span { class: "text-xs text-gray-500", "{order.purchases.len()} produkt(er)" }
                                                span { class: "font-black text-gray-900 text-sm", "Totalt: {total:.2} kr" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Favorites tab
                if active_tab() == 2 {
                    {
                        let fav_read = fav_resource.read();
                        let fav_loading = fav_read.is_none();
                        let fav_err: Option<String> = fav_read
                            .as_ref()
                            .and_then(|r| r.as_ref().err().map(|e| e.to_string()));
                        let fav_list = fav_read
                            .as_ref()
                            .and_then(|r| r.as_ref().ok())
                            .map(|v: &Box<[_]>| v.to_vec());
                        let fav_empty = fav_list.as_ref().map(|v| v.is_empty()).unwrap_or(false);
                        rsx! {
                            if fav_loading {
                                p { class: "text-gray-400 animate-pulse", "Laddar..." }
                            } else if let Some(err) = fav_err {
                                p { class: "text-red-400 text-sm", "Fel: {err}" }
                            } else if fav_empty {
                                div { class: "text-center py-16 bg-white rounded-2xl",
                                    i { class: "fa-regular fa-heart text-4xl text-gray-200 mb-3 block" }
                                    p { class: "text-gray-400 text-sm font-semibold", "Inga favoriter ännu." }
                                }
                            } else if let Some(prods) = fav_list {
                                div { class: "grid grid-cols-2 md:grid-cols-4 gap-4",
                                    for p in prods.iter() {
                                        ProductCard {
                                            id: p.id.get(),
                                            name: p.name.clone(),
                                            price: p.price.to_f64().unwrap_or_default(),
                                            comparison_price: format!("{:.2} kr / {}", p.price, p.amount_per_unit),
                                            image_url: p.thumbnail.to_string(),
                                            in_stock: u32::MAX,
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Reviews tab
                if active_tab() == 1 {
                    if rev_loading {
                        p { class: "text-gray-400 animate-pulse", "Laddar..." }
                    } else if let Some(err) = rev_err {
                        p { class: "text-red-400 text-sm", "Fel: {err}" }
                    } else if rev_empty {
                        p { class: "text-gray-400 text-sm", "Inga recensioner ännu." }
                    } else if let Some(reviews) = reviews_list {
                        div { class: "space-y-4",
                            for review in reviews.iter() {
                                div { class: "bg-white rounded-2xl shadow-sm p-4",
                                    div { class: "flex items-center gap-3 mb-3",
                                        Link {
                                            to: Route::Product {
                                                id: review.product.into(),
                                            },
                                            img {
                                                src: "{review.thumbnail}",
                                                class: "w-12 h-12 rounded-xl object-cover hover:opacity-80 transition",
                                                alt: "{review.product_name}",
                                            }
                                        }
                                        div {
                                            Link {
                                                to: Route::Product {
                                                    id: review.product.into(),
                                                },
                                                p { class: "font-bold text-sm text-gray-900 hover:text-green-700 transition",
                                                    "{review.product_name}"
                                                }
                                            }
                                            div { class: "flex gap-0.5 mt-0.5",
                                                for i in 0..5_u8 {
                                                    i { class: if i < review.rating.get().get() { "fa-solid fa-star text-yellow-400 text-xs" } else { "fa-regular fa-star text-yellow-400 text-xs" } }
                                                }
                                            }
                                        }
                                    }
                                    p { class: "font-bold text-gray-900 text-sm mb-1",
                                        "{review.title}"
                                    }
                                    p { class: "text-gray-600 text-sm", "{review.content}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

