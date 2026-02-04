//! Shared types and interfaces for interacting with the database.

mod types;
pub use types::*;

mod id;
pub use id::*;

mod connection;
#[cfg_attr(
    not(feature = "server"),
    expect(
        unused_imports,
        reason = "Public items are exported with the server feature."
    ),
    expect(
        unreachable_pub,
        reason = "Public items are exported with the server feature."
    )
)]
pub use connection::*;

mod auth;
pub use auth::*;
