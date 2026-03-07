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

/// See [`VendorProfile`] / [`VendorPage`].
mod vendor;
pub use vendor::VendorProfile;
pub use vendor::VendorProfile as VendorPage;

/// See [`CategoryPage`].
mod category;
pub use category::CategoryPage;

/// See [`FavoritesPage`].
mod favorites;
pub use favorites::FavoritesPage;

/// Auth views: Login, Register, VendorLogin, VendorRegister.
mod auth;
pub use auth::{Login, Register, VendorLogin, VendorRegister};

/// Administration view.
mod administration;
pub use administration::Administration;
