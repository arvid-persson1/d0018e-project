#![allow(non_snake_case)]
use crate::Route;
use crate::database::{Id, Vendor as VendorEntity};
use crate::database::products::vendor_products;
use crate::database::users::vendor_info;
use crate::state::GlobalState;
use dioxus::prelude::*;

/// Säljarprofil offentlig sida, men ägaren kan redigera
#[allow(clippy::same_name_method, reason = "Dioxus macro limitation")]
#[component]
pub fn VendorPage(id: Id<VendorEntity>) -> Element {
    let global_state = use_context::<Signal<GlobalState>>();
    let login = global_state.read().login.clone();

    let is_own_profile = login.as_ref().is_some_and(|l| {
        matches!(l.id, crate::database::LoginId::Vendor(vid) if vid == id)
    });

    let customer_id = login.as_ref().and_then(|l| {
        if let crate::database::LoginId::Customer(cid) = l.id { Some(cid) } else { None }
    });

    let info_resource = use_resource(move || async move { vendor_info(id).await });
    let products_resource = use_resource(move || async move {
        vendor_products(customer_id, id, 50, 0, is_own_profile).await
    });

    rsx! {
        div { class: "min-h-screen bg-gray-50",
            div { class: "max-w-6xl mx-auto p-6",
                Link {
                    to: Route::Home {},
                    class: "text-green-700 hover:text-green-900 font-bold flex items-center gap-2 mb-4 transition-colors",
                    i { class: "fa-solid fa-arrow-left" }
                    "Tillbaka till start"
                }

                match &*info_resource.read() {
                    None => rsx! {
                        p { class: "text-gray-400 animate-pulse mb-8", "Laddar..." }
                    },
                    Some(Err(e)) => rsx! {
                        p { class: "text-red-400 mb-8", "Fel: {e}" }
                    },
                    Some(Ok(info)) => rsx! {
                        div { class: "flex items-center gap-6 mb-8",
                            div { class: "w-20 h-20 rounded-full bg-green-100 flex items-center justify-center",
                                i { class: "fa-solid fa-store text-3xl text-green-700" }
                            }
                            div {
                                h1 { class: "text-3xl font-black text-gray-900", "{info.display_name}" }
                                p { class: "text-gray-500 text-sm mt-1", "{info.description}" }
                            }
                            if is_own_profile {
                                div { class: "ml-auto",
                                    span { class: "text-xs text-gray-400 bg-gray-100 px-3 py-1 rounded-full",
                                        "Din butik"
                                    }
                                }
                            }
                        }
                    },
                }

                h2 { class: "text-xl font-black text-gray-900 mb-4",
                    i { class: "fa-solid fa-tag text-green-700 mr-2" }
                    if is_own_profile {
                        "Dina produkter"
                    } else {
                        "Produkter"
                    }
                }

                match &*products_resource.read() {
                    None => rsx! {
                        p { class: "text-gray-400 animate-pulse", "Laddar..." }
                    },
                    Some(Err(e)) => rsx! {
                        p { class: "text-red-400 text-sm", "Fel: {e}" }
                    },
                    Some(Ok(products)) if products.is_empty() => rsx! {
                        p { class: "text-gray-400 text-sm", "Inga produkter ännu." }
                    },
                    Some(Ok(products)) => rsx! {
                        div { class: "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4",
                            for p in products.iter() {
                                div { class: "bg-white rounded-2xl shadow-sm overflow-hidden hover:shadow-md transition",
                                    Link { to: Route::Product { id: p.id.into() },
                                        img {
                                            src: "{p.thumbnail}",
                                            class: "w-full h-36 object-cover",
                                            alt: "{p.name}",
                                        }
                                        div { class: "p-3",
                                            p { class: "font-bold text-sm text-gray-900 truncate", "{p.name}" }
                                            p { class: "text-green-700 font-black text-sm", "{p.price:.2} kr" }
                                            if p.in_stock == 0 {
                                                p { class: "text-red-400 text-xs mt-1", "Slut i lager" }
                                            }
                                        }
                                    }
                                    if is_own_profile {
                                        div { class: "px-3 pb-3 flex gap-2",
                                            button { class: "flex-1 text-xs border border-gray-200 rounded-lg py-1 hover:bg-gray-50 text-gray-600",
                                                "Redigera"
                                            }
                                            button { class: "text-xs border border-red-100 rounded-lg py-1 px-2 hover:bg-red-50 text-red-500",
                                                i { class: "fa-solid fa-trash" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                }

                if is_own_profile {
                    div { class: "mt-6",
                        button { class: "flex items-center gap-2 bg-green-700 text-white font-black px-5 py-3 rounded-full hover:bg-green-800 transition",
                            i { class: "fa-solid fa-plus" }
                            "Lägg till produkt"
                        }
                    }
                }
            }
        }
    }
}