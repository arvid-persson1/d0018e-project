//! The fullstack web app.

#![feature(if_let_guard)]

use dioxus::prelude::*;
use views::{CategoryPage, FavoritesPage, Home, Navbar, ProductPage, Profile, VendorPage};
pub mod auth;
pub mod components;
pub mod database;
pub mod views;
use database::{Category, Id, Product, Vendor};

/// Structure of internal routes in our app. Each variant represents a different URL pattern that
/// can be matched by the router. If that pattern is matched, the components for that route will be
/// rendered.
#[derive(Debug, Clone, Routable, PartialEq)]
enum Route {
    #[layout(Navbar)]

    /// The home page.
    #[route("/")]
    Home,
    /// The profile page for a customer or vendor, or the administration page for an
    /// administrator.
    #[route("/profile")]
    Profile,
    /// The favorites page for a customer.
    #[route("/favorites")]
    FavoritesPage,
    /// The page for a vendor.
    #[route("/vendor/:id")]
    VendorPage {
        /// The ID of the vendor.
        id: Id<Vendor>,
    },
    /// The page for a product.
    #[route("/product/:id")]
    ProductPage {
        /// The ID of the product.
        id: Id<Product>,
    },
    /// The page for a category.
    #[route("/category/:id")]
    CategoryPage {
        /// The ID of the category.
        id: Id<Category>,
    },
}

fn main() {
    #[cfg(feature = "server")]
    {
        use dotenvy as _;
    }

    launch(App);
}

/// The main component.
#[component]
#[allow(
    clippy::volatile_composites,
    reason = "Violated but not silenced by Dioxus."
)]
fn App() -> Element {
    rsx! {
        Stylesheet { href: asset!("/assets/tailwind.css") }
        Router::<Route> {}
    }
}
