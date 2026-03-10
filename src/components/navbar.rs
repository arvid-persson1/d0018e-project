use crate::Route;
use crate::components::auth_dropdown::AuthDropdown;
use crate::components::cart_dropdown::CartDropdown;
use crate::database::categories::category_trees;
use crate::database::search::search_products;
use crate::state::GlobalState;
use dioxus::prelude::*;

// Class for the category navigation bar
#[allow(non_snake_case)]
#[component]
fn SidebarCategory(title: String, id: i32, subcategories: Vec<(String, i32)>) -> Element {
    let mut is_open = use_signal(|| false);
    let rotation = if is_open() { "rotate-180" } else { "" };

    rsx! {
        div { class: "flex flex-col w-full border-b border-gray-100",
            div {
                class: "flex justify-between items-center py-4 px-2 cursor-pointer hover:bg-green-50 transition-colors",
                onclick: move |_| is_open.toggle(),
                Link {
                    to: Route::Category { id: id.into() },
                    class: "font-bold text-gray-800 hover:text-green-700 flex-grow",
                    "{title}"
                }
                i { class: "fa-solid fa-chevron-down transition-transform duration-300 {rotation}" }
            }
            if is_open() {
                div { class: "bg-gray-50 flex flex-col pb-2",
                    for (name , sub_id) in subcategories.into_iter() {
                        Link {
                            to: Route::Category {
                                id: sub_id.into(),
                            },
                            class: "pl-6 py-2 text-gray-600 hover:text-green-700 hover:bg-gray-100 text-sm transition-colors",
                            "{name}"
                        }
                    }
                }
            }
        }
    }
}

