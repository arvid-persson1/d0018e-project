//! Shared views.

#![allow(
    clippy::missing_docs_in_private_items,
    reason = "Violated but not silenced by Dioxus."
)]
#![allow(
    clippy::same_name_method,
    reason = "Violated but not silenced by Dioxus."
)]

/// See [`Navbar`].
mod navbar;
pub use navbar::Navbar;

/// See [`Home`].
mod home;
pub use home::Home;

/// See [`ProductPage`]
mod product;
pub use product::ProductPage;

/// See [`Profile`].
mod profile;
pub use profile::Profile;

/// See [`VendorPage`].
mod vendor;
pub use vendor::VendorPage;

/// See [`CategoryPage`]
mod category;
pub use category::CategoryPage;

/// See [`FavoritesPage`].
mod favorites;
pub use favorites::FavoritesPage;
