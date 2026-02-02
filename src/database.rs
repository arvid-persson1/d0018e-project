//! Shared types and interfaces for interacting with the database.

/// The maximum length of a username in characters (not bytes).
pub const USERNAME_MAX_LENGTH: usize = 20;

mod types;
pub use types::*;

mod id;
pub use id::*;

#[cfg(feature = "server")]
mod connection;
#[cfg(feature = "server")]
pub use connection::Connection;
