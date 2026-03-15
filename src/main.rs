//! The entrypoint for the app.

#![cfg_attr(feature = "server", feature(iter_collect_into))]
#![cfg_attr(feature = "server", feature(never_type))]
#![cfg_attr(feature = "web", allow(unused_crate_dependencies))]

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
    CategoryPage, CustomerProfile, FavoritesPage, Home, Login, Product,
    ProfilePage, Register, Search, VendorLogin, VendorPage, VendorRegister, CartPage,
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
    /// See [`Search`].
    #[route("/search/:query", Search)]
    Search {
        /// Söksträngen.
        query: String,
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
    /// Se [`CartPage`].
    #[route("/cart", CartPage)]
    Cart,
}

#[allow(non_snake_case)]
#[component]
fn App() -> Element {
    let mut global_state = use_context_provider(|| Signal::new(GlobalState::begin_auth()));

    let _effect = use_effect(move || {
        let _task = spawn(async move {
            #[cfg(feature = "web")]
            {
                use crate::database::{Id, RawId, User, login_info};
                use web_sys::{HtmlDocument, wasm_bindgen::JsCast as _, window};
                if let Some(window) = window()
                    && let Some(document) = window.document()
                    && let Ok(html) = document.dyn_into::<HtmlDocument>()
                    && let Ok(cookies) = html.cookie()
                {
                    if let Some(value) = cookies
                        .split(';')
                        .filter_map(|pair| pair.split_once('='))
                        .find(|(key, _)| key.trim() == "user_id")
                        .map(|(_, value)| value.trim().to_string())
                        && !value.is_empty()
                        && let Ok(id) = value.parse::<RawId>()
                    {
                        match login_info(Id::<User>::from(id)).await {
                            Ok(info) => {
                                global_state.write().login = Some(info);
                                let customer_id = global_state.read().customer_id();
                                if let Some(cid) = customer_id {
                                    // Ladda favoriter
                                    if let Ok(favs) = crate::database::products::favorites(cid, 1000, 0).await {
                                        global_state.write().favorites =
                                            favs.iter().map(|p| p.id.get()).collect();
                                    }
                                    // Hämta local cart (lagd innan auth var klar)
                                    let local_cart = global_state.read().cart.clone();
                                    // Synka med DB
                                    match crate::database::cart::cart_products(cid).await {
                                        Ok((db_products, _)) => {
                                            if db_products.is_empty() && !local_cart.is_empty() {
                                                // Local → DB
                                                for item in local_cart.iter() {
                                                    let pid = crate::database::Id::<crate::database::Product>::from(item.product_id);
                                                    drop(crate::database::cart::set_in_shopping_cart(cid, pid, item.quantity).await);
                                                }
                                            } else if !db_products.is_empty() {
                                                // DB → local state
                                                global_state.write().cart = db_products
                                                    .iter()
                                                    .map(|p| crate::state::CartItem {
                                                        product_id: p.id.get(),
                                                        name: p.name.to_string(),
                                                        price: p.price.to_string().parse::<f64>().unwrap_or(0.0),
                                                        image_url: p.thumbnail.to_string(),
                                                        quantity: p.count.get(),
                                                    })
                                                    .collect();
                                            }
                                        }
                                        Err(_) => {
                                            // DB misslyckades — synka local → DB ändå
                                            for item in local_cart.iter() {
                                                let pid = crate::database::Id::<crate::database::Product>::from(item.product_id);
                                                drop(crate::database::cart::set_in_shopping_cart(cid, pid, item.quantity).await);
                                            }
                                        }
                                    }
                                }
                            }
                            Err(_) => {}
                        }
                    }
                }
            }
            // Auth-checken är klar, oavsett resultat.
            global_state.write().auth_loading = false;
        });
    });
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
