use crate::Route;
use crate::components::product_card::{offer_label, ProductCard};
use crate::database::categories::category_trees;
use crate::database::products::products_by_category;
use crate::database::{Category as CategoryMarker, Id};
use dioxus::prelude::*;
use rust_decimal::prelude::ToPrimitive;
 
/// Bygger en platt lista av alla kategorier med sina föräldrars namn för breadcrumb
/// Returnerar 'Vec<(id, Vec<(id, name)>)>' 
fn build_paths(
    trees: &[crate::database::categories::CategoryTree],
) -> Vec<(Id<CategoryMarker>, Vec<(Id<CategoryMarker>, Box<str>)>)> {
    fn recurse(
        tree: &crate::database::categories::CategoryTree,
        prefix: &[(Id<CategoryMarker>, Box<str>)],
        out: &mut Vec<(Id<CategoryMarker>, Vec<(Id<CategoryMarker>, Box<str>)>)>,
    ) {
        let mut path = prefix.to_vec();
        path.push((tree.id, tree.name.clone()));
        out.push((tree.id, path.clone()));
        for sub in tree.subcategories.iter() {
            recurse(sub, &path, out);
        }
    }
    let mut out = Vec::new();
    for tree in trees {
        recurse(tree, &[], &mut out);
    }
    out
}
 
/// Breadcrumb för kategorisidan.
#[component]
fn CategoryBreadcrumb(
    current_id: Id<CategoryMarker>,
    /// Alla kategorisökvägar
    paths: Vec<(Id<CategoryMarker>, Vec<(Id<CategoryMarker>, Box<str>)>)>,
) -> Element {
    // Visa ingen breadcrumb på "Visa alla" sidan
    if current_id == Id::<CategoryMarker>::from(0) {
        return rsx! {
            div {}
        };
    }
 
    let path = paths.iter().find(|(id, _)| *id == current_id).map(|(_, p)| p.clone());
 
    rsx! {
        nav { class: "flex items-center flex-wrap gap-1 text-sm text-gray-500 mb-6",
            Link {
                to: Route::Home {},
                class: "hover:text-green-700 transition-colors font-medium",
                i { class: "fa-solid fa-house text-xs mr-1" }
                "Start"
            }
            i { class: "fa-solid fa-chevron-right text-[10px] text-gray-300" }
            Link {
                to: Route::Category {
                    id: Id::<CategoryMarker>::from(0),
                },
                class: "hover:text-green-700 transition-colors font-medium",
                "Kategorier"
            }
            if let Some(segments) = path {
                for (seg_id , seg_name) in segments.iter() {
                    i { class: "fa-solid fa-chevron-right text-[10px] text-gray-300" }
                    if *seg_id == current_id {
                        // Sista segmentet är inte klickbar
                        span { class: "text-gray-900 font-semibold", "{seg_name}" }
                    } else {
                        Link {
                            to: Route::Category { id: *seg_id },
                            class: "hover:text-green-700 transition-colors font-medium",
                            "{seg_name}"
                        }
                    }
                }
            }
        }
    }
}
 
// A page for categorys
 
/// * `id` - Category to display.
#[allow(
    clippy::same_name_method,
    clippy::option_if_let_else,
    reason = "Dioxus macro limitation"
)]
#[component]
pub fn CategoryPage(id: Id<CategoryMarker>) -> Element {
    let categories_resource =
        use_resource(move || async move { category_trees().await.unwrap_or_default() });
 
    rsx! {
        div { class: "container mx-auto p-6 flex flex-col md:flex-row gap-8 min-h-screen",
            // Sidebar
            nav { class: "w-full md:w-64 flex flex-col gap-3 shrink-0",
                Link {
                    to: Route::Home {},
                    class: "flex items-center gap-2 text-gray-500 hover:text-green-700 text-sm mb-4 transition",
                    i { class: "fa-solid fa-arrow-left text-xs" }
                    "Tillbaka till startsidan"
                }
                Link {
                    to: Route::Category {
                        id: Id::<CategoryMarker>::from(0),
                    },
                    class: if id == Id::<CategoryMarker>::from(0) { "text-green-700 font-bold px-3 py-2 rounded-lg bg-green-50" } else { "text-gray-600 hover:text-green-700 px-3 py-2 rounded-lg hover:bg-gray-50" },
                    "Visa alla kategorier"
                }
                match &*categories_resource.read() {
                    None => rsx! {
                        p { class: "text-gray-400 text-sm px-3", "Laddar..." }
                    },
                    Some(trees) => rsx! {
                        for tree in trees.iter() {
                            Link {
                                to: Route::Category { id: tree.id },
                                class: if id == tree.id { "text-green-700 font-bold px-3 py-2 rounded-lg bg-green-50" } else { "text-gray-600 hover:text-green-700 px-3 py-2 rounded-lg hover:bg-gray-50" },
                                "{tree.name}"
                            }
                            for sub in tree.subcategories.iter() {
                                Link {
                                    to: Route::Category { id: sub.id },
                                    class: if id == sub.id { "text-green-700 font-bold pl-6 py-1.5 rounded-lg bg-green-50 text-sm" } else { "text-gray-400 hover:text-green-700 pl-6 py-1.5 rounded-lg hover:bg-gray-50 text-sm" },
                                    "— {sub.name}"
                                }
                            }
                        }
                    },
                }
            }

            main { class: "flex-grow overflow-hidden",
                // Breadcrumb
                match &*categories_resource.read() {
                    None => rsx! {
                        div { class: "h-8 mb-6" }
                    },
                    Some(trees) => {
                        let paths = build_paths(trees);
                        rsx! {
                            CategoryBreadcrumb { current_id: id, paths }
                        }
                    }
                }

                // Rubrik
                if id == Id::<CategoryMarker>::from(0) {
                    h1 { class: "text-4xl font-black mb-8 text-gray-900", "Kategorier" }
                }

                // Sektioner
                match &*categories_resource.read() {
                    None => rsx! {
                        p { class: "text-gray-400 py-20 text-center", "Laddar kategorier..." }
                    },
                    Some(trees) => rsx! {
                        for tree in trees.iter() {
                            if id == Id::<CategoryMarker>::from(0) || id == tree.id {
                                CategorySection {
                                    cat_id: tree.id,
                                    cat_name: tree.name.clone().into(),
                                    all_cat_ids: vec![tree.id],
                                    show_all: id != Id::<CategoryMarker>::from(0),
                                    scroll_index: 0,
                                }
                            }
                            // Visa underkategorier om man är på en underkategori
                            for sub in tree.subcategories.iter() {
                                if id == sub.id {
                                    CategorySection {
                                        cat_id: sub.id,
                                        cat_name: sub.name.clone().into(),
                                        all_cat_ids: vec![sub.id],
                                        show_all: true,
                                        scroll_index: 0,
                                    }
                                }
                            }
                        }
                    },
                }
            }
        }
    }
}
 
