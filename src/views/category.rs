use dioxus::prelude::*;

/// The page for a category.
#[component]
pub fn Category(id: i32) -> Element {
    rsx! { "Product {id}" }
}
