//! The entrypoint for the app.

#![cfg_attr(feature = "server", feature(iter_collect_into))]
#![cfg_attr(feature = "server", feature(never_type))]

pub mod components;
use components::Navbar;
pub mod database;
///
pub mod views;
use database::{Category, Id, Vendor};
mod state;
use state::GlobalState;

use dioxus::prelude::*;
use views::{
    Administration, CategoryPage, CustomerProfile, FavoritesPage, Home, Login, Product,
    ProfilePage, Register, VendorLogin, VendorPage, VendorRegister,
};

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
    /// See [`CustomerProfile`].
    #[route("/customer-profile", CustomerProfile)]
    CustomerProfile,
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
    #[route("/product/:id", Product)]
    Product {
        /// The ID of the product.
        id: i32,
    },
    /// See [`CategoryPage`].
    #[route("/category/:id", CategoryPage)]
    Category {
        /// The ID of the category.
        id: Id<Category>,
    },
    /// See [`Login`].
    #[route("/login", Login)]
    Login,
    /// See [`Register`].
    #[route("/register", Register)]
    Register,
    /// See [`VendorLogin`].
    #[route("/vendor-login", VendorLogin)]
    VendorLogin,
    /// See [`VendorRegister`].
    #[route("/vendor-register", VendorRegister)]
    VendorRegister,
    /// See [`Administration`].
    #[route("/admin", Administration)]
    Administration,
    // TODO: Shopping cart page.
}

#[allow(non_snake_case)]
#[component]
fn App() -> Element {
    let _ = use_context_provider(|| Signal::new(GlobalState::default()));
    rsx! {
        // TODO: Is this required?
        document::Script { src: "https://cdn.tailwindcss.com" }

        // TODO: Inline icons.
        document::Link {
            rel: "stylesheet",
            href: "https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.4.2/css/all.min.css",
        }

        Router::<Route> {}
    }
}

fn main() {
    launch(App)
}
