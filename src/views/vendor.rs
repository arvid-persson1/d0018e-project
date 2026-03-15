#![allow(non_snake_case)]
use crate::Route;
use crate::database::categories::category_trees;
use crate::database::products::{
    add_stock, create_product, set_origin, set_overview, set_price, set_product_name,
    set_thumbnail, set_visibility, vendor_orders, vendor_products, set_status,
    OrderStatus, OrderVendorView, ProductOverviewVendor,
};
use crate::database::{Amount, Id, Url, Vendor as VendorEntity};
use crate::database::users::vendor_info;
use crate::state::GlobalState;
use dioxus::prelude::*;
use rust_decimal::Decimal;
use std::str::FromStr;
 
// ─── Order status badge ───────────────────────────────────────────────────────
 
#[component]
fn OrderStatusBadge(status: OrderStatus) -> Element {
    let (bg, label, icon) = match status {
        OrderStatus::Pending  => ("bg-amber-100 text-amber-800 border-amber-200",  "Väntar",   "fa-solid fa-clock"),
        OrderStatus::Shipped  => ("bg-blue-100 text-blue-800 border-blue-200",     "Skickad",  "fa-solid fa-truck"),
        OrderStatus::Received => ("bg-green-100 text-green-800 border-green-200",  "Mottagen", "fa-solid fa-circle-check"),
    };
    rsx! {
        span { class: "inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-bold border {bg}",
            i { class: "{icon} text-[10px]" }
            "{label}"
        }
    }
}
 
// ─── Vendor orders tab ────────────────────────────────────────────────────────
 
