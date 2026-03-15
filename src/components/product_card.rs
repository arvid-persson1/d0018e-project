use crate::Route;
use crate::database::Deal;
use crate::state::GlobalState;
use dioxus::prelude::*;
 
/// Builds a short display string for a special offer badge.
/// Returns a short display string for a special offer badge, or `None` if no offer applies.
///
/// Matches the three [`Deal`] variants from the database:
/// `Discount`: `(Some(new_price), None, None)`; simple price reduction, shown as "−N%"
/// `Batch`: `(None, Some(take), Some(pay_for))`; "ta N betala för M"
/// `BatchPrice`: `(Some(pay), Some(take), None)`; "ta N betala X kr totalt"
// class for a product card
pub fn offer_label(deal: Option<Deal>, base_price: rust_decimal::Decimal) -> Option<String> {
    let repr = deal?.database_repr().ok()?;
    Some(match repr {
        // Discount
        (Some(new_price), None, None) => {
            use rust_decimal::Decimal;
            let pct = ((base_price - new_price) / base_price * Decimal::ONE_HUNDRED)
                .round()
                .to_string();
            format!("-{pct}%")
        },
        // Batch; take N pay for M
        (None, Some(take), Some(pay_for)) => {
            format!("Köp {take} betala för {pay_for}")
        },
        // BatchPrice; take N pay X total
        (Some(pay), Some(take), None) => {
            let pay_fmt = format!("{:.2}", pay).replace('.', ",");
            format!("{take} för {pay_fmt} kr")
        },
        _ => return None,
    })
}
 
#[derive(Props, Debug, Clone, PartialEq)]
#[expect(missing_docs, reason = "TODO")]
pub struct ProductProps {
    pub id: i32,
    pub name: String,
    pub price: f64,
    pub image_url: String,
    pub comparison_price: String,
    pub in_stock: u32,
    #[props(default)]
    pub special_offer: Option<String>,
}
 
