use crate::database::{Id, Vendor};
use dioxus::prelude::*;

/// The profile page for a vendor.
/// # Arguments
/// * `id` - The vendor ID.
#[allow(clippy::same_name_method, reason = "Dioxus macro limitation")]
#[component]
pub fn VendorPage(id: Id<Vendor>) -> Element {
    rsx! { "Page of vendor {id}" }
}
