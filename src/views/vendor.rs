#![allow(non_snake_case)]
use crate::Route;
use crate::database::categories::category_trees;
use crate::database::products::{
    create_product, vendor_products,
    add_stock, set_price, set_product_name, set_visibility,
    set_thumbnail, set_overview, set_origin,
    ProductOverviewVendor,
};
use crate::database::{Amount, Id, Url, Vendor as VendorEntity};
use crate::database::users::vendor_info;
use crate::state::GlobalState;
use dioxus::prelude::*;
use rust_decimal::Decimal;
use std::str::FromStr;
 
/// Möjliga orderstatusar. Lägg till DB-kolumn `order_status` i `orders` för att aktivera.
#[derive(Clone, PartialEq, Debug)]
enum OrderStatus {
    Placed,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
}

impl OrderStatus {
    fn label(&self) -> &str {
        match self {
            Self::Placed     => "Lagd",
            Self::Processing => "Behandlas",
            Self::Shipped    => "Skickad",
            Self::Delivered  => "Levererad",
            Self::Cancelled  => "Avbruten",
        }
    }
    fn value(&self) -> &str {
        match self {
            Self::Placed     => "placed",
            Self::Processing => "processing",
            Self::Shipped    => "shipped",
            Self::Delivered  => "delivered",
            Self::Cancelled  => "cancelled",
        }
    }
    fn from_str(s: &str) -> Self {
        match s {
            "processing" => Self::Processing,
            "shipped"    => Self::Shipped,
            "delivered"  => Self::Delivered,
            "cancelled"  => Self::Cancelled,
            _            => Self::Placed,
        }
    }
    fn badge_class(&self) -> &str {
        match self {
            Self::Placed     => "bg-gray-100 text-gray-700 border-gray-200",
            Self::Processing => "bg-yellow-100 text-yellow-800 border-yellow-200",
            Self::Shipped    => "bg-blue-100 text-blue-800 border-blue-200",
            Self::Delivered  => "bg-green-100 text-green-800 border-green-200",
            Self::Cancelled  => "bg-red-100 text-red-800 border-red-200",
        }
    }
    fn icon(&self) -> &str {
        match self {
            Self::Placed     => "fa-solid fa-clock",
            Self::Processing => "fa-solid fa-gear",
            Self::Shipped    => "fa-solid fa-truck",
            Self::Delivered  => "fa-solid fa-circle-check",
            Self::Cancelled  => "fa-solid fa-xmark",
        }
    }
}
 
/// Möjliga betalningsstatusar. Lägg till DB-kolumn `payment_status` i `orders` för att aktivera.
#[derive(Clone, PartialEq, Debug)]
enum PaymentStatus {
    Paid,
    Pending,
    Refunded,
}
 
impl PaymentStatus {
    fn label(&self) -> &str {
        match self {
            Self::Paid     => "Betald",
            Self::Pending  => "Väntar",
            Self::Refunded => "Återbetald",
        }
    }
    fn value(&self) -> &str {
        match self {
            Self::Paid     => "paid",
            Self::Pending  => "pending",
            Self::Refunded => "refunded",
        }
    }
    fn from_str(s: &str) -> Self {
        match s {
            "pending"  => Self::Pending,
            "refunded" => Self::Refunded,
            _          => Self::Paid,
        }
    }
    fn badge_class(&self) -> &str {
        match self {
            Self::Paid     => "bg-green-50 text-green-700 border-green-200",
            Self::Pending  => "bg-amber-50 text-amber-700 border-amber-200",
            Self::Refunded => "bg-purple-50 text-purple-700 border-purple-200",
        }
    }
    fn icon(&self) -> &str {
        match self {
            Self::Paid     => "fa-solid fa-check",
            Self::Pending  => "fa-solid fa-hourglass-half",
            Self::Refunded => "fa-solid fa-rotate-left",
        }
    }
}
 
