use crate::{Id, Vendor};
use dioxus::prelude::*;

/// The profile page for a vendor.
#[component]
pub fn VendorPage(id: Id<Vendor>) -> Element {
    rsx! { "Page of vendor {id}" }
}
