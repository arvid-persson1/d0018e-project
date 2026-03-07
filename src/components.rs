//! Shared UI components for boop

mod navbar;
pub use navbar::Navbar;

pub mod product_card;
pub use product_card::{ProductCard, ProductProps};

pub mod auth_dropdown;
pub use auth_dropdown::AuthDropdown;

pub mod cart_dropdown;
pub use cart_dropdown::CartDropdown;