#[component]
fn VendorOrdersTab(vendor_id: Id<VendorEntity>) -> Element {
    let mut orders_resource = use_resource(move || async move {
        vendor_orders(vendor_id, 100, 0).await
    });
    let mut status_msg: Signal<Option<String>> = use_signal(|| None);
 
    let orders_read  = orders_resource.read();
    let is_loading   = orders_read.is_none();
    let err_str: Option<String> = orders_read.as_ref().and_then(|r| r.as_ref().err().map(|e| e.to_string()));
    let orders_list: Option<Vec<OrderVendorView>> = orders_read.as_ref().and_then(|r| r.as_ref().ok()).map(|v| v.to_vec());
    let is_empty     = orders_list.as_ref().map(|v| v.is_empty()).unwrap_or(false);
 
    rsx! {
        div { class: "space-y-4",
            if let Some(msg) = status_msg() {
                div { class: "bg-green-50 border border-green-200 rounded-xl p-3 text-sm text-green-800 flex items-center gap-2",
                    i { class: "fa-solid fa-check" }
                    "{msg}"
                }
            }

            if is_loading {
                p { class: "text-gray-400 animate-pulse", "Laddar ordrar..." }
            } else if let Some(err) = err_str {
                p { class: "text-red-400 text-sm", "Fel: {err}" }
            } else if is_empty {
                div { class: "text-center py-20 bg-white rounded-2xl border-2 border-dashed border-gray-200",
                    i { class: "fa-solid fa-inbox text-4xl text-gray-200 mb-3 block" }
                    p { class: "font-bold text-gray-400", "Inga ordrar ännu" }
                    p { class: "text-xs text-gray-400 mt-1",
                        "Ordrar visas här när kunder köper dina produkter."
                    }
                }
            } else if let Some(orders) = orders_list {
                div { class: "bg-white rounded-2xl shadow-sm overflow-hidden border border-gray-100",
                    div { class: "grid grid-cols-[1fr_auto_auto_auto_auto] gap-4 px-5 py-3 bg-gray-50 border-b text-xs font-bold text-gray-500 uppercase tracking-wide",
                        span { "Produkt" }
                        span { class: "text-center", "Antal" }
                        span { class: "text-center", "Datum" }
                        span { class: "text-center", "Status" }
                        span {}
                    }
                    div { class: "divide-y divide-gray-50",
                        for order in orders.iter() {
                            {
                                let order_id = order.id;
                                let status = order.status;
                                let changed = order.product_changed;
                                let t = order.time;
                                let date_str = format!("{:04}-{:02}-{:02}", t.year(), t.month() as u8, t.day());
                                rsx! {
                                    div { class: "grid grid-cols-[1fr_auto_auto_auto_auto] gap-4 px-5 py-4 items-center hover:bg-gray-50 transition",
                                        div { class: "min-w-0",
                                            Link {
                                                to: Route::Product {
                                                    id: order.product.into(),
                                                },
                                                class: "font-bold text-sm text-gray-900 truncate hover:text-green-700 transition block",
                                                "{order.product_name}"
                                            }
                                            if changed {
                                                p { class: "text-xs text-amber-600 mt-0.5",
                                                    i { class: "fa-solid fa-circle-exclamation mr-1" }
                                                    "Produkten har ändrats sedan order lades"
                                                }
                                            }
                                        }
                                        span { class: "text-sm text-gray-700 font-semibold text-center", "{order.number} st" }
                                        span { class: "text-xs text-gray-400 text-center", "{date_str}" }
                                        div { class: "flex flex-col gap-1 items-center",
                                            OrderStatusBadge { status }
                                            span { class: "inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-bold border bg-green-50 text-green-700 border-green-200",
                                                i { class: "fa-solid fa-check text-[10px]" }
                                                "Betald"
                                            }
                                        }
                                        div {
                                            if status == OrderStatus::Pending {
                                                button {
                                                    class: "text-xs bg-blue-600 text-white font-bold px-3 py-1.5 rounded-lg hover:bg-blue-700 transition whitespace-nowrap",
                                                    onclick: move |_| {
                                                        let mut sm = status_msg;
                                                        let mut r = orders_resource;
                                                        #[allow(unused_results)]
                                                        spawn(async move {
                                                            match set_status(order_id, OrderStatus::Shipped).await {
                                                                Ok(()) => {
                                                                    sm.set(Some("Order markerad som skickad.".into()));
                                                                    r.restart();
                                                                }
                                                                Err(e) => sm.set(Some(format!("Fel: {e}"))),
                                                            }
                                                        });
                                                    },
                                                    i { class: "fa-solid fa-truck mr-1" }
                                                    "Markera skickad"
                                                }
                                            } else {
                                                span { class: "text-xs text-gray-300 px-3", "—" }
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
    }
}
 
// ─── Add product modal ────────────────────────────────────────────────────────
 
#[component]
fn AddProductModal(vendor_id: Id<VendorEntity>, on_close: EventHandler<bool>) -> Element {
    let mut name        = use_signal(String::new);
    let mut thumbnail   = use_signal(String::new);
    let mut price_str   = use_signal(String::new);
    let mut overview    = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut origin      = use_signal(String::new);
    let mut category_id = use_signal(|| 0_i32);
    let mut amount_qty  = use_signal(|| "1".to_string());
    let mut amount_unit = use_signal(|| "kg".to_string());
    let mut error       = use_signal(|| None::<String>);
    let mut loading     = use_signal(|| false);
 
    let categories = use_resource(|| async move { category_trees().await.unwrap_or_default() });
 
    let flat_cats: Vec<(i32, String)> = match &*categories.read() {
        None => vec![],
        Some(trees) => {
            let mut out = Vec::new();
            for tree in trees.iter() {
                for sub in tree.subcategories.iter() {
                    for leaf in sub.subcategories.iter() {
                        out.push((leaf.id.get(), format!("{} › {} › {}", tree.name, sub.name, leaf.name)));
                    }
                    if sub.subcategories.is_empty() {
                        out.push((sub.id.get(), format!("{} › {}", tree.name, sub.name)));
                    }
                }
                if tree.subcategories.is_empty() {
                    out.push((tree.id.get(), tree.name.to_string()));
                }
            }
            out
        }
    };
 
    rsx! {
        div {
            class: "fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4",
            onclick: move |_| on_close.call(false),
            div {
                class: "bg-white rounded-2xl shadow-2xl w-full max-w-lg max-h-[90vh] overflow-y-auto",
                onclick: move |e| e.stop_propagation(),
                div { class: "p-6 border-b flex justify-between items-center",
                    h2 { class: "text-xl font-black text-gray-900",
                        i { class: "fa-solid fa-plus text-green-700 mr-2" }
                        "Lägg till produkt"
                    }
                    button {
                        class: "text-gray-400 hover:text-gray-600 transition",
                        onclick: move |_| on_close.call(false),
                        i { class: "fa-solid fa-xmark text-xl" }
                    }
                }
                div { class: "p-6 space-y-4",
                    div {
                        label { class: "block text-sm font-bold text-gray-700 mb-1", "Produktnamn *" }
                        input {
                            r#type: "text",
                            class: "w-full border border-gray-200 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-green-500",
                            placeholder: "t.ex. Ekologiska äpplen",
                            value: "{name}",
                            oninput: move |e| name.set(e.value()),
                        }
                    }
                    div {
                        label { class: "block text-sm font-bold text-gray-700 mb-1", "Bild-URL *" }
                        input {
                            r#type: "url",
                            class: "w-full border border-gray-200 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-green-500",
                            placeholder: "https://...",
                            value: "{thumbnail}",
                            oninput: move |e| thumbnail.set(e.value()),
                        }
                    }
                    div {
                        label { class: "block text-sm font-bold text-gray-700 mb-1", "Pris (kr) *" }
                        input {
                            r#type: "number",
                            step: "0.01",
                            min: "0",
                            class: "w-full border border-gray-200 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-green-500",
                            placeholder: "29.90",
                            value: "{price_str}",
                            oninput: move |e| price_str.set(e.value()),
                        }
                    }
                    div {
                        label { class: "block text-sm font-bold text-gray-700 mb-1", "Kategori *" }
                        select {
                            class: "w-full border border-gray-200 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-green-500 bg-white",
                            onchange: move |e| {
                                if let Ok(id) = e.value().parse::<i32>() {
                                    category_id.set(id);
                                }
                            },
                            option {
                                value: "0",
                                disabled: true,
                                selected: category_id() == 0,
                                "Välj kategori..."
                            }
                            for (cid , lbl) in flat_cats.iter() {
                                option { value: "{cid}", "{lbl}" }
                            }
                        }
                    }
                    div { class: "flex gap-3",
                        div { class: "flex-1",
                            label { class: "block text-sm font-bold text-gray-700 mb-1",
                                "Mängd *"
                            }
                            input {
                                r#type: "number",
                                step: "0.01",
                                min: "0.01",
                                class: "w-full border border-gray-200 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-green-500",
                                placeholder: "1",
                                value: "{amount_qty}",
                                oninput: move |e| amount_qty.set(e.value()),
                            }
                        }
                        div { class: "flex-1",
                            label { class: "block text-sm font-bold text-gray-700 mb-1",
                                "Enhet *"
                            }
                            select {
                                class: "w-full border border-gray-200 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-green-500 bg-white",
                                onchange: move |e| amount_unit.set(e.value()),
                                option { value: "kg", "kg" }
                                option { value: "g", "g" }
                                option { value: "l", "l" }
                                option { value: "ml", "ml" }
                                option { value: "st", "st" }
                                option { value: "förpackning", "förpackning" }
                            }
                        }
                    }
                    div {
                        label { class: "block text-sm font-bold text-gray-700 mb-1", "Ursprung" }
                        input {
                            r#type: "text",
                            class: "w-full border border-gray-200 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-green-500",
                            placeholder: "t.ex. Sverige",
                            value: "{origin}",
                            oninput: move |e| origin.set(e.value()),
                        }
                    }
                    div {
                        label { class: "block text-sm font-bold text-gray-700 mb-1", "Kortbeskrivning *" }
                        input {
                            r#type: "text",
                            class: "w-full border border-gray-200 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-green-500",
                            placeholder: "En mening om produkten",
                            value: "{overview}",
                            oninput: move |e| overview.set(e.value()),
                        }
                    }
                    div {
                        label { class: "block text-sm font-bold text-gray-700 mb-1", "Beskrivning" }
                        textarea {
                            class: "w-full border border-gray-200 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-green-500 resize-none",
                            rows: 3,
                            placeholder: "Mer detaljerad beskrivning...",
                            value: "{description}",
                            oninput: move |e| description.set(e.value()),
                        }
                    }
                    div { class: "bg-amber-50 border border-amber-200 rounded-lg p-3 text-sm text-amber-800",
                        i { class: "fa-solid fa-circle-info mr-2" }
                        "Produkten skapas utan lager. Fyll på lagret efteråt via \"Redigera\"."
                    }
                    if let Some(err) = error() {
                        p { class: "text-red-500 text-sm bg-red-50 border border-red-200 rounded-lg p-3",
                            i { class: "fa-solid fa-triangle-exclamation mr-2" }
                            "{err}"
                        }
                    }
                    div { class: "flex gap-3 pt-2",
                        button {
                            class: "flex-1 border border-gray-200 text-gray-600 font-bold py-3 rounded-xl hover:bg-gray-50 transition",
                            onclick: move |_| on_close.call(false),
                            "Avbryt"
                        }
                        button {
                            class: if loading() { "flex-1 bg-gray-300 text-gray-500 font-black py-3 rounded-xl cursor-not-allowed" } else { "flex-1 bg-green-700 text-white font-black py-3 rounded-xl hover:bg-green-800 transition" },
                            disabled: loading(),
                            onclick: move |_| {
                                let name_val = name().trim().to_string();
                                let thumb_val = thumbnail().trim().to_string();
                                let price_val = price_str().trim().to_string();
                                let overview_val = overview().trim().to_string();
                                let cat_val = category_id();
                                if name_val.is_empty() || thumb_val.is_empty() || price_val.is_empty()
                                    || overview_val.is_empty() || cat_val == 0
                                {
                                    error.set(Some("Fyll i alla obligatoriska fält (*)".to_string()));
                                    return;
                                }
                                let Ok(price_dec) = Decimal::from_str(&price_val) else {
                                    error.set(Some("Ogiltigt pris".to_string()));
                                    return;
                                };
                                let Ok(qty_dec) = Decimal::from_str(&amount_qty()) else {
                                    error.set(Some("Ogiltig mängd".to_string()));
                                    return;
                                };
                                let unit_str = amount_unit();
                                let unit: Option<Box<str>> = if unit_str.is_empty() {
                                    None
                                } else {
                                    Some(unit_str.into())
                                };
                                let Some(amount) = Amount::new(qty_dec, unit) else {
                                    error.set(Some("Ogiltig mängd eller enhet".to_string()));
                                    return;
                                };
                                let thumb_url = Url::from(thumb_val);
                                let desc_val = description().trim().to_string();
                                let origin_val = origin().trim().to_string();
                                let cat_id: Id<crate::database::Category> = cat_val.into();
                                error.set(None);
                                loading.set(true);
                                #[allow(unused_results)]
                                spawn(async move {
                                    match create_product(
                                            vendor_id,
                                            name_val.into(),
                                            thumb_url,
                                            Box::new([]),
                                            price_dec,
                                            overview_val.into(),
                                            desc_val.into(),
                                            cat_id,
                                            amount,
                                            origin_val.into(),
                                        )
                                        .await
                                    {
                                        Err(e) => {
                                            error.set(Some(e.to_string()));
                                            loading.set(false);
                                        }
                                        Ok(()) => on_close.call(true),
                                    }
                                });
                            },
                            if loading() {
                                i { class: "fa-solid fa-spinner fa-spin mr-2" }
                                "Sparar..."
                            } else {
                                i { class: "fa-solid fa-plus mr-2" }
                                "Lägg till produkt"
                            }
                        }
                    }
                }
            }
        }
    }
}
 
// ─── Edit product modal ───────────────────────────────────────────────────────
 
#[component]
fn EditProductModal(product: ProductOverviewVendor, on_close: EventHandler<bool>) -> Element {
    let product_id    = product.id;
    let current_stock = product.in_stock;
    let mut name          = use_signal(|| product.name.to_string());
    let mut thumbnail     = use_signal(|| product.thumbnail.to_string());
    let mut price_str     = use_signal(|| product.price.to_string());
    let mut overview_text = use_signal(|| product.overview.to_string());
    let mut origin_text   = use_signal(|| product.origin.to_string());
    let mut stock_add     = use_signal(|| "0".to_string());
    let mut visible       = use_signal(|| true);
    let mut error         = use_signal(|| None::<String>);
    let mut loading       = use_signal(|| false);
    let mut saved         = use_signal(|| false);
 
    rsx! {
        div {
            class: "fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4",
            onclick: move |_| on_close.call(saved()),
            div {
                class: "bg-white rounded-2xl shadow-2xl w-full max-w-lg max-h-[90vh] overflow-y-auto",
                onclick: move |e| e.stop_propagation(),
                div { class: "p-6 border-b flex justify-between items-center",
                    h2 { class: "text-xl font-black text-gray-900",
                        i { class: "fa-solid fa-pen text-green-700 mr-2" }
                        "Redigera produkt"
                    }
                    button {
                        class: "text-gray-400 hover:text-gray-600 transition",
                        onclick: move |_| on_close.call(saved()),
                        i { class: "fa-solid fa-xmark text-xl" }
                    }
                }
                div { class: "p-6 space-y-4",
                    if saved() {
                        p { class: "text-green-700 text-sm bg-green-50 border border-green-200 rounded-lg p-3",
                            i { class: "fa-solid fa-check mr-2" }
                            "Ändringar sparade!"
                        }
                    }
                    div {
                        label { class: "block text-sm font-bold text-gray-700 mb-1", "Produktnamn" }
                        input {
                            r#type: "text",
                            class: "w-full border border-gray-200 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-green-500",
                            value: "{name}",
                            oninput: move |e| name.set(e.value()),
                        }
                    }
                    div {
                        label { class: "block text-sm font-bold text-gray-700 mb-1", "Bild-URL" }
                        input {
                            r#type: "url",
                            class: "w-full border border-gray-200 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-green-500",
                            value: "{thumbnail}",
                            oninput: move |e| thumbnail.set(e.value()),
                        }
                    }
                    div { class: "flex gap-3",
                        div { class: "flex-1",
                            label { class: "block text-sm font-bold text-gray-700 mb-1",
                                "Pris (kr)"
                            }
                            input {
                                r#type: "number",
                                step: "0.01",
                                min: "0",
                                class: "w-full border border-gray-200 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-green-500",
                                value: "{price_str}",
                                oninput: move |e| price_str.set(e.value()),
                            }
                        }
                        div { class: "flex-1",
                            label { class: "block text-sm font-bold text-gray-700 mb-1",
                                "Fyll på lager"
                                span { class: "text-gray-400 font-normal ml-1", "(nu: {current_stock})" }
                            }
                            input {
                                r#type: "number",
                                min: "0",
                                class: "w-full border border-gray-200 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-green-500",
                                placeholder: "0",
                                value: "{stock_add}",
                                oninput: move |e| stock_add.set(e.value()),
                            }
                        }
                    }
                    div {
                        label { class: "block text-sm font-bold text-gray-700 mb-1", "Kortbeskrivning" }
                        input {
                            r#type: "text",
                            class: "w-full border border-gray-200 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-green-500",
                            value: "{overview_text}",
                            oninput: move |e| overview_text.set(e.value()),
                        }
                    }
                    div {
                        label { class: "block text-sm font-bold text-gray-700 mb-1", "Ursprung" }
                        input {
                            r#type: "text",
                            class: "w-full border border-gray-200 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-green-500",
                            value: "{origin_text}",
                            oninput: move |e| origin_text.set(e.value()),
                        }
                    }
                    div { class: "flex items-center gap-3",
                        input {
                            r#type: "checkbox",
                            class: "w-4 h-4 accent-green-700",
                            id: "visible-check",
                            checked: visible(),
                            onchange: move |e| visible.set(e.checked()),
                        }
                        label {
                            r#for: "visible-check",
                            class: "text-sm font-bold text-gray-700 cursor-pointer",
                            "Synlig i butiken"
                        }
                    }
                    if let Some(err) = error() {
                        p { class: "text-red-500 text-sm bg-red-50 border border-red-200 rounded-lg p-3",
                            i { class: "fa-solid fa-triangle-exclamation mr-2" }
                            "{err}"
                        }
                    }
                    div { class: "flex gap-3 pt-2",
                        button {
                            class: "flex-1 border border-gray-200 text-gray-600 font-bold py-3 rounded-xl hover:bg-gray-50 transition",
                            onclick: move |_| on_close.call(saved()),
                            "Stäng"
                        }
                        button {
                            class: if loading() { "flex-1 bg-gray-300 text-gray-500 font-black py-3 rounded-xl cursor-not-allowed" } else { "flex-1 bg-green-700 text-white font-black py-3 rounded-xl hover:bg-green-800 transition" },
                            disabled: loading(),
                            onclick: move |_| {
                                let name_val = name().trim().to_string();
                                let thumb_val = thumbnail().trim().to_string();
                                let price_val = price_str().trim().to_string();
                                let overview_val = overview_text().trim().to_string();
                                let origin_val = origin_text().trim().to_string();
                                let visible_val = visible();
                                let stock_to_add = stock_add().trim().parse::<u32>().unwrap_or(0);
                                let Ok(price_dec) = Decimal::from_str(&price_val) else {
                                    error.set(Some("Ogiltigt pris".to_string()));
                                    return;
                                };
                                error.set(None);
                                loading.set(true);
                                saved.set(false);
                                #[allow(unused_results)]
                                let _task = spawn(async move {
                                    let mut errs: Vec<String> = vec![];
                                    if let Err(e) = set_product_name(product_id, name_val.into()).await {
                                        errs.push(e.to_string());
                                    }
                                    if let Err(e) = set_thumbnail(product_id, Url::from(thumb_val)).await {
                                        errs.push(e.to_string());
                                    }
                                    if let Err(e) = set_price(product_id, price_dec).await {
                                        errs.push(e.to_string());
                                    }
                                    if let Err(e) = set_overview(product_id, overview_val.into()).await {
                                        errs.push(e.to_string());
                                    }
                                    if let Err(e) = set_origin(product_id, origin_val.into()).await {
                                        errs.push(e.to_string());
                                    }
                                    if let Err(e) = set_visibility(product_id, visible_val).await {
                                        errs.push(e.to_string());
                                    }
                                    if let Some(n) = std::num::NonZeroU32::new(stock_to_add) {
                                        if let Err(e) = add_stock(product_id, n, None).await.map(|_| ()) {
                                            errs.push(e.to_string());
                                        }
                                    }
                                    loading.set(false);
                                    if errs.is_empty() {
                                        saved.set(true);
                                    } else {
                                        error.set(Some(errs.join(", ")));
                                    }
                                });
                            },
                            if loading() {
                                i { class: "fa-solid fa-spinner fa-spin mr-2" }
                                "Sparar..."
                            } else {
                                i { class: "fa-solid fa-floppy-disk mr-2" }
                                "Spara ändringar"
                            }
                        }
                    }
                }
            }
        }
    }
}
 
// ─── Vendor page ──────────────────────────────────────────────────────────────
 
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
 
    let info_resource         = use_resource(move || async move { vendor_info(id).await });
    let mut products_resource = use_resource(move || async move {
        vendor_products(customer_id, id, 100, 0, is_own_profile).await
    });
    let mut show_add_modal = use_signal(|| false);
    let mut edit_product: Signal<Option<ProductOverviewVendor>> = use_signal(|| None);
    let mut active_tab = use_signal(|| 0_u8);
 
    // Resolve outside rsx! — no match inside macros
    let info_read    = info_resource.read();
    let info_loading = info_read.is_none();
    let info_err: Option<String>  = info_read.as_ref().and_then(|r| r.as_ref().err().map(|e| e.to_string()));
    let info_data: Option<(String, String)> = info_read.as_ref()
        .and_then(|r| r.as_ref().ok())
        .map(|i| (i.display_name.to_string(), i.description.to_string()));
 
    let prod_read    = products_resource.read();
    let prod_loading = prod_read.is_none();
    let prod_err: Option<String>  = prod_read.as_ref().and_then(|r| r.as_ref().err().map(|e| e.to_string()));
    let products_vec: Option<Vec<ProductOverviewVendor>> = prod_read.as_ref()
        .and_then(|r| r.as_ref().ok())
        .map(|v| v.to_vec());
    let prod_empty = products_vec.as_ref().map(|v| v.is_empty()).unwrap_or(false);
 
    rsx! {
        div { class: "min-h-screen bg-gray-50",
            div { class: "max-w-6xl mx-auto p-6",
                Link {
                    to: Route::Home {},
                    class: "text-green-700 hover:text-green-900 font-bold flex items-center gap-2 mb-4 transition-colors",
                    i { class: "fa-solid fa-arrow-left" }
                    "Tillbaka till start"
                }

                // ── Vendor header ──
                if info_loading {
                    p { class: "text-gray-400 animate-pulse mb-8", "Laddar..." }
                } else if let Some(err) = info_err {
                    p { class: "text-red-400 mb-8", "Fel: {err}" }
                } else if let Some((display_name, description)) = info_data {
                    div { class: "flex items-center gap-6 mb-6",
                        div { class: "w-20 h-20 rounded-full bg-green-100 flex items-center justify-center",
                            i { class: "fa-solid fa-store text-3xl text-green-700" }
                        }
                        div {
                            h1 { class: "text-3xl font-black text-gray-900", "{display_name}" }
                            p { class: "text-gray-500 text-sm mt-1", "{description}" }
                        }
                        if is_own_profile {
                            div { class: "ml-auto",
                                span { class: "text-xs text-gray-400 bg-gray-100 px-3 py-1 rounded-full",
                                    "Din butik"
                                }
                            }
                        }
                    }
                }

                // ── Tabs ──
                if is_own_profile {
                    div { class: "flex gap-2 mb-6 border-b",
                        button {
                            class: if active_tab() == 0 { "px-4 py-2 font-bold text-green-700 border-b-2 border-green-700" } else { "px-4 py-2 text-gray-500 hover:text-gray-700" },
                            onclick: move |_| active_tab.set(0),
                            i { class: "fa-solid fa-tag mr-2" }
                            "Produkter"
                        }
                        button {
                            class: if active_tab() == 1 { "px-4 py-2 font-bold text-green-700 border-b-2 border-green-700" } else { "px-4 py-2 text-gray-500 hover:text-gray-700" },
                            onclick: move |_| active_tab.set(1),
                            i { class: "fa-solid fa-bag-shopping mr-2" }
                            "Ordrar"
                        }
                    }
                }

                // ── Products tab ──
                if active_tab() == 0 {
                    div { class: "flex items-center justify-between mb-4",
                        h2 { class: "text-xl font-black text-gray-900",
                            i { class: "fa-solid fa-tag text-green-700 mr-2" }
                            if is_own_profile {
                                "Dina produkter"
                            } else {
                                "Produkter"
                            }
                        }
                        if is_own_profile {
                            button {
                                class: "flex items-center gap-2 bg-green-700 text-white font-black px-5 py-2 rounded-full hover:bg-green-800 transition text-sm",
                                onclick: move |_| show_add_modal.set(true),
                                i { class: "fa-solid fa-plus" }
                                "Lägg till produkt"
                            }
                        }
                    }

                    if prod_loading {
                        p { class: "text-gray-400 animate-pulse", "Laddar..." }
                    } else if let Some(err) = prod_err {
                        p { class: "text-red-400 text-sm", "Fel: {err}" }
                    } else if prod_empty {
                        p { class: "text-gray-400 text-sm", "Inga produkter ännu." }
                    } else if let Some(products) = products_vec {
                        div { class: "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4",
                            for p in products.iter() {
                                {
                                    let p_clone = p.clone();
                                    rsx! {
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
                                                    } else {
                                                        p { class: "text-gray-400 text-xs mt-1", "{p.in_stock} i lager" }
                                                    }
                                                }
                                            }
                                            if is_own_profile {
                                                div { class: "px-3 pb-3",
                                                    button {
                                                        class: "w-full text-xs border border-gray-200 rounded-lg py-2 hover:bg-gray-50 text-gray-600 font-bold transition",
                                                        onclick: move |_| edit_product.set(Some(p_clone.clone())),
                                                        i { class: "fa-solid fa-pen mr-1" }
                                                        "Redigera"
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

                // ── Orders tab ──
                if active_tab() == 1 && is_own_profile {
                    VendorOrdersTab { vendor_id: id }
                }
            }
        }

        if show_add_modal() {
            AddProductModal {
                vendor_id: id,
                on_close: move |success: bool| {
                    show_add_modal.set(false);
                    if success {
                        products_resource.restart();
                    }
                },
            }
        }

        if let Some(p) = edit_product() {
            EditProductModal {
                product: p,
                on_close: move |changed: bool| {
                    edit_product.set(None);
                    if changed {
                        products_resource.restart();
                    }
                },
            }
        }
    }
}