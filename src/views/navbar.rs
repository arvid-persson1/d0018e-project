use crate::{
    Route,
    database::{Login, categories::category_trees, logged_in},
};
use dioxus::prelude::*;

/// The navgation bar.
#[component]
pub fn Navbar() -> Element {
    // All database "getter" functions will have to be used as "resources".
    let username = use_resource(logged_in);
    #[expect(unused_variables, reason = "TODO")]
    let categories = use_resource(category_trees);
    // If a resource `foo` needs arguments, it would be called like this:
    // `use_resource(move || async move { foo(arg1, arg2).await })`
    // In some cases, `move` may be omitted.

    rsx! {
        nav {
            h1 { "Category menu button" }
            h1 { "Logo" }
            h1 { "Search bar" }
            // The main way to use resources will be through `read_unchecked`. If an (async) server
            // function returns `Result<T, E>`, a call to `read_unchecked` will return
            // `Option<Result<T, E>>` where `None` indicates that the resource is still processing
            // and the result is not ready yet.
            //
            // Here, specifically, we get `Option<Result<Option<Login, E>>>` (`E` is some concrete
            // error type subject to change) where `Some(Ok(Some(_)))` means a login session
            // exists and `Some(Ok(None))` means the user is not logged in.
            //
            // Note that due to "magic" happening in the `rsx` macro, we get live updates. This
            // unfortunately means we can't for example introduce an `Element` variable outside of
            // the block to reduce nesting. We also can't get ownership of the data as operations
            // on the resource pass through a non-trivial `Deref`.
            match &*username.read_unchecked() {
                Some(Ok(Some(Login { username, .. }))) => rsx! {
                    h1 { "{username}" }
                },
                Some(Ok(None)) => rsx! {
                    h1 { "Not logged in" }
                },
                None => rsx! {
                    h1 { "Loading" }
                },
                // TODO: Improve error handling by introducing error classification and customized
                // error page.
                Some(Err(e)) => return Err(e.clone().into()),
            }
            Link { to: Route::Home {}, "Home" }
            Link { to: Route::ProductPage { id: 1.into() }, "Sample product" }
        }
        Outlet::<Route> {}
    }
}
