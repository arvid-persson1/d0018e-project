//! Rudimentary authentication.

use crate::{
    Id,
    database::{Administrator, Customer, Username, Vendor},
};

// TODO: Move this to `database`?

/// Get information about the currently logged in user, if any.
#[inline]
#[must_use]
#[expect(clippy::todo, reason = "TODO")]
pub fn logged_in() -> Option<Login> {
    todo!()
}

/// Information about a login session.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Login {
    /// The username of the logged in user.
    pub username: Username,
    /// The ID of the logged in user.
    pub id: LoginId,
}

/// A user's role and their ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