// ─── Fake order struct (ersätts med DB-data när kolumner finns) ──────────────
 
/// Placeholder-order för vendor-vyn tills `order_status` / `payment_status` finns i DB.
#[derive(Clone, Debug, PartialEq)]
struct VendorOrderRow {
    order_id: i32,
    product_name: Box<str>,
    thumbnail: Box<str>,
    customer_display: Box<str>, // "Kund #N" — vi visar inte kundens namn av integritetsskäl
    number: u32,
    paid: Decimal,
    time: String,
    order_status: OrderStatus,
    payment_status: PaymentStatus,
}
 
// ─── Modaler ─────────────────────────────────────────────────────────────────
 
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
                            for (cid , label) in flat_cats.iter() {
                                option { value: "{cid}", "{label}" }
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
                                        Ok(()) => {
                                            on_close.call(true);
                                        }
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
 
#[component]
fn EditProductModal(product: ProductOverviewVendor, on_close: EventHandler<bool>) -> Element {
    let product_id   = product.id;
    let current_stock = product.in_stock;
    let mut name         = use_signal(|| product.name.to_string());
    let mut thumbnail    = use_signal(|| product.thumbnail.to_string());
    let mut price_str    = use_signal(|| product.price.to_string());
    let mut overview_text = use_signal(|| product.overview.to_string());
    let mut origin_text  = use_signal(|| product.origin.to_string());
    let mut stock_add    = use_signal(|| "0".to_string());
    let mut visible      = use_signal(|| true);
    let mut error        = use_signal(|| None::<String>);
    let mut loading      = use_signal(|| false);
    let mut saved        = use_signal(|| false);
 
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
 

 
/// Modal som låter vendorn uppdatera order- och betalningsstatus.
/// TODO: Koppla till DB-server-functions när 'order_status' + 'payment_status' kolumner finns.
#[component]
fn UpdateOrderStatusModal(
    order: VendorOrderRow,
    on_close: EventHandler<VendorOrderRow>,
) -> Element {
    // Lagra order i en signal så att alla closures kan läsa den
    let order_signal       = use_signal(|| order);
    let mut order_status   = use_signal(|| order_signal.read().order_status.clone());
    let mut payment_status = use_signal(|| order_signal.read().payment_status.clone());
    let mut saved          = use_signal(|| false);
 
    rsx! {
        div {
            class: "fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4",
            onclick: move |_| {
                on_close
                    .call(VendorOrderRow {
                        order_status: order_status(),
                        payment_status: payment_status(),
                        ..order_signal.read().clone()
                    });
            },
            div {
                class: "bg-white rounded-2xl shadow-2xl w-full max-w-md",
                onclick: move |e| e.stop_propagation(),

                // Header
                div { class: "p-5 border-b flex justify-between items-center",
                    h2 { class: "text-lg font-black text-gray-900",
                        i { class: "fa-solid fa-pen-to-square text-green-700 mr-2" }
                        "Uppdatera orderstatus"
                    }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| {
                            on_close
                                .call(VendorOrderRow {
                                    order_status: order_status(),
                                    payment_status: payment_status(),
                                    ..order_signal.read().clone()
                                });
                        },
                        i { class: "fa-solid fa-xmark text-xl" }
                    }
                }

                div { class: "p-5 space-y-5",
                    // Produktinfo
                    div { class: "flex items-center gap-3 bg-gray-50 rounded-xl p-3",
                        img {
                            src: "{order_signal.read().thumbnail}",
                            class: "w-12 h-12 rounded-lg object-cover",
                            alt: "{order_signal.read().product_name}",
                        }
                        div {
                            p { class: "font-bold text-sm text-gray-900",
                                "{order_signal.read().product_name}"
                            }
                            p { class: "text-xs text-gray-500",
                                "{order_signal.read().customer_display} · {order_signal.read().number} st · {order_signal.read().paid:.2} kr"
                            }
                        }
                    }

                    // Orderstatus-väljare
                    div {
                        label { class: "block text-sm font-bold text-gray-700 mb-2",
                            i { class: "fa-solid fa-truck-fast mr-1.5 text-green-700" }
                            "Orderstatus"
                        }
                        div { class: "grid grid-cols-2 gap-2",
                            for status_val in ["placed", "processing", "shipped", "delivered", "cancelled"] {
                                {
                                    let s = OrderStatus::from_str(status_val);
                                    let is_active = order_status() == s;
                                    let badge = s.badge_class();
                                    let icon = s.icon();
                                    let label = s.label().to_string();
                                    rsx! {
                                        button {
                                            class: if is_active { format!(
                                                "flex items-center gap-2 px-3 py-2 rounded-lg border-2 border-green-500 {badge} font-bold text-sm transition",
                                            ) } else { format!(
                                                "flex items-center gap-2 px-3 py-2 rounded-lg border border-gray-200 hover:border-gray-300 text-gray-600 text-sm transition",
                                            ) },
                                            onclick: move |_| {
                                                order_status.set(OrderStatus::from_str(status_val));
                                                saved.set(false);
                                            },
                                            i { class: "{icon} text-xs" }
                                            "{label}"
                                            if is_active {
                                                i { class: "fa-solid fa-check ml-auto text-green-600 text-xs" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Betalningsstatus-väljare
                    div {
                        label { class: "block text-sm font-bold text-gray-700 mb-2",
                            i { class: "fa-solid fa-credit-card mr-1.5 text-green-700" }
                            "Betalningsstatus"
                        }
                        div { class: "flex gap-2",
                            for status_val in ["paid", "pending", "refunded"] {
                                {
                                    let s = PaymentStatus::from_str(status_val);
                                    let is_active = payment_status() == s;
                                    let badge = s.badge_class();
                                    let icon = s.icon();
                                    let label = s.label().to_string();
                                    rsx! {
                                        button {
                                            class: if is_active { format!(
                                                "flex-1 flex items-center justify-center gap-1.5 px-3 py-2 rounded-lg border-2 border-green-500 {badge} font-bold text-sm transition",
                                            ) } else { format!(
                                                "flex-1 flex items-center justify-center gap-1.5 px-3 py-2 rounded-lg border border-gray-200 hover:border-gray-300 text-gray-600 text-sm transition",
                                            ) },
                                            onclick: move |_| {
                                                payment_status.set(PaymentStatus::from_str(status_val));
                                                saved.set(false);
                                            },
                                            i { class: "{icon} text-xs" }
                                            "{label}"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // TODO: info-banner
                    div { class: "bg-amber-50 border border-amber-200 rounded-lg p-3 text-xs text-amber-800",
                        i { class: "fa-solid fa-circle-info mr-1.5" }
                        "Status sparas lokalt tills "
                        code { class: "font-mono", "order_status" }
                        " och "
                        code { class: "font-mono", "payment_status" }
                        " kolumner lagts till i "
                        code { class: "font-mono", "orders" }
                        "-tabellen."
                    }

                    if saved() {
                        div { class: "bg-green-50 border border-green-200 rounded-lg p-3 text-sm text-green-800",
                            i { class: "fa-solid fa-check mr-2" }
                            "Status uppdaterad!"
                        }
                    }

                    // Knappar
                    div { class: "flex gap-3",
                        button {
                            class: "flex-1 border border-gray-200 text-gray-600 font-bold py-2.5 rounded-xl hover:bg-gray-50 transition text-sm",
                            onclick: move |_| {
                                on_close
                                    .call(VendorOrderRow {
                                        order_status: order_status(),
                                        payment_status: payment_status(),
                                        ..order_signal.read().clone()
                                    });
                            },
                            "Stäng"
                        }
                        button {
                            class: "flex-1 bg-green-700 text-white font-black py-2.5 rounded-xl hover:bg-green-800 transition text-sm",
                            onclick: move |_| {
                                // TODO: Anropa server-function när DB-kolumner finns:
                                // spawn(async move {
                                //     let _ = update_order_status(order_id, order_status().value()).await;
                                //     let _ = update_payment_status(order_id, payment_status().value()).await;
                                // });
                                saved.set(true);
                            },
                            i { class: "fa-solid fa-floppy-disk mr-2" }
                            "Spara status"
                        }
                    }
                }
            }
        }
    }
}
 
// Vendor orders tab
 
/// Placeholder ordrar för vendor-vyn
/// TODO: Ersätt med riktig DB-query när 'order_status' + 'payment_status' finns
fn placeholder_vendor_orders() -> Vec<VendorOrderRow> {
    vec![]
}
 
/// Vendor-ordertabell
#[component]
fn VendorOrdersTab() -> Element {
    let mut orders: Signal<Vec<VendorOrderRow>> =
        use_signal(placeholder_vendor_orders);
    let mut editing_order: Signal<Option<usize>> = use_signal(|| None);
 
    rsx! {
        div {
            // Info-banner
            div { class: "mb-4 bg-amber-50 border border-amber-200 rounded-xl p-3 flex items-start gap-2 text-sm text-amber-800",
                i { class: "fa-solid fa-circle-info mt-0.5 shrink-0" }
                div {
                    p { class: "font-semibold", "Orderstatus är redo, väntar på databas" }
                    p { class: "text-xs mt-0.5 text-amber-700",
                        "Lägg till "
                        code { class: "font-mono bg-amber-100 px-1 rounded", "order_status" }
                        " och "
                        code { class: "font-mono bg-amber-100 px-1 rounded", "payment_status" }
                        " i tabellen "
                        code { class: "font-mono bg-amber-100 px-1 rounded", "orders" }
                        " för att aktivera"
                    }
                }
            }

            if orders.read().is_empty() {
                div { class: "text-center py-20 bg-white rounded-2xl border-2 border-dashed border-gray-200",
                    i { class: "fa-solid fa-inbox text-4xl text-gray-200 mb-3" }
                    p { class: "font-bold text-gray-400", "Inga ordrar ännu" }
                    p { class: "text-xs text-gray-400 mt-1",
                        "Ordrar kommer visas här när kunder köper dina produkter."
                    }
                }
            } else {
                // Ordertabell
                div { class: "bg-white rounded-2xl shadow-sm overflow-hidden border border-gray-100",
                    // Header
                    div { class: "grid grid-cols-[1fr_auto_auto_auto_auto] gap-4 px-5 py-3 bg-gray-50 border-b text-xs font-bold text-gray-500 uppercase tracking-wide",
                        span { "Produkt / Kund" }
                        span { class: "text-center", "Antal" }
                        span { class: "text-center", "Betalt" }
                        span { class: "text-center", "Status" }
                        span {}
                    }
                    // Rader
                    div { class: "divide-y divide-gray-50",
                        for (idx , order) in orders.read().iter().enumerate() {
                            {
                                let os = order.order_status.clone();
                                let ps = order.payment_status.clone();
                                rsx! {
                                    div { class: "grid grid-cols-[1fr_auto_auto_auto_auto] gap-4 px-5 py-4 items-center hover:bg-gray-50 transition",
                                        // Produkt & kund
                                        div { class: "flex items-center gap-3 min-w-0",
                                            img {
                                                src: "{order.thumbnail}",
                                                class: "w-10 h-10 rounded-lg object-cover shrink-0",
                                                alt: "{order.product_name}",
                                            }
                                            div { class: "min-w-0",
                                                p { class: "font-bold text-sm text-gray-900 truncate", "{order.product_name}" }
                                                p { class: "text-xs text-gray-400", "{order.customer_display} · {order.time}" }
                                            }
                                        }
                                        // Antal
                                        span { class: "text-sm text-gray-700 font-semibold text-center", "{order.number} st" }
                                        // Betalt
                                        span { class: "text-sm font-black text-green-700 text-center", "{order.paid:.2} kr" }
                                        // Status-badges
                                        div { class: "flex flex-col gap-1 items-center",
                                            span { class: "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-bold border {os.badge_class()}",
                                                i { class: "{os.icon()} text-[10px]" }
                                                "{os.label()}"
                                            }
                                            span { class: "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-semibold border {ps.badge_class()}",
                                                i { class: "{ps.icon()} text-[10px]" }
                                                "{ps.label()}"
                                            }
                                        }
                                        // Redigera-knapp
                                        button {
                                            class: "text-xs border border-gray-200 rounded-lg px-3 py-1.5 hover:bg-green-50 hover:border-green-300 hover:text-green-700 text-gray-600 font-bold transition",
                                            onclick: move |_| editing_order.set(Some(idx)),
                                            i { class: "fa-solid fa-pen text-[10px] mr-1" }
                                            "Status"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Modal
            if let Some(idx) = editing_order() {
                if let Some(order) = orders.read().get(idx).cloned() {
                    UpdateOrderStatusModal {
                        order,
                        on_close: move |updated: VendorOrderRow| {
                            orders.write()[idx] = updated;
                            editing_order.set(None);
                        },
                    }
                }
            }
        }
    }
}
 
// Vendor page
 
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
    let mut products_resource = use_resource(move || async move {
        vendor_products(customer_id, id, 100, 0, is_own_profile).await
    });
 
    let mut show_add_modal  = use_signal(|| false);
    let mut edit_product: Signal<Option<ProductOverviewVendor>> = use_signal(|| None);
    let mut active_tab      = use_signal(|| 0_u8); // 0 = produkter, 1 = ordrar
 
    rsx! {
        div { class: "min-h-screen bg-gray-50",
            div { class: "max-w-6xl mx-auto p-6",
                Link {
                    to: Route::Home {},
                    class: "text-green-700 hover:text-green-900 font-bold flex items-center gap-2 mb-4 transition-colors",
                    i { class: "fa-solid fa-arrow-left" }
                    "Tillbaka till start"
                }

                // Vendor header
                {
                    let info_read = info_resource.read();
                    if info_read.is_none() {
                        rsx! {
                            p { class: "text-gray-400 animate-pulse mb-8", "Laddar..." }
                        }
                    } else if let Some(Err(e)) = info_read.as_ref() {
                        rsx! {
                            p { class: "text-red-400 mb-8", "Fel: {e}" }
                        }
                    } else if let Some(Ok(info)) = info_read.as_ref() {
                        rsx! {
                            div { class: "flex items-center gap-6 mb-6",
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
                        }
                    } else {
                        rsx! {}
                    }
                }

                // Flikar (visas bara för ägaren)
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

                // Produkter-flik
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

                    {
                        let prod_read = products_resource.read();
                        if prod_read.is_none() {
                            rsx! {
                                p { class: "text-gray-400 animate-pulse", "Laddar..." }
                            }
                        } else if let Some(Err(e)) = prod_read.as_ref() {
                            rsx! {
                                p { class: "text-red-400 text-sm", "Fel: {e}" }
                            }
                        } else if let Some(Ok(products)) = prod_read.as_ref() {
                            if products.is_empty() {
                                rsx! {
                                    p { class: "text-gray-400 text-sm", "Inga produkter ännu." }
                                }
                            } else {
                                rsx! {
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
                                                                }
                                                                p { class: "text-gray-400 text-xs mt-1", "{p.in_stock} i lager" }
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
                        } else {
                            rsx! {}
                        }
                    }
                }

                // Ordrar-flik (bara för ägaren)
                if active_tab() == 1 && is_own_profile {
                    VendorOrdersTab {}
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