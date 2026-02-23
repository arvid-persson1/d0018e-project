//! The fullstack web app.

#![feature(if_let_guard)]
#![feature(stmt_expr_attributes)]
#![feature(iter_collect_into)]
#![cfg_attr(feature = "server", expect(clippy::todo, reason = "TODO"))]

use dioxus::prelude::*;
use views::{CategoryPage, FavoritesPage, Home, Navbar, ProductPage, Profile, VendorPage};
#[cfg(feature = "server")]
use {database::init_connection, futures::executor::block_on};

pub mod components;
pub mod views;

pub mod database;
use database::{Category, Id, Product, Vendor};

/// Structure of internal routes in our app. Each variant represents a different URL pattern that
/// can be matched by the router. If that pattern is matched, the components for that route will be
/// rendered.
#[derive(Debug, Clone, PartialEq, Routable)]
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

/// # Panics
///
/// Panics if the `DATABASE_URL` environment variable is not set, or if an error occurs during
/// communication or internally in the database.
fn main() {
    #[cfg(feature = "server")]
    {
        let database_url = dotenvy::var("DATABASE_URL").expect("`DATABASE_URL` not set.");
        block_on(init_connection(database_url))
            .expect("Failed to establish a connection to the database.");
    }

    launch(App);
}
