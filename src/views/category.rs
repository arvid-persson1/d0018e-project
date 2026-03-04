use crate::Route;
use crate::components::ProductCard;
use crate::database::categories::category_trees;
use crate::database::products::products_by_category;
use crate::database::{Category as CategoryMarker, Id};
use dioxus::prelude::*;

// A page for categorys

#[component]
pub fn CategoryPage(id: Id<CategoryMarker>) -> Element {
    // Hämta kategorier från databasen
    let categories_resource =
        use_resource(|| async move { category_trees().await.unwrap_or_default() });

    rsx! {
        div { class: "container mx-auto p-6 flex flex-col md:flex-row gap-8 min-h-screen",

            // side bar för kategorier
            aside { class: "w-full md:w-64 flex-shrink-0",
                div { class: "sticky top-24 bg-white p-4 rounded-lg shadow-sm border border-gray-100",
                    h2 { class: "text-xl font-bold border-b pb-4 mb-4", "Kategorier" }
                    nav { class: "flex flex-col gap-3",
                        // Aktiv länk = grön, inaktiv = grå
                        Link {
                            to: Route::Category { id: 0.into() },
                            class: if id == 0.into() { "text-green-700 font-bold" } else { "text-gray-600 hover:text-green-700" },
                            "Visa alla kategorier"
                        }

                        match &*categories_resource.read() {
                            None => rsx! {
                                p { class: "text-gray-400 text-sm", "Laddar..." }
                            },
                            Some(trees) => rsx! {
                                for tree in trees.iter() {
                                    Link {
                                        to: Route::Category { id: tree.id },
                                        class: if id == tree.id { "text-green-700 font-bold" } else { "text-gray-600 hover:text-green-700" },
                                        "{tree.name}"
                                    }
                                    for sub in tree.subcategories.iter() {
                                        Link {
                                            to: Route::Category { id: sub.id },
                                            class: if id == sub.id { "text-green-700 font-bold pl-4" } else { "text-gray-400 hover:text-green-700 pl-4 text-sm" },
                                            "— {sub.name}"
                                        }
                                    }
                                }
                            },
                        }
                    }
                }
            }

            main { class: "flex-grow overflow-hidden",
                // Rubrik
                if id == 0.into() {
                    h1 { class: "text-4xl font-black mb-8 text-gray-900", "Kategorier" }
                }

                // Slider
                match &*categories_resource.read() {
                    None => rsx! {
                        p { class: "text-gray-400 py-20 text-center", "Laddar kategorier..." }
                    },
                    Some(trees) => rsx! {
                        for (i , tree) in trees.iter().enumerate() {
                            if id == 0.into() || id == tree.id {
                                CategorySection {
                                    cat_id: tree.id,
                                    cat_name: tree.name.clone(),
                                    show_all: id != 0.into(),
                                    scroll_index: i,
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
#[derive(Props, Clone, PartialEq)]
struct CategorySectionProps {
    cat_id: Id<CategoryMarker>,
    cat_name: Box<str>,
    show_all: bool,
    scroll_index: usize,
}

/// A section showing products for one category
#[component]
fn CategorySection(props: CategorySectionProps) -> Element {
    let cat_id = props.cat_id;
    let show_all = props.show_all;
    let mut pos = use_signal(|| 0_usize);

    let products = use_resource(move || async move {
        products_by_category(None, cat_id, None, if show_all { 50 } else { 12 }, 0)
            .await
            .unwrap_or_default()
    });

    let current_pos = *pos.read();

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

            match &*products.read() {
                None => rsx! {
                    p { class: "text-gray-400", "Laddar produkter..." }
                },
                Some(items) if items.is_empty() => rsx! {
                    div { class: "h-48 flex flex-col items-center justify-center bg-gray-50 rounded-xl border-2 border-dashed border-gray-200",
                        i { class: "fa-solid fa-box-open text-4xl text-gray-400 mb-3" }
                        p { class: "font-bold text-gray-500", "Denna kategori är tom" }
                    }
                },
                Some(items) => {
                    let total = items.len();
                    let max_steps = if total > 4 { total.div_ceil(4) } else { 1 };
                    let offset = current_pos * 100;

                    if show_all {
                        rsx! {
                            div { class: "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6",
                                for p in items.iter() {
                                    ProductCard {
                                        id: p.id,
                                        name: p.name.clone(),
                                        price: p.price,
                                        comparison_price: p.amount_per_unit.to_string().into(),
                                        image_url: p.thumbnail.clone(),
                                    }
                                }
                            }
                        }
                    } else {
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
                                                    id: p.id,
                                                    name: p.name.clone(),
                                                    price: p.price,
                                                    comparison_price: p.amount_per_unit.to_string().into(),
                                                    image_url: p.thumbnail.clone(),
                                                }
                                            }
                                        }
                                        div { class: "min-w-full md:min-w-[25%] p-2",
                                            Link { to: Route::Category { id: cat_id },
                                                div { class: "h-full flex flex-col items-center justify-center bg-gray-50 rounded-xl border-2 border-dashed border-gray-200 hover:bg-green-100 transition-all min-h-[380px]",
                                                    i { class: "fa-solid fa-arrow-right-long text-4xl text-green-700 mb-3" }
                                                    p { class: "font-bold text-gray-800", "Visa hela sortimentet" }
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
                }
            }
        }
    }
}
