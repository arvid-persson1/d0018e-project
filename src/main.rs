#![allow(clippy::cargo_common_metadata)]

use dioxus::prelude::*;

use views::{
    Administration, Category, CustomerProfile, Favorites, Home, Navbar, Product, VendorProfile,
};

mod components;
mod views;

/// Structure of internal routes in our app. Each variant represents a different URL pattern that
/// can be matched by the router. If that pattern is matched, the components for that route will be
/// rendered.
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
// TODO: Add fallback page.
enum Route {
    #[layout(Navbar)]
        #[route("/")]
        Home {},
        // Can only visit own profile; no ID needed.
        #[route("/profile")]
        CustomerProfile {},
        #[route("/favorites")]
        Favorites {},
        #[route("/vendor/:id")]
        VendorProfile { id: i32 },
        #[route("/product/:id")]
        Product { id: i32 },
        #[route("/category/:id")]
        Category { id: i32 },
        #[route("/admin")]
        Administration {}
}

fn main() {
    launch(App);
}

/// The main component.
#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.css") }

        Router::<Route> {}
    }
}
