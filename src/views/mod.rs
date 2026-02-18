//! Shared views.

mod home;
pub use home::Home;

mod product;
pub use product::Product;

mod customer;
pub use customer::CustomerProfile;

mod vendor;
pub use vendor::VendorProfile;

mod category;
pub use category::Category;

mod admin;
pub use admin::Administration;

mod favorites;
pub use favorites::Favorites;