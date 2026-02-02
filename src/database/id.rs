//! The [`Id`] type and associated items.

use std::{
    fmt::{Debug, Display, Error as FmtError, Formatter},
    hash::Hash,
    marker::PhantomData,
    num::ParseIntError,
    str::FromStr,
};

/// Type-safe ID used as primary key in database queries.
///
/// An ID is considered *valid* if a row exists in the corresponding table in the database with the
/// ID. There is no way to check if an ID is valid since any result would be worthless as it could
/// change before anything useful is done with the information.
///
/// Instances of this type are not intended to be created manually, although it is allowed (see
/// [`From`] implementation), as a manually created ID is likely invalid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id<T: Key>(i32, PhantomData<T>);

impl<T: Key> From<i32> for Id<T> {
    #[inline]
    fn from(value: i32) -> Self {
        Self(value, PhantomData)
    }
}

impl<T: Key> From<Id<T>> for i32 {
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
pub trait Key: Sealed {}

/// Common marker trait for user IDs.
pub trait User: Clone + Copy + Debug + Default + PartialEq + Eq + Hash {}
impl<U: User> Sealed for U {}
impl<U: User> Key for U {}

/// Marker for customer IDs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Customer;
impl User for Customer {}

/// Marker for vendor IDs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Vendor;
impl User for Vendor {}

/// Marker for administrator IDs.
// NOTE: There is no `administrators` table, so the only use of `Id<Administrator>` is as a
// realization of `Id<impl User>`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Administrator;
impl User for Administrator {}

/// Marker for product IDs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Product;
impl Sealed for Product {}
impl Key for Product {}

/// Marker for category IDs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Category;
impl Sealed for Category {}
impl Key for Category {}

/// Marker for review IDs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Review;
impl Sealed for Review {}
impl Key for Review {}

/// Marker for comment IDs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Comment;
impl Sealed for Comment {}
impl Key for Comment {}

/// Marker for special offer IDs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SpecialOffer;
impl Sealed for SpecialOffer {}
impl Key for SpecialOffer {}
