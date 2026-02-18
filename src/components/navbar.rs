use crate::Route;
use crate::state::GlobalState;
use dioxus::prelude::*;

// Class for the category navigation bar

#[component]
fn SidebarCategory(title: String, subcategories: Vec<String>) -> Element {
    let mut is_open = use_signal(|| false);

    let rotation = if is_open() { "rotate-180" } else { "" };

    rsx! {
        div { class: "flex flex-col w-full border-b border-gray-100",
            div {
                class: "flex justify-between items-center py-4 px-2 cursor-pointer hover:bg-green-50 transition-colors",
                onclick: move |_| is_open.toggle(),
                span { class: "font-bold text-gray-800", "{title}" }

                i { class: "fa-solid fa-chevron-down transition-transform duration-300 {rotation}" }
            }

            if is_open() {
                div { class: "bg-gray-50 flex flex-col pb-2",
                    for sub in subcategories {
                        a {
                            class: "pl-6 py-2 text-gray-600 hover:text-green-700 hover:bg-gray-100 text-sm transition-colors",
                            href: "#",
                            "{sub}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn Navbar() -> Element {
    let mut show_sidebar = use_signal(|| false);

    let global_state = use_context::<Signal<GlobalState>>();

    let fav_count = global_state.read().favorites.len();

    rsx! {

        rect {

            header { class: "w-full sticky top-0 z-50",
                nav { class: "bg-gray-700 text-white p-5 flex items-center shadow-lg",
                    div { class: "container mx-auto flex items-center justify-between gap-4",

                        // Logga
                        Link {
                            to: Route::Home {},
                            class: "bg-green-900 px-4 py-1 rounded-lg border border-green-600 shadow-inner flex items-center justify-center hover:bg-green-800 transition-colors cursor-pointer",
                            span { class: "text-3xl font-black italic tracking-tighter text-white",
                                "boop"
                            }
                        }

                        // Kategori
                        div { class: "flex justify-center items-center py-2 shadow-sm",
                            button {
                                class: "flex items-center justify-center w-9 h-9 rounded-full bg-gray-100 text-gray-700 hover:bg-green-100 hover:text-green-700 transition-all duration-200 shadow-sm border border-gray-200",
                                onclick: move |_| show_sidebar.set(true),
                                i { class: "fa-solid fa-bars text-xl" }
                            }
                        }

                        // Sökfält
                        div { class: "flex-grow max-w-2xl relative hidden md:block",
                            i { class: "fa-solid fa-magnifying-glass absolute left-4 top-1/2 -translate-y-1/2 text-gray-400" }
                            input {
                                r#type: "text",
                                placeholder: "Sök på boop...",
                                class: "w-full py-2.5 pl-12 pr-4 rounded-full text-black bg-white focus:outline-none focus:ring-4 focus:ring-green-500/30 transition-all",
                            }
                        }

                        div { class: "flex items-center gap-6",
                            // favoriter
                            Link {
                                to: Route::Favorites {},
                                class: "relative flex flex-col items-center hover:text-green-200 cursor-pointer transition",
                                i { class: "fa-solid fa-heart text-2xl" }
                                span { class: "text-[10px] font-bold uppercase", "Favoriter" }

                                // ANVÄND DIN NYA VARIABEL HÄR:
                                if fav_count > 0 {
                                    span { class: "absolute -top-1 -right-1 bg-red-500 text-white text-[10px] rounded-full w-5 h-5 flex items-center justify-center border-2 border-green-700",
                                        "{fav_count}"
                                    }
                                }
                            }

                            // konto
                            div { class: "flex flex-col items-center hover:text-green-200 cursor-pointer transition",
                                i { class: "fa-solid fa-circle-user text-2xl" }
                                span { class: "text-[10px] font-bold uppercase", "Konto" }
                            }

                            // varukorg
                            button { class: "bg-white text-green-700 px-5 py-2 rounded-full font-black flex items-center gap-2 hover:bg-green-50 transition shadow-sm",
                                i { class: "fa-solid fa-basket-shopping" }
                                span { "{global_state().cart_count}" }
                            }
                        }
                    }
                }
            }

            // Meny
            if show_sidebar() {
                div { class: "fixed inset-0 z-[100] flex",
                    div {
                        class: "absolute inset-0 bg-black/50 transition-opacity",
                        onclick: move |_| show_sidebar.set(false),
                    }

                    div { class: "relative w-80 bg-white h-full shadow-xl flex flex-col",
                        div { class: "p-6 flex justify-between items-center border-b bg-gray-700 text-white",
                            h2 { class: "text-xl font-black", "Kategorier" }
                            button { onclick: move |_| show_sidebar.set(false),
                                i { class: "fa-solid fa-xmark text-2xl" }
                            }
                        }

                        div { class: "flex-grow overflow-y-auto p-4",
                            SidebarCategory {
                                title: "Mejeri & Ägg".to_string(),
                                subcategories: vec![
                                    "Mjölk".to_string(),
                                    "Smör".to_string(),
                                    "Ost".to_string(),
                                    "Ägg".to_string(),
                                ],
                            }
                            SidebarCategory {
                                title: "Frukt & Grönt".to_string(),
                                subcategories: vec![
                                    "Bananer".to_string(),
                                    "Äpplen".to_string(),
                                    "Sallad".to_string(),
                                    "Rotfrukter".to_string(),
                                ],
                            }
                            SidebarCategory {
                                title: "Kött & Chark".to_string(),
                                subcategories: vec!["Nötkött".to_string(), "Kyckling".to_string(), "Korv".to_string()],
                            }
                            SidebarCategory {
                                title: "Skafferi".to_string(),
                                subcategories: vec!["Pasta".to_string(), "Ris".to_string(), "Konserver".to_string()],
                            }
                        }
                    }
                }
            }

            Outlet::<Route> {}
        }
    }
}
