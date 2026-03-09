//! Database functions for interacting with users.

use crate::database::{Customer, Email, Id, ProfilePicture, Url, User, Username, Vendor};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use {
    crate::database::{POOL, QueryResultExt},
    sqlx::{query, query_as},
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
    query!("CALL delete_user($1)", user.get())
        .execute(&*POOL)
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
        SET profile_picture = $2
        WHERE id = $1
        ",
        customer.get(),
        url as Url,
    )
    .execute(&*POOL)
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
        SET profile_picture = $2
        WHERE id = $1
        ",
        vendor.get(),
        url as Url,
    )
    .execute(&*POOL)
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
        SET username = $2
        WHERE id = $1
        ",
        user.get(),
        username as Username,
    )
    .execute(&*POOL)
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
        &email,
    )
    .execute(&*POOL)
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
    .execute(&*POOL)
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
    .execute(&*POOL)
    .await?
    .by_unique_key(|| todo!())
}

/// Information about a vendor, for display on their profile page.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VendorProfile {
    /// The username of the vendor. While public, this is not the primary name used to refer to the
    /// vendor, see `display_name`.
    pub username: Username,
    /// The profile picture of the vendor.
    pub profile_picture: ProfilePicture,
    /// The display name of the vendor.
    pub display_name: Box<str>,
    /// A description of the vendor.
    pub description: Box<str>,
}

#[cfg(feature = "server")]
struct VendorProfileRepr {
    username: String,
    profile_picture: Option<String>,
    display_name: String,
    description: String,
}

#[cfg(feature = "server")]
impl From<VendorProfileRepr> for VendorProfile {
    fn from(
        VendorProfileRepr {
            username,
            profile_picture,
            display_name,
            description,
        }: VendorProfileRepr,
    ) -> Self {
        Self {
            username: Username::new(username.into()).expect("Invalid username."),
            profile_picture: ProfilePicture::Vendor(profile_picture.map(Into::into)),
            display_name: display_name.into(),
            description: description.into(),
        }
    }
}

/// Get information about a vendor, for display on their profile page.
///
/// # Errors
///
/// Fails if:
/// - `vendor` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn vendor_profile(id: Id<Vendor>) -> Result<VendorProfile> {
    query_as!(
        VendorProfileRepr,
        r#"
        SELECT username, profile_picture, display_name, description
        FROM vendors v
        JOIN users ON users.id = v.id
        WHERE v.id = $1
        "#,
        id.get(),
    )
    .fetch_one(&*POOL)
    .await
    .map(Into::into)
    .map_err(Into::into)
}
