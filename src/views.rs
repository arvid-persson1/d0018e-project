/// See [`Home`].
mod home;
pub use home::Home;

/// See [`ProductPage`].
mod product;
pub use product::Product;

/// See [`CustomerProfile`] / [`ProfilePage`].
mod customer;
pub use customer::CustomerProfile;
pub use customer::CustomerProfile as ProfilePage;

/// See [`VendorPage`].
mod vendor;
pub use vendor::VendorPage;
pub use vendor::VendorPage as Vendor;

/// See [`CategoryPage`].
mod category;
pub use category::CategoryPage;

/// See [`FavoritesPage`].
mod favorites;
pub use favorites::FavoritesPage;

/// See [`Search`].
mod search;
pub use search::Search;

/// Auth views: Login, Register, VendorLogin, VendorRegister.
mod auth;
pub use auth::{Login, Register, VendorLogin, VendorRegister};

/// Se [`CartPage`].
mod cart;
pub use cart::CartPage;