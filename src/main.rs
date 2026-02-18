use dioxus::prelude::*;

use crate::state::GlobalState;
use crate::views::{
    Administration, Category, CustomerProfile, Favorites, Home, Product, VendorProfile,
};

use crate::components::navbar::Navbar;
mod state;

mod components;
mod views;

mod fake_data;

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
        #[route("/favorites")]
        Favorites {},
        #[route("/profile")]
        CustomerProfile {},
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
    launch(MainApp);
}

#[component]
fn MainApp() -> Element {
    use_context_provider(|| Signal::new(GlobalState {
        favorites: Vec::new(),
        cart_items: Vec::new(),
    }));

    rsx! {
        script { src: "https://cdn.tailwindcss.com" }
        document::Link {
            rel: "stylesheet",
            href: "https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.4.2/css/all.min.css",
        }

        Router::<Route> {}
    }
}