/// Navbar.
#[allow(non_snake_case)]
#[component]
pub fn Navbar() -> Element {
    let mut show_sidebar = use_signal(|| false);
    let mut show_auth = use_signal(|| false);
    let mut show_cart = use_signal(|| false);
    let global_state = use_context::<Signal<GlobalState>>();

    let fav_count = global_state.read().favorites.len();
    let cart_count = global_state.read().cart_count();

    let mut search_query = use_signal(String::new);
    let mut show_search_results = use_signal(|| false);

    // Hämta sökresultat
    let search_results = use_resource(move || {
        let q = search_query();
        async move {
            if q.trim().is_empty() {
                Ok(Vec::new())
            } else {
                search_products(q.into(), 8).await
            }
        }
    });

    // Hämta kategorier från databasen för sidebaren
    let categories = use_resource(|| async move { category_trees().await.unwrap_or_default() });

    rsx! {
        div {
            header { class: "w-full sticky top-0 z-50",
                nav { class: "bg-gray-700 text-white p-5 flex items-center shadow-lg",
                    div { class: "container mx-auto flex items-center justify-between gap-4",

                        Link {
                            to: Route::Home {},
                            class: "bg-green-900 px-4 py-1 rounded-lg border border-green-600 shadow-inner flex items-center justify-center hover:bg-green-800 transition-colors cursor-pointer",
                            span { class: "text-3xl font-black italic tracking-tighter text-white",
                                "boop"
                            }
                        }

                        div { class: "flex justify-center items-center py-2 shadow-sm",
                            button {
                                class: "flex items-center justify-center w-9 h-9 rounded-full bg-gray-100 text-gray-700 hover:bg-green-100 hover:text-green-700 transition-all duration-200 shadow-sm border border-gray-200",
                                onclick: move |_| show_sidebar.set(true),
                                i { class: "fa-solid fa-bars text-xl" }
                            }
                        }

                        // Sökfält
                        div { class: "flex-grow max-w-2xl relative hidden md:block",
                            i { class: "fa-solid fa-magnifying-glass absolute left-4 top-1/2 -translate-y-1/2 text-gray-400 z-10" }
                            input {
                                r#type: "text",
                                placeholder: "Sök på boop...",
                                value: "{search_query}",
                                class: "w-full py-2.5 pl-12 pr-4 rounded-full text-black bg-white focus:outline-none focus:ring-4 focus:ring-green-500/30 transition-all",
                                oninput: move |e| {
                                    search_query.set(e.value());
                                    show_search_results.set(true);
                                },
                                onfocus: move |_| show_search_results.set(true),
                            }

                            // Sökning dropdown
                            if show_search_results() && !search_query().trim().is_empty() {

                                div {
                                    class: "fixed inset-0 z-10",
                                    onclick: move |_| show_search_results.set(false),
                                }
                                div { class: "absolute top-full mt-2 left-0 right-0 bg-white rounded-2xl shadow-xl border z-20 overflow-hidden",
                                    match &*search_results.read() {
                                        None => rsx! {
                                            div { class: "p-4 text-gray-400 text-sm animate-pulse", "Söker..." }
                                        },
                                        Some(Err(_)) => rsx! {
                                            div { class: "p-4 text-red-400 text-sm", "Något gick fel." }
                                        },
                                        Some(Ok(results)) if results.is_empty() => rsx! {
                                            div { class: "p-4 text-gray-400 text-sm", "Inga produkter hittades för \"{search_query}\"" }
                                        },
                                        Some(Ok(results)) => rsx! {
                                            div { class: "py-2",
                                                for result in results.iter() {
                                                    Link {
                                                        to: Route::Product {
                                                            id: result.id.into(),
                                                        },
                                                        class: "flex items-center gap-3 px-4 py-2 hover:bg-gray-50 transition",
                                                        onclick: move |_| {
                                                            show_search_results.set(false);
                                                            search_query.set(String::new());
                                                        },
                                                        img {
                                                            src: "{result.thumbnail}",
                                                            class: "w-10 h-10 rounded-lg object-cover bg-gray-100",
                                                            alt: "{result.name}",
                                                        }
                                                        span { class: "text-gray-800 text-sm font-medium", "{result.name}" }
                                                    }
                                                }
                                            }
                                        },
                                    }
                                }
                            }
                        }

                        div { class: "flex items-center gap-6",
                            // Favoriter
                            Link {
                                to: Route::Favorites {},
                                class: "relative flex flex-col items-center hover:text-green-200 cursor-pointer transition",
                                i { class: "fa-solid fa-heart text-2xl" }
                                span { class: "text-[10px] font-bold uppercase", "Favoriter" }
                                if fav_count > 0 {
                                    span { class: "absolute -top-1 -right-1 bg-red-500 text-white text-[10px] rounded-full w-5 h-5 flex items-center justify-center border-2 border-green-700",
                                        "{fav_count}"
                                    }
                                }
                            }

                            // Konto
                            // TODO(db): Koppla "Konto"-knappen till inloggningssida/användarprofil
                            div { class: "relative",
                                button {
                                    class: "flex flex-col items-center hover:text-green-200 cursor-pointer transition",
                                    onclick: move |_| {
                                        show_auth.toggle();
                                        show_cart.set(false);
                                    },
                                    i { class: "fa-solid fa-circle-user text-2xl" }
                                    span { class: "text-[10px] font-bold uppercase",
                                        "Konto"
                                    }
                                }
                                if show_auth() {
                                    AuthDropdown { on_close: move |_| show_auth.set(false) }
                                }
                            }

                            // Kundvagn
                            div { class: "relative",
                                button {
                                    class: "bg-white text-green-700 px-5 py-2 rounded-full font-black flex items-center gap-2 hover:bg-green-50 transition shadow-sm",
                                    onclick: move |_| {
                                        show_cart.toggle();
                                        show_auth.set(false);
                                    },
                                    i { class: "fa-solid fa-basket-shopping" }
                                    span { "{cart_count}" }
                                }
                                if show_cart() {
                                    CartDropdown { on_close: move |_| show_cart.set(false) }
                                }
                            }
                        }
                    }
                }
            }

            // Sidebar
            if show_sidebar() {
                div { class: "fixed inset-0 z-[100] flex",
                    div {
                        class: "absolute inset-0 bg-black/50 transition-opacity",
                        onclick: move |_| show_sidebar.set(false),
                    }
                    div { class: "relative w-80 bg-white h-full shadow-xl flex flex-col",
                        div { class: "p-6 flex justify-between items-center border-b bg-gray-700 text-white",
                            Link {
                                to: Route::Category { id: 0.into() },
                                onclick: move |_| show_sidebar.set(false),
                                h2 { class: "text-xl font-black hover:text-green-400 cursor-pointer",
                                    "Kategorier"
                                }
                            }
                            button { onclick: move |_| show_sidebar.set(false),
                                i { class: "fa-solid fa-xmark text-2xl" }
                            }
                        }
                        div { class: "flex-grow overflow-y-auto p-4",
                            // hårdkodade kategorier
                            // TODO(db): Ersätt hårdkodade SidebarCategory-anrop med kategorier från databasen
                            // TODO(db): Subkategorier ska också hämtas från databasen per kategori
                            match &*categories.read() {
                                None => rsx! {
                                    p { class: "text-gray-400 text-sm p-4", "Laddar..." }
                                },
                                Some(trees) => rsx! {
                                    for tree in trees.iter() {
                                        SidebarCategory {
                                            title: tree.name.to_string(),
                                            id: tree.id.get(),
                                            subcategories: tree.subcategories.iter().map(|s| (s.name.to_string(), s.id.get())).collect(),
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
            }

            Outlet::<Route> {}
        }
    }
}