/// Product card.
#[component]
pub fn ProductCard(props: ProductProps) -> Element {
    let mut global_state = use_context::<Signal<GlobalState>>();
 
    let is_favorite = global_state.read().favorites.contains(&props.id);
 
    let product_id = props.id;
    let product_name = props.name.clone();
    let product_price = props.price;
    let product_image = props.image_url.clone();
    let in_stock = props.in_stock;
 
    let formatted_price = format!("{:.2}", props.price).replace('.', ",");
    let formatted_comparison = props.comparison_price.replace('.', ",");
 
    let heart_class = if is_favorite { "text-red-500" } else { "text-gray-400 hover:text-red-500" };
 
    let quantity = global_state
        .read()
        .cart
        .iter()
        .find(|i| i.product_id == product_id)
        .map(|i| i.quantity)
        .unwrap_or(0);
 
    rsx! {
        div { class: "bg-white border border-gray-200 rounded-lg shadow-sm hover:shadow-md transition p-4 flex flex-col gap-3 relative",

            if let Some(offer) = &props.special_offer {
                div { class: "absolute top-2 left-2 z-10 bg-green-600 text-white text-xs font-bold px-2 py-1 rounded-full shadow",
                    "{offer}"
                }
            }

            Link {
                to: Route::Product {
                    id: props.id.into(),
                },
                img {
                    src: "{props.image_url}",
                    class: "w-full h-60 object-contain mb-2 cursor-pointer hover:opacity-80 transition",
                }
            }

            div { class: "flex flex-col gap-0.5",
                Link {
                    to: Route::Product {
                        id: props.id.into(),
                    },
                    h3 { class: "font-bold text-lg text-gray-800 hover:text-green-700 cursor-pointer",
                        "{props.name}"
                    }
                }
                p { class: "text-2xl font-black text-black-600", "{formatted_price} kr" }
                p { class: "text-gray-500 text-xs font-medium", "Jfr pris {formatted_comparison}" }
            }

            div { class: "flex items-center gap-2 mt-auto",
                if in_stock == 0 {
                    button {
                        class: "flex-grow bg-gray-200 text-gray-400 font-bold py-2 rounded-full cursor-not-allowed flex justify-center items-center gap-2",
                        disabled: true,
                        i { class: "fas fa-ban text-sm" }
                        "Slut i lager"
                    }
                } else if quantity == 0 {
                    button {
                        class: "flex-grow bg-green-700 text-white font-bold py-2 rounded-full hover:bg-green-800 transition flex justify-center items-center gap-2",
                        onclick: move |_| {
                            global_state
                                .write()
                                .add_to_cart(
                                    product_id,
                                    product_name.clone(),
                                    product_price,
                                    product_image.clone(),
                                );
                            if let Some(cid) = global_state.read().customer_id() {
                                let new_qty = global_state
                                    .read()
                                    .cart
                                    .iter()
                                    .find(|i| i.product_id == product_id)
                                    .map(|i| i.quantity)
                                    .unwrap_or(1);
                                #[allow(unused_results)]
                                spawn(async move {
                                    let pid = crate::database::Id::<
                                        crate::database::Product,
                                    >::from(product_id);
                                    drop(
                                        crate::database::cart::set_in_shopping_cart(cid, pid, new_qty).await,
                                    );
                                });
                            }
                        },
                        i { class: "fas fa-shopping-cart" }
                    }
                } else {
                    div { class: "flex-grow flex items-center justify-between bg-green-100 rounded-full overflow-hidden",
                        button {
                            class: "px-4 py-2 bg-green-700 text-white font-bold",
                            onclick: move |_| {
                                let new_qty = quantity - 1;
                                global_state.write().set_quantity(product_id, new_qty);
                                if let Some(cid) = global_state.read().customer_id() {
                                    #[allow(unused_results)]
                                    spawn(async move {
                                        let pid = crate::database::Id::<
                                            crate::database::Product,
                                        >::from(product_id);
                                        drop(
                                            crate::database::cart::set_in_shopping_cart(cid, pid, new_qty).await,
                                        );
                                    });
                                }
                            },
                            i { class: "fas fa-minus" }
                        }
                        span { class: "font-bold text-green-900", "{quantity}" }
                        button {
                            class: "px-4 py-2 bg-green-700 text-white font-bold",
                            onclick: move |_| {
                                let new_qty = quantity + 1;
                                global_state.write().set_quantity(product_id, new_qty);
                                if let Some(cid) = global_state.read().customer_id() {
                                    #[allow(unused_results)]
                                    spawn(async move {
                                        let pid = crate::database::Id::<
                                            crate::database::Product,
                                        >::from(product_id);
                                        drop(
                                            crate::database::cart::set_in_shopping_cart(cid, pid, new_qty).await,
                                        );
                                    });
                                }
                            },
                            i { class: "fas fa-plus" }
                        }
                    }
                }

                button {
                    class: "p-2 transition-colors {heart_class} text-xl",
                    onclick: move |_| {
                        let customer_id = global_state.read().customer_id();
                        let new_state = !global_state.read().favorites.contains(&product_id);
                        let mut state = global_state.write();
                        if new_state {
                            state.favorites.push(product_id);
                        } else {
                            state.favorites.retain(|&x| x != product_id);
                        }
                        drop(state);
                        if let Some(cid) = customer_id {
                            #[allow(unused_results)]
                            spawn(async move {
                                let db_id = crate::database::Id::<
                                    crate::database::Product,
                                >::from(product_id);
                                let _unused = crate::database::products::set_favorite(
                                        cid,
                                        db_id,
                                        new_state,
                                    )
                                    .await;
                            });
                        }
                    },
                    if is_favorite {
                        i { class: "fa-solid fa-heart" }
                    } else {
                        i { class: "fa-regular fa-heart" }
                    }
                }
            }
        }
    }
}