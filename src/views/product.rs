use dioxus::prelude::*;

/// The page for a product.
#[component]
pub fn Product(id: u32) -> Element {
    rsx! { "Product {id}" }
}
