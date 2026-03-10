use crate::Route;
use crate::components::product_card::ProductCard;
use crate::database::products::{
    ProductOverview, ProductOverviewDiscounted, best_discounts, newest_products,
};
use crate::state::GlobalState;
use dioxus::prelude::*;

/// Home page..
#[allow(non_snake_case)]
#[component]
pub fn Home() -> Element {
    let global_state = use_context::<Signal<GlobalState>>();
    let customer_id = global_state.read().login.as_ref().and_then(|l| {
        use crate::database::LoginId;
        if let LoginId::Customer(id) = l.id {
            Some(id)
        } else {
            None
        }
    });

    let discounted = use_resource(move || async move { best_discounts(customer_id, 12, 0).await });
    let newest = use_resource(move || async move { newest_products(customer_id, 12, 0).await });

    rsx! {
        div { class: "min-h-screen bg-gray-50",
            main { class: "container mx-auto p-4 py-8",
                div { class: "mb-10",
                    h1 { class: "text-2xl font-bold text-gray-800", "Välkommen till boop!" }
                    p { class: "text-gray-600", "Vi är definitivt inte coop" }
                }

                // Bästa erbjudanden
                div { class: "mb-20",
                    div { class: "flex justify-between items-center mb-6",
                        h2 { class: "text-xl font-bold flex items-center gap-2",
                            span { class: "w-2 h-8 bg-green-700 rounded-full block" }
                            //i { class: "fa-solid fa-tag text-green-600 mr-1" }
                            "Bästa erbjudanden!"
                        }
                    }
                    match &*discounted.read() {
                        None => rsx! {
                            div { class: "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6",
                                for _ in 0..4 {
                                    div { class: "bg-white border border-gray-200 rounded-lg h-80 animate-pulse" }
                                }
                            }
                        },
                        Some(Err(e)) => rsx! {
                            p { class: "text-red-400 text-sm", "Kunde inte hämta erbjudanden: {e}" }
                        },
                        Some(Ok(products)) if products.is_empty() => rsx! {
                            div { class: "h-48 flex flex-col items-center justify-center bg-gray-50 rounded-xl border-2 border-dashed border-gray-200",
                                i { class: "fa-solid fa-tag text-4xl text-gray-400 mb-3" }
                                p { class: "font-bold text-gray-500", "Inga aktiva erbjudanden just nu." }
                            }
                        },
                        Some(Ok(products)) => rsx! {
                            DiscountedSlider { products: products.to_vec() }
                        },
                    }
                }

                // Senast tillagda
                div { class: "mb-20",
                    div { class: "flex justify-between items-center mb-6",
                        h2 { class: "text-xl font-bold flex items-center gap-2",
                            span { class: "w-2 h-8 bg-green-700 rounded-full block" }
                            //i { class: "fa-solid fa-clock text-green-600 mr-1" }
                            "Senast tillagda"
                        }
                    }
                    match &*newest.read() {
                        None => rsx! {
                            div { class: "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6",
                                for _ in 0..4 {
                                    div { class: "bg-white border border-gray-200 rounded-lg h-80 animate-pulse" }
                                }
                            }
                        },
                        Some(Err(e)) => rsx! {
                            p { class: "text-red-400 text-sm", "Kunde inte hämta produkter: {e}" }
                        },
                        Some(Ok(products)) if products.is_empty() => rsx! {
                            div { class: "h-48 flex flex-col items-center justify-center bg-gray-50 rounded-xl border-2 border-dashed border-gray-200",
                                i { class: "fa-solid fa-box-open text-4xl text-gray-400 mb-3" }
                                p { class: "font-bold text-gray-500", "Inga produkter hittades." }
                            }
                        },
                        Some(Ok(products)) => rsx! {
                            NewestSlider { products: products.to_vec() }
                        },
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct DiscountedSliderProps {
    products: Vec<ProductOverviewDiscounted>,
}

#[component]
fn DiscountedSlider(props: DiscountedSliderProps) -> Element {
    use rust_decimal::prelude::ToPrimitive;
    let mut pos = use_signal(|| 0_usize);
    let items = props.products;
    let total = items.len();
    let max_steps = if total > 4 {
        (total as f64 / 4.0).ceil() as usize
    } else {
        1
    };
    let current_pos = *pos.read();
    let offset = current_pos * 100;

    rsx! {
        div { class: "relative group",
            if current_pos > 0 && total > 4 {
                button {
                    class: "absolute -left-5 top-1/2 -translate-y-1/2 w-12 h-12 bg-white shadow-2xl border rounded-full flex items-center justify-center hover:bg-green-700 hover:text-white transition-all z-30",
                    onclick: move |_| pos.set(current_pos - 1),
                    i { class: "fa-solid fa-chevron-left text-lg" }
                }
            }
            div { class: "overflow-hidden",
                div {
                    class: "flex transition-transform duration-700 ease-in-out",
                    style: "transform: translateX(-{offset}%);",
                    for p in items.iter().take(11) {
                        div { class: "min-w-full md:min-w-[25%] p-2",
                            ProductCard {
                                id: p.id.get(),
                                name: p.name.to_string(),
                                price: p.special_offer_deal
                                    .database_repr()
                                    .and_then(|(new_price, _, _)| new_price)
                                    .and_then(|np| np.to_f64())
                                    .unwrap_or_else(|| p.price.to_f64().unwrap_or_default()),
                                comparison_price: format!("{:.2} kr / {}", p.price, p.amount_per_unit),
                                image_url: p.thumbnail.to_string(),
                            }
                        }
                    }
                    div { class: "min-w-full md:min-w-[25%] p-2",
                        Link { to: Route::Home {},
                            div { class: "h-full flex flex-col items-center justify-center bg-gray-50 rounded-xl border-2 border-dashed border-gray-200 hover:bg-green-100 transition-all min-h-[380px]",
                                div { class: "text-center p-4",
                                    i { class: "fa-solid fa-arrow-right-long text-4xl text-green-700 mb-3" }
                                    p { class: "font-bold text-gray-800", "Visa alla erbjudanden" }
                                }
                            }
                        }
                    }
                }
            }
            if current_pos < max_steps - 1 && total > 4 {
                button {
                    class: "absolute -right-5 top-1/2 -translate-y-1/2 w-12 h-12 bg-white shadow-2xl border rounded-full flex items-center justify-center hover:bg-green-700 hover:text-white transition-all z-30",
                    onclick: move |_| pos.set(current_pos + 1),
                    i { class: "fa-solid fa-chevron-right text-lg" }
                }
            }
            if total > 4 {
                div { class: "flex justify-center items-center gap-3 mt-8",
                    for i in 0..max_steps {
                        button {
                            class: if current_pos == i { "w-10 h-2 rounded-full bg-green-700 transition-all" } else { "w-2 h-2 rounded-full bg-gray-300 hover:bg-gray-400" },
                            onclick: move |_| pos.set(i),
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct NewestSliderProps {
    products: Vec<ProductOverview>,
}

#[component]
fn NewestSlider(props: NewestSliderProps) -> Element {
    use rust_decimal::prelude::ToPrimitive;
    let mut pos = use_signal(|| 0_usize);
    let items = props.products;
    let total = items.len();
    let max_steps = if total > 4 {
        (total as f64 / 4.0).ceil() as usize
    } else {
        1
    };
    let current_pos = *pos.read();
    let offset = current_pos * 100;

    rsx! {
        div { class: "relative group",
            if current_pos > 0 && total > 4 {
                button {
                    class: "absolute -left-5 top-1/2 -translate-y-1/2 w-12 h-12 bg-white shadow-2xl border rounded-full flex items-center justify-center hover:bg-green-700 hover:text-white transition-all z-30",
                    onclick: move |_| pos.set(current_pos - 1),
                    i { class: "fa-solid fa-chevron-left text-lg" }
                }
            }
            div { class: "overflow-hidden",
                div {
                    class: "flex transition-transform duration-700 ease-in-out",
                    style: "transform: translateX(-{offset}%);",
                    for p in items.iter().take(11) {
                        div { class: "min-w-full md:min-w-[25%] p-2",
                            ProductCard {
                                id: p.id.get(),
                                name: p.name.to_string(),
                                price: p.price.to_f64().unwrap_or_default(),
                                comparison_price: format!("{:.2} kr / {}", p.price, p.amount_per_unit),
                                image_url: p.thumbnail.to_string(),
                            }
                        }
                    }
                    div { class: "min-w-full md:min-w-[25%] p-2",
                        Link {
                            to: Route::Category {
                                id: crate::database::Id::<crate::database::Category>::from(0),
                            },
                            div { class: "h-full flex flex-col items-center justify-center bg-gray-50 rounded-xl border-2 border-dashed border-gray-200 hover:bg-green-100 transition-all min-h-[380px]",
                                div { class: "text-center p-4",
                                    i { class: "fa-solid fa-arrow-right-long text-4xl text-green-700 mb-3" }
                                    p { class: "font-bold text-gray-800", "Visa alla produkter" }
                                }
                            }
                        }
                    }
                }
            }
            if current_pos < max_steps - 1 && total > 4 {
                button {
                    class: "absolute -right-5 top-1/2 -translate-y-1/2 w-12 h-12 bg-white shadow-2xl border rounded-full flex items-center justify-center hover:bg-green-700 hover:text-white transition-all z-30",
                    onclick: move |_| pos.set(current_pos + 1),
                    i { class: "fa-solid fa-chevron-right text-lg" }
                }
            }
            if total > 4 {
                div { class: "flex justify-center items-center gap-3 mt-8",
                    for i in 0..max_steps {
                        button {
                            class: if current_pos == i { "w-10 h-2 rounded-full bg-green-700 transition-all" } else { "w-2 h-2 rounded-full bg-gray-300 hover:bg-gray-400" },
                            onclick: move |_| pos.set(i),
                        }
                    }
                }
            }
        }
    }
}