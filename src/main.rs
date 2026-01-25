use dioxus::prelude::*;

use views::{Home, Navbar, Product};

mod components;
mod views;

/// Structure of internal routes in our app. Each variant represents a different URL pattern that
/// can be matched by the router. If that pattern is matched, the components for that route will be
/// rendered.
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
        #[route("/")]
        Home {},
        #[route("/product/:id")]
        Product { id: u32 },
}

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

/// The main component.
#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        Router::<Route> {}
    }
}
