//! Database functions for interacting with users.

use crate::database::{Customer, Email, Id, Url, User, Username, Vendor};
use dioxus::prelude::*;
#[cfg(feature = "server")]
use {
    crate::database::{QueryResultExt, connection},
    sqlx::query,
};

/// Mark a user as deleted.
///
/// This deletes their reviews, shopping cart, favorites, votes and more (if they were a customer),
/// products (if they were a vendor), as well as their comments. Order history is kept if they
/// were a customer.
///
/// # Errors
///
/// Fails if:
/// - `user` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn delete_user(user: Id<User>) -> Result<()> {
    query!("SELECT delete_user($1)", user.get())
        .execute(connection())
        .await
        .map(QueryResultExt::procedure)
        .map_err(Into::into)
}

/// Set a customer's profile picture.
///
/// # Errors
///
/// Fails if:
/// - `customer` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_customer_profile_picture(customer: Id<Customer>, url: Url) -> Result<()> {
    query!(
        "
        UPDATE customers
        SET profile_picture = $2::TEXT
        WHERE id = $1
        ",
        customer.get(),
        &url,
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}

/// Set a vendor's profile picture.
///
/// # Errors
///
/// Fails if:
/// - `vendor` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_vendor_profile_picture(vendor: Id<Vendor>, url: Url) -> Result<()> {
    query!(
        "
        UPDATE vendors
        SET profile_picture = $2::TEXT
        WHERE id = $1
        ",
        vendor.get(),
        &url,
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}

/// Set a user's username.
///
/// # Errors
///
/// Fails if:
/// - `user` is invalid.
/// - `username` is already taken.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_username(user: Id<User>, username: Username) -> Result<()> {
    query!(
        "
        UPDATE users
        SET username = $2::TEXT
        WHERE id = $1
        ",
        user.get(),
        &*username,
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}

/// Set a user's email.
///
/// # Errors
///
/// Fails if:
/// - `user` is invalid.
/// - `email` is already associated with another user.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_email(user: Id<User>, email: Email) -> Result<()> {
    query!(
        "
        UPDATE users
        SET email = $2::CITEXT
        WHERE id = $1
        ",
        user.get(),
        &*email,
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}

/// Set a vendor's display name.
///
/// # Errors
///
/// Fails if:
/// - `vendor` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_vendor_display_name(vendor: Id<Vendor>, display_name: Box<str>) -> Result<()> {
    query!(
        "
        UPDATE vendors
        SET display_name = $2
        WHERE id = $1
        ",
        vendor.get(),
        &display_name,
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}

/// Set a vendor's description.
///
/// # Errors
///
/// Fails if:
/// - `vendor` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_vendor_description(vendor: Id<Vendor>, description: Box<str>) -> Result<()> {
    query!(
        "
        UPDATE vendors
        SET description = $2
        WHERE id = $1
        ",
        vendor.get(),
        &description,
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}
