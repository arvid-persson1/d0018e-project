#![allow(non_snake_case)]
use crate::Route;
use crate::database::search::{SearchResult, search_products};
use dioxus::prelude::*;

/// Sökresultatsida som visas när man trycker Enter i sökning
#[component]
pub fn Search(query: String) -> Element {
    let nav = use_navigator();
    let mut search_input = use_signal(|| query.clone());

    let q = query.clone();
    let results = use_resource(move || {
        let q = q.clone();
        async move {
            if q.trim().is_empty() {
                Ok(Vec::new())
            } else {
                search_products(q.into(), 50).await
            }
        }
    });

    rsx! {
        div { class: "min-h-screen bg-gray-50",
            main { class: "container mx-auto p-4 py-8",

                // Sökfält
                div { class: "mb-8 max-w-2xl",
                    div { class: "relative",
                        i { class: "fa-solid fa-magnifying-glass absolute left-4 top-1/2 -translate-y-1/2 text-gray-400" }
                        input {
                            r#type: "text",
                            value: "{search_input}",
                            placeholder: "Sök på boop...",
                            class: "w-full py-3 pl-12 pr-4 rounded-full text-black bg-white border focus:outline-none focus:ring-4 focus:ring-green-500/30 transition-all shadow-sm",
                            oninput: move |e| search_input.set(e.value()),
                            onkeydown: move |e| {
                                if e.key() == Key::Enter {
                                    let q = search_input.read().clone();
                                    if !q.trim().is_empty() {
                                        let _unused = nav.push(Route::Search { query: q });
                                    }
                                }
                            },
                        }
                    }
                }

                h1 { class: "text-2xl font-black text-gray-800 mb-2", "Sökresultat för \"{query}\"" }

                match &*results.read() {
                    None => rsx! {
                        p { class: "text-gray-400 animate-pulse mt-4", "Söker..." }
                    },
                    Some(Err(e)) => rsx! {
                        p { class: "text-red-400 text-sm mt-4", "Fel: {e}" }
                    },
                    Some(Ok(products)) if products.is_empty() => rsx! {
                        div { class: "text-center py-20",
                            i { class: "fa-solid fa-magnifying-glass text-5xl text-gray-200 mb-4" }
                            p { class: "text-gray-500 text-lg font-bold", "Inga produkter hittades" }
                            p { class: "text-gray-400 text-sm mt-1", "Prova ett annat sökord" }
                        }
                    },
                    Some(Ok(products)) => rsx! {
                        p { class: "text-gray-400 text-sm mb-6", "{products.len()} produkter hittades" }
                        div { class: "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6",
                            for p in products.iter() {
                                SearchResultCard { result: p.clone() }
                            }
                        }
                    },
                }
            }
        }
    }
}

/// sökresultat
#[component]
fn SearchResultCard(result: SearchResult) -> Element {
    rsx! {
        Link {
            to: Route::Product {
                id: result.id.into(),
            },
            class: "bg-white rounded-2xl shadow-sm overflow-hidden hover:shadow-md transition block",
            div { class: "h-40 bg-gray-100 overflow-hidden",
                img {
                    src: "{result.thumbnail}",
                    class: "w-full h-full object-cover",
                    alt: "{result.name}",
                }
            }
            div { class: "p-4",
                p { class: "font-bold text-gray-900", "{result.name}" }
            }
        }
    }
}