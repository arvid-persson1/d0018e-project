//! The [`Id`] type and associated items.

use nameof::name_of_type;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Borrow,
    fmt::{Debug, Display, Error as FmtError, Formatter},
    hash::{Hash, Hasher},
    marker::PhantomData,
    num::ParseIntError,
    str::FromStr,
};

/// The type internally used in the database to represent IDs.
pub type RawId = i32;

/// Type-safe ID used as primary key in database queries.
///
/// An ID is considered *valid* if a row exists in the corresponding table in the database with the
/// ID. There is no way to check if an ID is valid since any result would be worthless as it could
/// change before anything useful is done with the information.
///
/// Instances of this type are not intended to be created manually, as a manually created ID is
/// likely invalid.
///
/// [`Id`]s created by database functions are valid at the time of retrieval from the database, but
/// might of course be invalidated later as a result of deletions. It might even be the case that
/// an ID is invalidated in the time between the ID being fetched from the database and the
/// associated [`Future`] completing.
#[derive(Serialize, Deserialize)]
// FIXME: `PhantomData` may be overly restrictive here when considering variance.
pub struct Id<T: Key>(RawId, PhantomData<T>);

impl<T: Key> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Key> Copy for Id<T> {}

impl<T: Key> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        let Self(i, _) = self;
        *i == other.0
    }
}

impl<T: Key> Eq for Id<T> {}

impl<T: Key> Hash for Id<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Self(i, _) = self;
        i.hash(state);
    }
}

impl<T: Key> Debug for Id<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        let Self(i, _) = self;
        f.debug_tuple(name_of_type!(Self)).field(i).finish()
    }
}

impl<T: Key> From<RawId> for Id<T> {
    fn from(value: RawId) -> Self {
        Self(value, PhantomData)
    }
}

impl<T: Key> From<Id<T>> for RawId {
    fn from(value: Id<T>) -> Self {
        let Id(i, _) = value;
        i
    }
}

impl<T: Key> Id<T> {
    /// Get the inner type-erased ID.
    ///
    /// This is equivalent to [`into`](Into::into), but with a known output type.
    #[must_use]
    pub const fn get(self) -> RawId {
        let Self(i, _) = self;
        i
    }
}

impl<T: Key> FromStr for Id<T> {
    type Err = ParseIntError;
    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        s.parse().map(|i| Self(i, PhantomData))
    }
}

impl<T: Key> Display for Id<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        let Self(i, _) = self;
        write!(f, "{i}")
    }
}

impl<T: Key> Borrow<RawId> for Id<T> {
    fn borrow(&self) -> &RawId {
        &self.0
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

/// Marker for order IDs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Order;
impl Sealed for Order {}
impl Key for Order {}
