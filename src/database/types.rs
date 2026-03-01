//! Types produced by database operations.

use derive_more::{Deref, Display, Into};
use regex::Regex;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use sqlx::Type;
use std::{
    cmp::Ordering,
    fmt::{Display, Error as FmtError, Formatter},
    num::{NonZero, NonZeroU8, NonZeroU32},
    sync::LazyLock,
};
use thiserror::Error;

/// URL to an external resource, owned.
pub type Url = Box<str>;

/// Quantity of a product, possibly along with unit, e.g. "4.2 kg" or "8.15 dl".
///
/// If no unit is specified, the quantity is assumed to be in discrete amounts, and must be an
/// integer.
///
/// This type is oblivious to any actual meaning behind the units, so it can't for example handle
/// conversions.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(Type))]
pub struct Amount {
    /// The quantity.
    quantity: Decimal,
    /// The unit.
    unit: Option<Box<str>>,
}

#[expect(missing_docs, reason = "TODO")]
impl Amount {
    // TODO: Add error variants.
    #[must_use]
    pub fn new(quantity: Decimal, unit: Option<Box<str>>) -> Option<Self> {
        if quantity < Decimal::ZERO {
            None
        } else if unit.is_some() {
            Some(Self { quantity, unit })
        } else if quantity.is_integer() {
            Some(Self {
                quantity,
                unit: None,
            })
        } else {
            None
        }
    }

    #[must_use]
    pub fn discrete(quantity: u32) -> Self {
        Self {
            quantity: quantity.into(),
            unit: None,
        }
    }

    #[must_use]
    pub const fn with_unit(quantity: Decimal, unit: Box<str>) -> Self {
        Self {
            quantity,
            unit: Some(unit),
        }
    }

    /// Get the quantity. If [`unit`](Self::unit) is [`None`], this will be an integer.
    #[must_use]
    pub const fn quantity(&self) -> Decimal {
        self.quantity
    }

    /// Get the unit. If this is [`None`], [`quantity`](Self::quantity) will be an integer.
    #[must_use]
    pub fn unit(&self) -> Option<&str> {
        self.unit.as_deref()
    }
}

impl PartialOrd for Amount {
    /// Compare the amounts if their units are equal, otherwise returns `None`.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let Self { quantity, unit } = self;
        (*unit == other.unit).then(|| quantity.cmp(&other.quantity))
    }
}

impl Display for Amount {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        let Self { quantity, unit } = self;
        if let Some(unit) = unit {
            write!(f, "{quantity:.2} {unit}")
        } else {
            write!(f, "{quantity}")
        }
    }
}

/// The rating of a product. Will be between 1 and 5.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Into, Deref, Serialize, Deserialize,
)]
// TODO: Derive `Deserialize` manually, disallowing out-of-range values.
// TODO: Add constant presets.
#[repr(transparent)]
pub struct Rating(NonZeroU8);

impl Rating {
    /// A one-star rating.
    pub const ONE_STAR: Self = Self(NonZero::new(1).unwrap());
    /// A two-star rating.
    pub const TWO_STARS: Self = Self(NonZero::new(2).unwrap());
    /// A three-star rating.
    pub const THREE_STARS: Self = Self(NonZero::new(3).unwrap());
    /// A four-star rating.
    pub const FOUR_STARS: Self = Self(NonZero::new(4).unwrap());
    /// A five-star rating.
    pub const FIVE_STARS: Self = Self(NonZero::new(5).unwrap());

    /// Verify the rating is in range and constructs a `Rating` on success.
    #[must_use]
    #[expect(clippy::missing_panics_doc, reason = "See note.")]
    pub fn new(r: u8) -> Option<Self> {
        (1..=5).contains(&r).then(|| {
            #[expect(clippy::unwrap_used, reason = "Just verified.")]
            Self(NonZero::new(r).unwrap())
        })
    }

    /// Get the inner numerical rating value.
    ///
    /// This is equivalent to [`into`](Into::into), but with a known output type.
    #[must_use]
    pub const fn get(self) -> NonZeroU8 {
        let Self(r) = self;
        r
    }
}

/// The average rating of a product (between 1 and 5), as well as the number of ratings
/// contributing to that score.
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
// TODO: Derive `Deserialize` manually, disallowing out-of-range values.
pub struct AverageRating {
    /// The average rating, unspecified if `count == 0`.
    rating: f64,
    /// The number of ratings contributing to the score.
    count: u64,
}

