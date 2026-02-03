use dioxus::prelude::*;

/// The page for a product page
#[component]
pub fn Product(id: i32) -> Element {
    rsx! { "Product {id}" }
}
