use crate::{Category, Id};
use dioxus::prelude::*;

/// The page for a category.
#[component]
pub fn CategoryPage(id: Id<Category>) -> Element {
    rsx! { "Product {id}" }
}
