//! Rudimentary authentication.

use crate::{
    Id,
    database::{Administrator, Customer, Username, Vendor},
};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

// TODO: Move this to `database`?

/// Get information about the currently logged in user, if any.
#[expect(clippy::missing_errors_doc, reason = "TODO")]
#[server]
pub async fn logged_in() -> Result<Option<Login>> {
    todo!()
}

/// Information about a login session.
// TODO: Include profile picture.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Login {
    /// The username of the logged in user.
    pub username: Username,
    /// The ID of the logged in user.
    pub id: LoginId,
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
