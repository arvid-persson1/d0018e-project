use dioxus::prelude::*;

/// The profile page for a vendor.
#[component]
pub fn VendorProfile(id: i32) -> Element {
    rsx! { "Profile of vendor {id}" }
}
