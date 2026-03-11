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

/// Modal för att lägga till en ny produkt.
#[component]
fn AddProductModal(vendor_id: Id<VendorEntity>, on_close: EventHandler<bool>) -> Element {
    let mut name = use_signal(String::new);
    let mut thumbnail = use_signal(String::new);
    let mut price_str = use_signal(String::new);
    let mut overview = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut origin = use_signal(String::new);
    let mut category_id = use_signal(|| 0_i32);
    let mut amount_qty = use_signal(|| "1".to_string());
    let mut amount_unit = use_signal(|| "kg".to_string());
    let mut stock_str = use_signal(|| "0".to_string());
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);

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
                    div { class: "flex gap-3",
                        div { class: "flex-1",
                            label { class: "block text-sm font-bold text-gray-700 mb-1",
                                "Pris (kr) *"
                            }
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
                        div { class: "flex-1",
                            label { class: "block text-sm font-bold text-gray-700 mb-1",
                                "Lager (antal) *"
                            }
                            input {
                                r#type: "number",
                                min: "0",
                                class: "w-full border border-gray-200 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-green-500",
                                placeholder: "100",
                                value: "{stock_str}",
                                oninput: move |e| stock_str.set(e.value()),
                            }
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
                                let Ok(stock_val) = stock_str().trim().parse::<u32>() else {
                                    error.set(Some("Ogiltigt lagerantal".to_string()));
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
                                            if stock_val > 0 {}
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

/// Modal för att redigera en befintlig produkt.
#[component]
fn EditProductModal(product: ProductOverviewVendor, on_close: EventHandler<bool>) -> Element {
    let product_id = product.id;
    let current_stock = product.in_stock;
    let mut name = use_signal(|| product.name.to_string());
    let mut thumbnail = use_signal(|| product.thumbnail.to_string());
    let mut price_str = use_signal(|| product.price.to_string());
    let mut overview_text = use_signal(|| product.overview.to_string());
    let mut origin_text = use_signal(|| product.origin.to_string());
    let mut stock_add = use_signal(|| "0".to_string());
    let mut visible = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);
    let mut saved = use_signal(|| false);

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
                                span { class: "text-gray-400 font-normal ml-1",
                                    "(nu: {current_stock}, läggs till)"
                                }
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
                                    if let Some(stock_nonzero) = std::num::NonZeroU32::new(stock_to_add) {
                                        if let Err(e) = add_stock(product_id, stock_nonzero, None)
                                            .await
                                            .map(|_| ())
                                        {
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
    let mut products_resource = use_resource(move || async move {
        vendor_products(customer_id, id, 100, 0, is_own_profile).await
    });

    let mut show_add_modal = use_signal(|| false);
    let mut edit_product: Signal<Option<ProductOverviewVendor>> = use_signal(|| None);

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
                    },
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