use crate::components::Echo;
use dioxus::prelude::*;

/// The home page.
#[component]
pub fn Home() -> Element {
    rsx! {
        Echo {}
    }
}