impl PartialOrd for AverageRating {
    /// Compare the ratings, ignoring the counts.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AverageRating {
    /// Compare the ratings, ignoring the counts.
    fn cmp(&self, other: &Self) -> Ordering {
        let Self { rating, count: _ } = self;
        #[expect(clippy::unwrap_used, reason = "The rating is known to be real.")]
        rating.partial_cmp(&other.rating).unwrap()
    }
}

impl PartialEq for AverageRating {
    /// Compare the ratings, ignoring the counts.
    fn eq(&self, other: &Self) -> bool {
        let Self { rating, count: _ } = self;
        // The rating is known to be real.
        *rating == other.rating
    }
}

impl Eq for AverageRating {}

impl AverageRating {
    /// Verify the rating is in range and constructs an `AverageRating` on success.
    #[must_use]
    pub fn new(rating: f64, count: u64) -> Option<Self> {
        if count == 0 {
            Some(Self {
                rating: 0.,
                count: 0,
            })
        } else if (1. ..=5.).contains(&rating) {
            Some(Self { rating, count })
        } else {
            None
        }
    }

    /// Get the average rating if there are any.
    #[must_use]
    pub const fn rating(self) -> Option<f64> {
        if self.count > 0 {
            Some(self.rating)
        } else {
            None
        }
    }

    /// Get the number of ratings.
    #[must_use]
    pub const fn count(self) -> u64 {
        self.count
    }
}

impl From<Rating> for AverageRating {
    fn from(value: Rating) -> Self {
        let Rating(r) = value;
        Self {
            rating: f64::from(r.get()),
            count: 1,
        }
    }
}

impl Display for AverageRating {
    /// Format the rating with a single decimal point.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        let Self { rating, count } = *self;
        if count > 0 {
            write!(f, "No ratings")
        } else {
            write!(f, "{rating:.1}")
        }
    }
}

// NOTE: No `NonFutureTimestamp` or `NonFutureDate` is implemented here as "proper" usage could
// cause creating a valid instance, but not consuming it until the time has passed and the instance
// is invalidated. Hence, this validation is done only in the database.

/// A valid username.
///
/// A username may contain letters, numbers, underscores and dashes. Furthermore, usernames are
/// between 3 and 20 characters long. Note that other scripts are allowed, so this does not put
/// strict limits on byte length.
#[derive(
    Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Into, Deref, Serialize, Deserialize,
)]
// TODO: Derive `Deserialize` manually, disallowing invalid values.
#[repr(transparent)]
pub struct Username(Box<str>);

impl Username {
    /// Verifiy the format and constructs a `Username` on success.
    pub fn new(s: Box<str>) -> Option<Self> {
        static USERNAME_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"^[\w-]{3,20}$").expect("Failed to compile regex pattern.")
        });

        Some(s).filter(|s| USERNAME_PATTERN.is_match(s)).map(Self)
    }
}

// TODO: Link to specification.
/// A valid Email address, according to the HTML5 specification.
///
/// Notably, this is *not* compatible with RFC5322.
#[derive(
    Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Into, Deref, Serialize, Deserialize,
)]
// TODO: Derive `Deserialize` manually, disallowing invalid values.
#[repr(transparent)]
pub struct Email(Box<str>);

impl Email {
    /// Verifiy the format and constructs an `Email` on success.
    pub fn new(s: Box<str>) -> Option<Self> {
        static EMAIL_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").expect("Failed to compile regex pattern.")
        });

        Some(s).filter(|s| EMAIL_PATTERN.is_match(s)).map(Self)
    }
}

/// URL to the profile picture all admins use.
pub const ADMIN_PROFILE_PICTURE: &str = "";

/// A user's profile picture.
///
/// Customers and vendors can set their own profile pictures, while admins always have the same
/// fixed one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct ProfilePicture(Option<Url>);

impl ProfilePicture {
    /// Construct a new `ProfilePicture` for a customer or vendor from a URL.
    #[must_use]
    pub const fn new(url: Url) -> Self {
        Self(Some(url))
    }

    /// Construct a new `ProfilePicture` for an administrator.
    #[must_use]
    pub const fn admin() -> Self {
        Self(None)
    }

    /// Get whether the profile picture is [`ADMIN_PROFILE_PICTURE`].
    #[must_use]
    pub const fn is_admin(&self) -> bool {
        let Self(url) = self;
        url.is_none()
    }

    /// Returns the URL to the profile picture.
    #[must_use]
    pub fn url(&self) -> &str {
        let Self(url) = self;
        url.as_deref().unwrap_or(ADMIN_PROFILE_PICTURE)
    }
}

