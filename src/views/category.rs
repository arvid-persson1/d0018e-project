use dioxus::prelude::*;
use crate::components::product_card::ProductCard;
use crate::fake_data::get_fake_products;
use crate::Route;

// A page for categorys

#[component]
pub fn Category(id: i32) -> Element {
    // TODO(db): Ersätt get_fake_products() med ett API-anrop eller databas-query
    let products = get_fake_products();
    
    let scroll_pos_1 = use_signal(|| 0);
    let scroll_pos_2 = use_signal(|| 0);
    let scroll_pos_3 = use_signal(|| 0);
    let scroll_pos_4 = use_signal(|| 0);

    // TODO(db): Ersätt hårdkodad lista med hämtade kategorier från API
    let categories = vec![
        (1, "Mejeri & Ägg", scroll_pos_1),
        (2, "Frukt & Grönt", scroll_pos_2),
        (3, "Kött & Chark", scroll_pos_3),
        (4, "Skafferi", scroll_pos_4),
    ];

    rsx! {

        div { class: "container mx-auto p-6 flex flex-col md:flex-row gap-8 min-h-screen",

            // side bar för alla kategorier
            aside { class: "w-full md:w-64 flex-shrink-0",
                div { class: "sticky top-24 bg-white p-4 rounded-lg shadow-sm border border-gray-100",
                    h2 { class: "text-xl font-bold border-b pb-4 mb-4", "Kategorier" }
                    nav { class: "flex flex-col gap-3",
                        // Aktiv länk = grön, inaktiv = grå
                        Link {
                            to: Route::Category { id: 0 },
                            class: if id == 0 { "text-green-700 font-bold" } else { "text-gray-600 hover:text-green-700" },
                            "Visa alla kategorier"
                        }
                        // Loopar alla kategorier och skapar en länk per kategori
                        for (cat_id , name , _pos) in categories.clone() {
                            Link {
                                to: Route::Category { id: cat_id },
                                class: if id == cat_id { "text-green-700 font-bold" } else { "text-gray-600 hover:text-green-700" },
                                "{name}"
                            }
                        }
                    }
                }
            }

            main { class: "flex-grow overflow-hidden",
                // Rubrik visas bara på "Visa alla"-sidan
                if id == 0 {
                    h1 { class: "text-4xl font-black mb-8 text-gray-900", "Våra Kategorier" }
                }

                // räkna ut steg för scrollning av kategorier
                for (cat_id , cat_name , mut pos) in categories {
                    if id == 0 || id == cat_id {
                        {
                            let current_pos = *pos.read();
                            let offset = current_pos * 100;
                            // TODO(db): Ersätt med en query som hämtar produkter filtrerade per
                            let cat_products: Vec<_> = products

                                .iter()
                                .filter(|p| p.category_id == cat_id)
                                .collect();
                            // Antal produkter
                            let total_cards = cat_products.iter().take(12).count();
                            // Antal sidor i slidern, varje sida visar 4 produkter
                            let max_steps = if total_cards > 4 { (total_cards + 3) / 4 } else { 1 };
                            rsx! {
                                div { class: "mb-20",
                                    div { class: "flex justify-between items-center mb-6",
                                        // Kategorinamn
                                        h2 { class: "text-2xl font-bold flex items-center gap-2",
                                            span { class: "w-2 h-8 bg-green-700 rounded-full block" }
                                            "{cat_name}"
                                        }
                                        // "Visa alla" knapp till
                                        Link {
                                            to: Route::Category { id: cat_id },
                                            class: "flex items-center gap-2 text-green-700 font-bold hover:text-green-800 transition-colors bg-green-50 px-4 py-2 rounded-full text-sm",
                                            "Visa alla {cat_name}"
                                            i { class: "fa-solid fa-chevron-right text-xs" }
                                        }
                                    }

                                    if id == 0 {
                                        div { class: "relative group",

                                            // Vänster pil
                                            if current_pos > 0 && total_cards > 4 {
                                                button {
                                                    class: "absolute -left-5 top-1/2 -translate-y-1/2 w-12 h-12 bg-white shadow-2xl border rounded-full flex items-center justify-center hover:bg-green-700 hover:text-white transition-all z-30",
                                                    onclick: move |_| pos.set(current_pos - 1),
                                                    i { class: "fa-solid fa-chevron-left text-lg" }
                                                }
                                            }

                                            // Produktslider
                                            div { class: "overflow-hidden",
                                                div {
                                                    class: "flex transition-transform duration-700 ease-in-out",
                                                    style: "transform: translateX(-{offset}%);",

                                                    // loopa maxsteg
                                                    for p in cat_products.iter().take(11) {
                                                        div { class: "min-w-full md:min-w-[25%] p-2",
                                                            // TODO(db): ProductCard är samma, bara datan ändras
                                                            ProductCard {
                                                                id: p.id,
                                                                name: p.name.clone(),
                                                                price: p.price,
                                                                comparison_price: p.comparison_price.clone(),
                                                                image_url: p.image_url.clone(),
                                                            }
                                                        }
                                                    }

                                                    // etikett i slutet av scrollningen
                                                    if total_cards == 0 {
                                                        div { class: "min-w-full md:min-w-[25%] p-2",
                                                            div { class: "h-full flex flex-col items-center justify-center bg-gray-50 rounded-xl border-2 border-dashed border-gray-200 min-h-[380px]",
                                                                div { class: "text-center p-4",
                                                                    i { class: "fa-solid fa-box-open text-4xl text-gray-400 mb-3" }
                                                                    p { class: "font-bold text-gray-500", "Denna kategori är tom" }
                                                                }
                                                            }
                                                        }
                                                    } else {
                                                        // "Visa hela sortimentet" kort i slutet
                                                        div { class: "min-w-full md:min-w-[25%] p-2",
                                                            Link { to: Route::Category { id: cat_id },
                                                                div { class: "h-full flex flex-col items-center justify-center bg-gray-50 rounded-xl border-2 border-dashed border-gray-200 hover:bg-green-100 transition-all min-h-[380px]",
                                                                    div { class: "text-center p-4",
                                                                        i { class: "fa-solid fa-arrow-right-long text-4xl text-green-700 mb-3" }
                                                                        p { class: "font-bold text-gray-800", "Visa hela sortimentet" }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            // Höger pil
                                            if current_pos < max_steps - 1 && total_cards > 4 {
                                                button {
                                                    class: "absolute -right-5 top-1/2 -translate-y-1/2 w-12 h-12 bg-white shadow-2xl border rounded-full flex items-center justify-center hover:bg-green-700 hover:text-white transition-all z-30",
                                                    onclick: move |_| pos.set(current_pos + 1),
                                                    i { class: "fa-solid fa-chevron-right text-lg" }
                                                }
                                            }

                                            // Dot-indikator
                                            if total_cards > 4 {
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
                                    } else {
                                        // grid läge för när man är i enskild kategori
                                        div { class: "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6",
                                            for p in cat_products {
                                                ProductCard {
                                                    id: p.id,
                                                    name: p.name.clone(),
                                                    price: p.price,
                                                    comparison_price: p.comparison_price.clone(),
                                                    image_url: p.image_url.clone(),
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
}