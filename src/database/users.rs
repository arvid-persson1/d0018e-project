//! Database functions for interacting with users.

use crate::database::{Customer, Email, Id, Url, User, Username, Vendor};
use dioxus::prelude::*;
#[cfg(feature = "server")]
use {
    crate::database::{QueryResultExt as _, connection},
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
    let user = user.get();
    let mut tx = connection().begin().await?;

    // PERF: Several of these queries are not supported by indices: we imagine account deletions
    // are rare.

    // NOTE: Soft deletion. Possible corresponding row in role-specific table is also kept.
    query!(
        "
        UPDATE users
        SET deleted = true
        WHERE id = $1
        ",
        user
    )
    .execute(&mut *tx)
    .await?
    .by_unique_key(|| todo!())?;

    query!(
        "
        DELETE FROM products
        WHERE vendor = $1
        ",
        user
    )
    .execute(&mut *tx)
    .await?
    .allow_any();

    query!(
        "
        DELETE FROM special_offer_uses
        WHERE customer = $1
        ",
        user
    )
    .execute(&mut *tx)
    .await?
    .allow_any();

    // NOTE: Must be done before deleting rating.
    query!(
        "
        DELETE FROM reviews
        WHERE customer = $1
        ",
        user
    )
    .execute(&mut *tx)
    .await?
    .allow_any();

    query!(
        "
        DELETE FROM ratings
        WHERE customer = $1
        ",
        user
    )
    .execute(&mut *tx)
    .await?
    .allow_any();

    query!(
        "
        DELETE FROM review_votes
        WHERE customer = $1
        ",
        user
    )
    .execute(&mut *tx)
    .await?
    .allow_any();

    query!(
        "
        DELETE FROM comments
        WHERE user_id = $1
        ",
        user
    )
    .execute(&mut *tx)
    .await?
    .allow_any();

    query!(
        "
        DELETE FROM comment_votes
        WHERE customer = $1
        ",
        user
    )
    .execute(&mut *tx)
    .await?
    .allow_any();

    query!(
        "
        DELETE FROM shopping_cart_items
        WHERE customer = $1
        ",
        user
    )
    .execute(&mut *tx)
    .await?
    .allow_any();

    query!(
        "
        DELETE FROM customer_favorites
        WHERE customer = $1
        ",
        user
    )
    .execute(&mut *tx)
    .await?
    .allow_any();

    tx.commit().await.map_err(|_err| todo!())
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
