//! The entrypoint for the app.

#![feature(iter_collect_into)]
#![cfg_attr(feature = "server", expect(clippy::todo, reason = "TODO"))]

pub mod components;
use components::Navbar;
pub mod database;
pub mod views;
use database::{Category, Id, Product, Vendor};
mod fake_data;
mod state;

use dioxus::prelude::*;
use views::{CategoryPage, FavoritesPage, Home, ProductPage, ProfilePage, VendorPage};
#[cfg(feature = "server")]
use {database::init_connection, futures::executor::block_on};

/// Structure of all non-internal endpoints.
#[derive(Debug, Clone, PartialEq, Routable)]
enum Route {
    #[layout(Navbar)]

    /// See [`Home`].
    #[route("/")]
    Home,
    /// See [`ProfilePage`].
    #[route("/profile", ProfilePage)]
    Profile,
    /// See [`FavoritesPage`].
    #[route("/favorites", FavoritesPage)]
    Favorites,
    /// See [`VendorPage`].
    #[route("/vendor/:id", VendorPage)]
    Vendor {
        /// The ID of the vendor.
        id: Id<Vendor>,
    },
    /// See [`ProductPage`].
    #[route("/product/:id", ProductPage)]
    Product {
        /// The ID of the product.
        id: Id<Product>,
    },
    /// See [`CategoryPage`].
    #[route("/category/:id", CategoryPage)]
    Category {
        /// The ID of the category.
        id: Id<Category>,
    },
    // TODO: Shopping cart page.
}

#[component]
fn App() -> Element {
    rsx! {
        // TODO: Is this required?
        // script { src: "https://cdn.tailwindcss.com" }

        // TODO: Inline icons.
        // document::Link {
        //     rel: "stylesheet",
        //     href: "https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.4.2/css/all.min.css",
        // }

        Router::<Route> {}
    }
}

fn main() {
    #[cfg(feature = "server")]
    {
        let database_url = dotenvy::var("DATABASE_URL").expect("`DATABASE_URL` not set.");
        block_on(init_connection(database_url))
            .expect("Failed to establish a connection to the database.");
    }

    launch(App);
}