/// Details on the discount of a special offer.
///
/// Note that although an instance of this type is guaranteed to actually provide a discount at the
/// time of construction, it might be the case that it no longer does so later due to a price
/// change in the database. As such, attempting to insert it into the database might still result
/// in an error.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Into)]
#[repr(transparent)]
pub struct Deal(DealImpl);

impl Deal {
    /// Construct a new `Deal` offering free samples.
    ///
    /// Care should be taken to set a limit when inserting this deal into the database.
    #[must_use]
    pub const fn free() -> Self {
        Self(DealImpl::Discount {
            new_price: Decimal::ZERO,
        })
    }

    /// Construct a new `Deal` from the format used in the database.
    ///
    /// The following variants exists:
    /// 1. "NEW PRICE X" (sale) has `new_price` as <code>[Some]\(X)</code>, and both quantities as
    ///    [`None`].
    /// 2. "TAKE M PAY FOR N" has `quantity1` as <code>[Some]\(M)</code>, `quantity2` as
    ///    <code>[Some]\(N)</code> and `new_price` as [`None`].
    /// 3. "TAKE N PAY X" has `quantity1` as <code>[Some]\(N)</code>, `new_price` as
    ///    <code>[Some]\(X)</code>, and `quantity2` as [`None`].
    ///
    /// # Errors
    ///
    /// Returns [`InvalidVariant`](DealError::InvalidVariant) if none of the above cases
    /// match, or [`NoDiscount`](DealError::NoDiscount) if the deal is structurally valid
    /// but does not actually provide a discount compared to the price of the product.
    ///
    /// Silently overflows if either quantity is [`Some`] and negative.
    pub fn from_repr(
        new_price: Option<Decimal>,
        quantity1: Option<i32>,
        quantity2: Option<i32>,
        base_price: Decimal,
    ) -> Result<Self, DealError> {
        Self::try_from_repr(new_price, quantity1, quantity2, base_price)?
            .ok_or(DealError::InvalidVariant)
    }

    /// Construct a new `Deal` from the format used in the database.
    ///
    /// Unlike [`from_repr`], this returns an <code>[Option]\<Deal\></code>, allowing all fields to
    /// be [`None`] and then producing [`None`]. See [`from_repr`] for details.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidVariant`](DealError::InvalidVariant) if none of the above cases
    /// match, or [`NoDiscount`](DealError::NoDiscount) if the deal is structurally valid
    /// but does not actually provide a discount compared to the price of the product.
    ///
    /// [`from_repr`]: Self::from_repr
    pub fn try_from_repr(
        new_price: Option<Decimal>,
        quantity1: Option<i32>,
        quantity2: Option<i32>,
        base_price: Decimal,
    ) -> Result<Option<Self>, DealError> {
        const ONE: NonZeroU32 = NonZero::new(1).unwrap();

        if base_price <= Decimal::ZERO {
            return Err(DealError::ZeroPrice);
        }

        match (new_price, quantity1, quantity2) {
            (Some(new_price), None, None) => {
                if new_price < Decimal::ZERO {
                    Err(DealError::OutOfRange)
                } else if new_price >= base_price {
                    Err(DealError::NoDiscount)
                } else {
                    Ok(Some(Self(DealImpl::Discount { new_price })))
                }
            },
            (None, Some(take), Some(pay_for)) => {
                if take <= pay_for {
                    Err(DealError::NoDiscount)
                } else if let Some(take) = u32::try_from(take).ok().and_then(NonZeroU32::new)
                    && let Some(pay_for) = u32::try_from(pay_for).ok().and_then(NonZeroU32::new)
                {
                    Ok(Some(Self(DealImpl::Batch { take, pay_for })))
                } else {
                    Err(DealError::OutOfRange)
                }
            },
            (Some(pay), Some(take), None) => {
                if pay * Decimal::from(take) >= base_price {
                    Err(DealError::NoDiscount)
                } else if let Some(take) = u32::try_from(take).ok().and_then(NonZeroU32::new) {
                    Ok(Some(Self(if take == ONE {
                        // Paying X is equivalent to taking 1 and paying X, including when
                        // considering limits. For consistency, stick to the former.
                        DealImpl::Discount { new_price: pay }
                    } else {
                        DealImpl::BatchPrice { take, pay }
                    })))
                } else {
                    Err(DealError::OutOfRange)
                }
            },
            (None, None, None) => Ok(None),
            _ => Err(DealError::InvalidVariant),
        }
    }

