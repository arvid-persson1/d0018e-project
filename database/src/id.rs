//! The [`Id`] type and associated items.

use std::{fmt::Debug, hash::Hash, marker::PhantomData};

/// Type-safe ID used as primary key in database queries.
///
/// IDs returned by methods on [`Connection`](crate::Connection) are valid (refer to a row in the
/// corresponding table in the database) at the time of retrieval from the database, but may of
/// course be invalidated later as a result of deletions. Although not documented, all said methods
/// will fail if provided with an invalid ID.
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

/// Dummy to create externally-inaccessible public items.
mod private {
    /// Dummy used to seal a trait.
    #[expect(unnameable_types, reason = "Intentional in order to seal trait.")]
    pub trait Sealed {}
}
use private::Sealed;

/// A type that can be used as an [`Id`].
pub trait Key: Sealed {}

/// Common trait for user IDs.
pub trait UserSuper: Clone + Copy + Debug + Default + PartialEq + Eq + Hash {}
// WARN: Due to trait coherence, this is the only blanket implementation we can have, meaning with
// this design we can only have one supertrait total. The decision has been made to have that be
// `UserSuper` as there may be more user roles in the future.
impl<U: UserSuper> Sealed for U {}
impl<U: UserSuper> Key for U {}

/// Marker for user IDs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct User;
impl UserSuper for User {}

/// Marker for customer IDs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Customer;
impl UserSuper for Customer {}

/// Marker for vendor IDs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Vendor;
impl UserSuper for Vendor {}

/// Marker for administrator IDs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Administrator;
impl UserSuper for Administrator {}

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
