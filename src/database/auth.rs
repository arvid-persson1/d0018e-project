//! Rudimentary authentication.

use crate::{
    Id,
    database::{Administrator, Customer, ProfilePicture, Username, Vendor},
};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// Get information about the currently logged in user, if any.
#[expect(clippy::missing_errors_doc, reason = "TODO")]
#[expect(clippy::unused_async, reason = "TODO")]
#[server]
pub async fn logged_in() -> Result<Option<Login>> {
    // TODO: Implement.
    eprintln!("Login unimplemented.");
    Ok(None)
}

// TODO: Register and login functions.

/// Information about a login session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Login {
    /// The ID of the logged in user.
    pub id: LoginId,
    /// The username of the logged in user.
    pub username: Username,
    /// The profile picture of the logged in user.
    pub profile_picture: ProfilePicture,
}

/// A user's role and their ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoginId {
    /// The user is a customer.
    Customer(Id<Customer>),
    /// The user is a vendor.
    Vendor(Id<Vendor>),
    /// The user is an administrator.
    Administrator(Id<Administrator>),
}

impl PartialEq<Id<Customer>> for LoginId {
    fn eq(&self, other: &Id<Customer>) -> bool {
        if let Self::Customer(id) = self {
            id == other
        } else {
            false
        }
    }
}

impl PartialEq<Id<Vendor>> for LoginId {
    fn eq(&self, other: &Id<Vendor>) -> bool {
        if let Self::Vendor(id) = self {
            id == other
        } else {
            false
        }
    }
}

impl PartialEq<Id<Administrator>> for LoginId {
    fn eq(&self, other: &Id<Administrator>) -> bool {
        if let Self::Administrator(id) = self {
            id == other
        } else {
            false
        }
    }
}
