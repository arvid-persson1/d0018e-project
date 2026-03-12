//! Shared types and interfaces for interacting with the database.

#![allow(clippy::shadow_unrelated, reason = "Common in mappings.")]
#![allow(
    clippy::missing_docs_in_private_items,
    reason = "Generates a lot of noise for mappings. Documentation is delegated to either corresponding public items, or database schema."
)]

// TODO: Replace `#[server]` usage with method-specific attributes.

#[cfg(feature = "server")]
use {
    crate::dioxus_fullstack::Lazy,
    sqlx::{PgPool as Pool, postgres::PgQueryResult as QueryResult, query},
    thiserror::Error,
};

mod types;
pub use types::*;

mod id;
pub use id::*;

pub mod auth;
pub use auth::*;

// FIXME: It's possible that `Decimal`s will have to be rescaled, clamped, truncated or rounded
// before insertion into the database. This might warrant a newtype.

// TODO: Is it possible to have borrowed arguments in server functions?

// TODO: Consider having functions that create or update rows return the IDs.

pub mod cart;
pub mod categories;
pub mod offers;
pub mod products;
pub mod reviews;
pub mod search;
pub mod users;

#[cfg(feature = "server")]
static POOL: Lazy<Pool> = Lazy::new(|| async move {
    let database_url = dotenvy::var("DATABASE_URL").expect("`DATABASE_URL` not set.");

    let pool = Pool::connect(&database_url)
        .await
        .expect("Failed to establish a connection to the database.");

    #[expect(clippy::unwrap_used, reason = "Cell was just initialized.")]
    let res = query!("SELECT process_expiries();")
        .fetch_all(&pool)
        .await
        .expect("Failed to run database startup code.");
    drop(res);

    Ok::<_, !>(pool)
});

#[cfg(feature = "server")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Error)]
#[error("The provided key was invalid.")]
struct InvalidKey;

/// Extension trait to make decisions based on the number of rows affected by a query.
///
/// See [`QueryResult`].
#[cfg(feature = "server")]
trait QueryResultExt: Sized {
    /// Allow any number of rows to have been affected, discarding the result.
    fn allow_any(self) {}

    /// Assert that exactly one row was affected, panic otherwise.
    ///
    /// # Panics
    ///
    /// Panics if zero or more than one row were affected.
    fn expect_one(self);

    /// Assert that zero or one row was affected, panic otherwise.
    ///
    /// # Panics
    ///
    /// Panics if more than one row were affected.
    fn expect_maybe(self);

    /// Assert that the query was a call to a stored procedure.
    ///
    /// This function *has false positives* since a normal query could also affect 0 rows.
    ///
    /// # Panics
    ///
    /// Panics if any rows were affected, as procedures always return 0.
    fn procedure(self);

    /// Assert that exactly one row was affected as the query specified a unique key, returning
    /// an error if the key didn't exist and panicking if it wasn't unique.
    ///
    /// # Errors
    ///
    /// Fails with the output of `on_zero` if the key did not exist or the query for some other
    /// reason did not affect any rows.
    ///
    /// # Panics
    ///
    /// Panics if the query affected multiple rows, i.e. the key wasn't unique.
    // TODO: Have this return `CapturedError` and perform conversion in the method.
    fn by_unique_key(self) -> Result<(), InvalidKey>;
}

#[cfg(feature = "server")]
impl QueryResultExt for QueryResult {
    fn expect_one(self) {
        match self.rows_affected() {
            0 => panic!("Query unexpectedly did not affect any rows."),
            1 => {},
            _ => panic!("Query unexpectedly affected several rows."),
        }
    }

    fn expect_maybe(self) {
        match self.rows_affected() {
            0 | 1 => {},
            _ => panic!("Query unexpectedly affected several rows."),
        }
    }

    fn procedure(self) {
        assert!(self.rows_affected() != 0, "Query was not a procedure call.");
    }

    #[expect(clippy::unreachable, reason = "Key enforces uniqueness.")]
    fn by_unique_key(self) -> Result<(), InvalidKey> {
        match self.rows_affected() {
            0 => Err(InvalidKey),
            1 => Ok(()),
            _ => unreachable!("Non-unique key."),
        }
    }
}
