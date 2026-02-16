//! The [`Id`] type and associated items.

use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display, Error as FmtError, Formatter},
    hash::Hash,
    marker::PhantomData,
    num::ParseIntError,
    str::FromStr,
};

/// The type internally used in the database to represent IDs.
pub type Inner = i32;

/// Type-safe ID used as primary key in database queries.
///
/// An ID is considered *valid* if a row exists in the corresponding table in the database with the
/// ID. There is no way to check if an ID is valid since any result would be worthless as it could
/// change before anything useful is done with the information.
///
/// Instances of this type are not intended to be created manually, as a manually created ID is
/// likely invalid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
// FIXME: `PhantomData` may be overly restrictive here when considering variance.
pub struct Id<T: Key + ?Sized>(Inner, PhantomData<T>);

impl<T: Key> From<Inner> for Id<T> {
    #[inline]
    fn from(value: Inner) -> Self {
        Self(value, PhantomData)
    }
}

impl<T: Key> From<Id<T>> for Inner {
    #[inline]
    fn from(value: Id<T>) -> Self {
        let Id(i, _) = value;
        i
    }
}

impl<T: Key> FromStr for Id<T> {
    type Err = ParseIntError;
    #[inline]
    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        s.parse().map(|i| Self(i, PhantomData))
    }
}

impl<T: Key> Display for Id<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        let Self(i, _) = self;
        write!(f, "{i}")
    }
}

/// Dummy to create externally-inaccessible public items.
mod private {
    /// Dummy used to seal a trait.
    #[expect(unnameable_types, reason = "Intentional in order to seal trait.")]
    pub trait Sealed {}
}
use private::Sealed;

/// A type that can be used as an [`Id`].
pub trait Key: Sealed + 'static {}

/// Marker for product IDs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct User;
impl Sealed for User {}
impl Key for User {}

/// Marker for customer IDs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Customer;
impl Sealed for Customer {}
impl Key for Customer {}
impl From<Id<Customer>> for Id<User> {
    #[inline]
    fn from(value: Id<Customer>) -> Self {
        let Id(i, _) = value;
        Self(i, PhantomData)
    }
}

/// Marker for vendor IDs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Vendor;
impl Sealed for Vendor {}
impl Key for Vendor {}
impl From<Id<Vendor>> for Id<User> {
    #[inline]
    fn from(value: Id<Vendor>) -> Self {
        let Id(i, _) = value;
        Self(i, PhantomData)
    }
}

/// Marker for administrator IDs.
// NOTE: There is no `administrators` table, so the only use of `Id<Administrator>` is in its
// `Into<Id<User>>` implementation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Administrator;
impl Sealed for Administrator {}
impl Key for Administrator {}
impl From<Id<Administrator>> for Id<User> {
    #[inline]
    fn from(value: Id<Administrator>) -> Self {
        let Id(i, _) = value;
        Self(i, PhantomData)
    }
}

/// Marker for product IDs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Product;
impl Sealed for Product {}
impl Key for Product {}

/// Marker for category IDs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Category;
impl Sealed for Category {}
impl Key for Category {}

/// Marker for review IDs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Review;
impl Sealed for Review {}
impl Key for Review {}

/// Marker for comment IDs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Comment;
impl Sealed for Comment {}
impl Key for Comment {}

/// Marker for special offer IDs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpecialOffer;
impl Sealed for SpecialOffer {}
impl Key for SpecialOffer {}