    /// Convert a `Deal` into the format used in the database.
    ///
    /// Specifically, this returns a tuple representing the columns  `new_price`, `quantity1` and
    /// `quantity2` respectively on success. If either quantity is greater than `i32::MAX`, `None`
    /// is returned.
    #[must_use]
    pub fn database_repr(self) -> Option<(Option<Decimal>, Option<i32>, Option<i32>)> {
        let Self(deal) = self;
        match deal {
            DealImpl::Discount { new_price } => Some((Some(new_price), None, None)),
            DealImpl::Batch { take, pay_for }
                if let Ok(take) = take.get().try_into()
                    && let Ok(pay_for) = pay_for.get().try_into() =>
            {
                Some((None, Some(take), Some(pay_for)))
            },
            DealImpl::BatchPrice { take, pay } if let Ok(take) = take.get().try_into() => {
                Some((Some(pay), Some(take), None))
            },
            DealImpl::Batch { .. } | DealImpl::BatchPrice { .. } => None,
        }
    }

    /// Calculate the discount in percent as average per unit.
    ///
    /// If `base_price` is the price used during construction, this value will be between 0 and 1.
    /// If [`free`](Self::free) was used, this value will be 1.
    ///
    /// # Panics
    ///
    /// Panics if `base_price` is 0.
    #[must_use]
    pub fn average_discount(self, base_price: Decimal) -> Decimal {
        let Self(deal) = self;
        match deal {
            DealImpl::Discount { new_price } => Decimal::ONE - new_price / base_price,
            DealImpl::Batch { take, pay_for } => {
                Decimal::ONE - Decimal::from(pay_for.get()) / Decimal::from(take.get())
            },
            DealImpl::BatchPrice { take, pay } => {
                Decimal::ONE - pay / (base_price * Decimal::from(take.get()))
            },
        }
    }

    /// Calculate the price of a bunch of units of a product using this deal.
    ///
    /// Returns a tuple (`price`, `uses`) where `price` is the final price after discounts and
    /// `uses` is how many times the special offer was applied to get that price.
    #[must_use]
    pub fn discounted_price(
        self,
        units: NonZeroU32,
        price_per_unit: Decimal,
        limit: Option<u32>,
    ) -> (Decimal, u32) {
        let limit = limit.unwrap_or(u32::MAX);
        let units = units.get();
        let Self(deal) = self;
        match deal {
            DealImpl::Discount { new_price } => {
                let uses = limit.min(units);
                let price = Decimal::from(uses) * (new_price - price_per_unit)
                    + price_per_unit * Decimal::from(units);
                (price, uses)
            },
            DealImpl::Batch { take, pay_for } => {
                let uses = limit.min(units / take);
                let price =
                    price_per_unit * Decimal::from(units - uses * (take.get() - pay_for.get()));
                (price, uses)
            },
            DealImpl::BatchPrice { take, pay } => {
                let uses = limit.min(units / take);
                let price = pay * Decimal::from(uses)
                    + price_per_unit * Decimal::from(units - take.get() * uses);
                (price, uses)
            },
        }
    }
}

/// Backing implementation of [`Deal`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum DealImpl {
    /// The price has been reduced.
    Discount {
        /// The new price of the product.
        new_price: Decimal,
    },
    /// A "take N pay for M" deal.
    Batch {
        /// N; how many products to take.
        take: NonZeroU32,
        /// M; how many products to pay for.
        pay_for: NonZeroU32,
    },
    /// A "take N pay X" deal.
    BatchPrice {
        /// N; how many products to take.
        take: NonZeroU32,
        /// X; how much to pay.
        pay: Decimal,
    },
}

/// Errors created by methods of [`Deal`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum DealError {
    /// The provided base price was 0.
    #[error("Base price must be positive.")]
    ZeroPrice,
    /// The new price or a quantity was non-positive. The new price may be 0 only if both
    /// quantities are [`None`].
    #[error("New price and quantities must be positive.")]
    OutOfRange,
    #[error("Invalid type of special offer.")]
    /// The arguments did not represent a valid type of special offer.
    InvalidVariant,
    #[error("Special offer does not provide discount.")]
    /// The deal did not result in a discount.
    /// expected.
    NoDiscount,
}

/// A vote on a review or comment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(Type))]
pub enum Vote {
    /// The user disliked the review/comment. Counts as -1 for tallying.
    Dislike,
    /// The user liked the review/comment. Counts as 1 for tallying.
    Like,
}

/// The role of a user.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(Type))]
pub enum Role {
    /// The author is a customer.
    Customer,
    /// The author is a vendor.
    Vendor,
    /// The user is a site administrator.
    Administrator,
}
