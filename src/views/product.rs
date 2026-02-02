use crate::{Id, Product};
use dioxus::prelude::*;

/// The page for a product.
#[component]
pub fn ProductPage(id: Id<Product>) -> Element {
    rsx! { "Product {id}" }
}