/// Props for a single category section
#[allow(clippy::same_name_method, reason = "Dioxus macro limitation")]
#[derive(Props, Clone, PartialEq)]
struct CategorySectionProps {
    cat_id: Id<CategoryMarker>,
    cat_name: Box<str>,
    all_cat_ids: Vec<Id<CategoryMarker>>,
    show_all: bool,
    scroll_index: usize,
}
 
/// A section showing products for one category
#[component]
fn CategorySection(props: CategorySectionProps) -> Element {
    let cat_id   = props.cat_id;
    let show_all = props.show_all;
    let all_cat_ids = props.all_cat_ids.clone();
    let mut pos = use_signal(|| 0_usize);
 
    let products = use_resource(move || {
        let cat_ids = all_cat_ids.clone();
        async move {
            let limit_per_cat = if show_all { 50 } else { 12 };
            let mut all_products = Vec::new();
            for cid in cat_ids {
                if let Ok(mut prods) = products_by_category(None, cid, None, limit_per_cat, 0).await {
                    all_products.extend(prods.iter().cloned());
                }
            }
            all_products.truncate(if show_all { 200 } else { 40 });
            all_products
        }
    });
 
    let current_pos = *pos.read();
    let products_read = products.read();
    let loading    = products_read.is_none();
    let items_opt  = products_read.as_deref();
    let is_empty   = items_opt.map(|v| v.is_empty()).unwrap_or(false);
    let items_data = items_opt.filter(|v| !v.is_empty());
 
    let total      = items_data.map(|v| v.len()).unwrap_or(0);
    let max_steps  = if total > 4 { (total as f64 / 4.0).ceil() as usize } else { 1 };
    let offset     = current_pos * 100;
 
    rsx! {
        div { class: "mb-20",
            div { class: "flex justify-between items-center mb-6",
                h2 { class: "text-2xl font-bold flex items-center gap-2",
                    span { class: "w-2 h-8 bg-green-700 rounded-full block" }
                    "{props.cat_name}"
                }
                if !show_all {
                    Link {
                        to: Route::Category { id: cat_id },
                        class: "flex items-center gap-2 text-green-700 font-bold hover:text-green-800 transition-colors bg-green-50 px-4 py-2 rounded-full text-sm",
                        "Visa alla {props.cat_name}"
                        i { class: "fa-solid fa-chevron-right text-xs" }
                    }
                }
            }

            if loading {
                p { class: "text-gray-400", "Laddar produkter..." }
            } else if is_empty {
                div { class: "h-48 flex flex-col items-center justify-center bg-gray-50 rounded-xl border-2 border-dashed border-gray-200",
                    i { class: "fa-solid fa-box-open text-4xl text-gray-400 mb-3" }
                    p { class: "font-bold text-gray-500", "Denna kategori är tom" }
                }
            } else if let Some(items) = items_data {
                if show_all {
                    div { class: "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6",
                        for p in items.iter() {
                            ProductCard {
                                id: p.id.get(),
                                name: p.name.clone(),
                                price: p.price.to_f64().unwrap_or_default(),
                                comparison_price: format!("{:.2} kr", p.price),
                                image_url: p.thumbnail.to_string(),
                                in_stock: p.in_stock.into(),
                                special_offer: offer_label(p.special_offer_deal, p.price),
                            }
                        }
                    }
                } else {
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
                                            name: p.name.clone(),
                                            price: p.price.to_f64().unwrap_or_default(),
                                            comparison_price: format!("{:.2} kr", p.price),
                                            image_url: p.thumbnail.to_string(),
                                            in_stock: p.in_stock.into(),
                                            special_offer: offer_label(p.special_offer_deal, p.price),
                                        }
                                    }
                                }
                                div { class: "min-w-full md:min-w-[25%] p-2",
                                    Link { to: Route::Category { id: cat_id },
                                        div { class: "h-full flex flex-col items-center justify-center bg-gray-50 rounded-xl border-2 border-dashed border-gray-200 hover:bg-green-100 transition-all min-h-[380px]",
                                            div { class: "text-center p-4",
                                                i { class: "fa-solid fa-arrow-right-long text-4xl text-green-700 mb-3" }
                                                p { class: "font-bold text-gray-800",
                                                    "Visa hela sortimentet"
                                                }
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
                                for step in 0..max_steps {
                                    button {
                                        class: if current_pos == step { "w-10 h-2 rounded-full bg-green-700 transition-all" } else { "w-2 h-2 rounded-full bg-gray-300 hover:bg-gray-400" },
                                        onclick: move |_| pos.set(step),
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