use crate::Route;
use dioxus::prelude::*;

/// The navgation bar.
#[component]
pub fn Navbar() -> Element {
    rsx! {
        nav {
            Link { to: Route::Home {}, "Home" }
            Link { to: Route::Product { id: 1 }, "Sample product" }
        }

        // Render the next component inside the layout.
        Outlet::<Route> {}
    }
}
