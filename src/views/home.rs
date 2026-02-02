use dioxus::prelude::*;

/// The home page.
#[component]
pub fn Home() -> Element {
    rsx! {
        h1 { class: "text-sky-500", "Home page" }
    }
}
