use crate::database::{Id, Vendor};
use dioxus::prelude::*;

/// The profile page for a vendor.
/// # Arguments
/// * `id` - The vendor ID.
#[component]
pub fn VendorPage(id: Id<Vendor>) -> Element {
    rsx! { "Page of vendor {id}" }
}
