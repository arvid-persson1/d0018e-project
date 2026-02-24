//! Types produced by database operations.

use crate::database::{Id, Product};
use derive_more::{Deref, Display, Into};
use regex::Regex;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use sqlx::Type;
use std::{
    cmp::Ordering,
    fmt::{Display, Error as FmtError, Formatter},
    num::NonZeroU8,
    sync::LazyLock,
};
use thiserror::Error;
use time::PrimitiveDateTime;

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
    /// Compares the amounts if their units are equal, otherwise returns `None`.
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
pub struct Rating(NonZeroU8);

impl Rating {
    /// Verifies the rating is in range and constructs a `Rating` on success.
    #[must_use]
    #[expect(clippy::missing_panics_doc, reason = "See note.")]
    pub fn new(r: u8) -> Option<Self> {
        (1..=5).contains(&r).then(|| {
            #[expect(clippy::unwrap_used, reason = "Just verified.")]
            Self(NonZeroU8::new(r).unwrap())
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
    /// Compares the ratings, ignoring the counts.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AverageRating {
    /// Compares the ratings, ignoring the counts.
    fn cmp(&self, other: &Self) -> Ordering {
        let Self { rating, count: _ } = self;
        #[expect(clippy::unwrap_used, reason = "The rating is known to be real.")]
        rating.partial_cmp(&other.rating).unwrap()
    }
}

impl PartialEq for AverageRating {
    /// Compares the ratings, ignoring the counts.
    fn eq(&self, other: &Self) -> bool {
        let Self { rating, count: _ } = self;
        // The rating is known to be real.
        *rating == other.rating
    }
}

impl Eq for AverageRating {}

impl AverageRating {
    /// Verifies the rating is in range and constructs an `AverageRating` on success.
    // TODO: Add infallible variant based on `Rating`.
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

    /// Returns the average rating if there are any.
    #[must_use]
    pub const fn rating(self) -> Option<f64> {
        if self.count > 0 {
            Some(self.rating)
        } else {
            None
        }
    }

    /// Returns the number of ratings.
    #[must_use]
    pub const fn count(self) -> u64 {
        self.count
    }
}

impl Display for AverageRating {
    /// Formats the rating with a single decimal point.
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
pub struct Username(Box<str>);

impl Username {
    /// Verifies the format and constructs a `Username` on success.
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
pub struct Email(Box<str>);

impl Email {
    /// Verifies the format and constructs an `Email` on success.
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
/// This type is unaware of what product it belongs to or its pricing, and is therefore unable to
/// verify that it actually provides a discount. Nevertheless, attempting to insert such a special
/// offer into the database will result in an error.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Deal {
    /// The price has been reduced.
    Discount {
        /// The new price of the product.
        new_price: Decimal,
    },
    /// A "take N pay for M" deal.
    Batch {
        /// N; how many products to take.
        take: u32,
        /// M; how many products to pay for.
        pay_for: u32,
    },
    /// A "take N pay X" deal.
    BatchPrice {
        /// N; how many products to take.
        take: u32,
        /// X; how much to pay.
        pay: Decimal,
    },
}

/// Errors created by [`Deal::new`].
// NOTE: Defensively not `Eq`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum DealError {
    #[error("Invalid type of special offer.")]
    /// The arguments did not represent a valid type of special offer.
    InvalidVariant,
    #[error("Special offer does not provide discount.")]
    /// The deal did not actually provide a discount. The deal is given anyway in case this was
    /// expected.
    NoDiscount(Deal),
}

impl Deal {
    /// Constructs a new `Deal` from the format used in the database.
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
    pub fn new(
        new_price: Option<Decimal>,
        quantity1: Option<i32>,
        quantity2: Option<i32>,
        base_price: Decimal,
    ) -> Result<Self, DealError> {
        let deal = Self::try_new(new_price, quantity1, quantity2, base_price)?;
        deal.ok_or(DealError::InvalidVariant)
    }

    /// Constructs a new `Deal` from the format used in the database.
    ///
    /// Unlike [`new`], this returns an <code>[Option]\<Deal\></code>, allowing all fields to be
    /// [`None`] and then producing [`None`]. See [`new`] for details.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidVariant`](DealError::InvalidVariant) if none of the above cases
    /// match, or [`NoDiscount`](DealError::NoDiscount) if the deal is structurally valid
    /// but does not actually provide a discount compared to the price of the product.
    ///
    /// Silently overflows if either quantity is [`Some`] and negative.
    ///
    /// [`new`]: Self::new
    pub fn try_new(
        new_price: Option<Decimal>,
        quantity1: Option<i32>,
        quantity2: Option<i32>,
        base_price: Decimal,
    ) -> Result<Option<Self>, DealError> {
        let deal = match (new_price, quantity1, quantity2) {
            (Some(new_price), None, None) => Self::Discount { new_price },
            (None, Some(take), Some(pay_for)) => Self::Batch {
                take: take as u32,
                pay_for: pay_for as u32,
            },
            (Some(pay), Some(take), None) => Self::BatchPrice {
                pay,
                take: take as u32,
            },
            (None, None, None) => return Ok(None),
            _ => return Err(DealError::InvalidVariant),
        };
        if deal.discount_average(base_price).is_some() {
            Ok(Some(deal))
        } else {
            Err(DealError::NoDiscount(deal))
        }
    }

    /// Converts a `Deal` into the format used in the database.
    ///
    /// Specifically, this returns a tuple representing the columns  `new_price`, `quantity1` and
    /// `quantity2` respectively on success. If either quantity is greater than `i32::MAX`, `None`
    /// is returned.
    #[must_use]
    pub fn database_repr(self) -> Option<(Option<Decimal>, Option<i32>, Option<i32>)> {
        match self {
            Self::Discount { new_price } => Some((Some(new_price), None, None)),
            Self::Batch { take, pay_for }
                if let Ok(take) = take.try_into()
                    && let Ok(pay_for) = pay_for.try_into() =>
            {
                Some((None, Some(take), Some(pay_for)))
            },
            Self::BatchPrice { take, pay } if let Ok(take) = take.try_into() => {
                Some((Some(pay), Some(take), None))
            },
            Self::Batch { .. } | Self::BatchPrice { .. } => None,
        }
    }

    /// Calculates the discount in percent as average per unit. If the deal doesn't actually offer
    /// a price reduction, `None` is returned.
    #[must_use]
    pub fn discount_average(self, base_price: Decimal) -> Option<Decimal> {
        match self {
            Self::Discount { new_price } if new_price < base_price => {
                Some(Decimal::ONE - new_price / base_price)
            },
            Self::Batch { take, pay_for } if take > pay_for => {
                Some(Decimal::ONE - Decimal::from(pay_for) / Decimal::from(take))
            },
            Self::BatchPrice { take, pay }
                if take > 1
                    && let take = Decimal::from(take)
                    && pay * take < base_price =>
            {
                Some(Decimal::ONE - pay / (base_price * take))
            },
            Self::Discount { .. } | Self::Batch { .. } | Self::BatchPrice { .. } => None,
        }
    }
}

/// A record of a customer's review, for display on profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomerReview {
    /// The ID of the product.
    pub product: Id<Product>,
    /// URL to an image of the product.
    pub thumbnail: Url,
    /// The name of the product.
    pub product_name: Box<str>,
    /// The given rating of the product.
    pub rating: Rating,
    /// The title of the review.
    pub title: Box<str>,
    /// The content of the review.
    pub content: Box<str>,
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

/// A record of a customer's purchase.
///
/// This does not include a timestamp, as they are intended to be grouped by timestamp in an
/// [`Order`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Purchase {
    /// How much was paid.
    pub paid: Decimal,
    /// Whether a special offer affected the price.
    pub special_offer_used: bool,
    /// How much of the product was included in one unit at the time of purchase.
    pub amount_per_unit: Amount,
    /// How many units were purchased.
    pub number: u32,
    /// The name of the product.
    pub product_name: Box<str>,
    /// URL to an image of the product.
    pub thumbnail: Url,
}

/// A completed order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Order {
    /// The time of purchase.
    pub time: PrimitiveDateTime,
    /// Purchases included in this order.
    pub purchases: Box<[Purchase]>,
}

impl Order {
    /// Calculcates the total price of all purchases in the order.
    #[must_use]
    pub fn price(&self) -> Decimal {
        self.purchases.iter().map(|purchase| purchase.paid).sum()
    }
}
