use crate::Route;
use dioxus::prelude::*;

/// The navgation bar.
#[component]
pub fn Navbar() -> Element {
    rsx! {
        nav {

            Link { to: Route::Home {}, "Home" }
            Link { to: Route::ProductPage { id: 1.into() }, "Sample product" }
        }
        Outlet::<Route> {
        }
    }
}
