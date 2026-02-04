//! Rudimentary authentication.

#![allow(clippy::future_not_send, reason = "Violated by the `#[server]` macro.")]

use crate::{
    Id,
    database::{Administrator, Customer, Username, Vendor},
};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

// TODO: Move this to `database`?

/// Get information about the currently logged in user, if any.
#[inline]
#[expect(clippy::missing_errors_doc, reason = "TODO")]
#[expect(clippy::todo, reason = "TODO")]
#[server]
pub(crate) async fn logged_in() -> Result<Option<Login>> {
    todo!()
}

/// Information about a login session.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Login {
    /// The username of the logged in user.
    pub username: Username,
    /// The ID of the logged in user.
    pub id: LoginId,
}

/// A user's role and their ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LoginId {
    /// The user is a customer.
    Customer(Id<Customer>),
    /// The user is a vendor.
    Vendor(Id<Vendor>),
    /// The user is an administrator.
    Administrator(Id<Administrator>),
}

impl PartialEq<Id<Customer>> for LoginId {
    #[inline]
    fn eq(&self, other: &Id<Customer>) -> bool {
        if let Self::Customer(id) = self {
            id == other
        } else {
            false
        }
    }
}

impl PartialEq<Id<Vendor>> for LoginId {
    #[inline]
    fn eq(&self, other: &Id<Vendor>) -> bool {
        if let Self::Vendor(id) = self {
            id == other
        } else {
            false
        }
    }
}

impl PartialEq<Id<Administrator>> for LoginId {
    #[inline]
    fn eq(&self, other: &Id<Administrator>) -> bool {
        if let Self::Administrator(id) = self {
            id == other
        } else {
            false
        }
    }
}
