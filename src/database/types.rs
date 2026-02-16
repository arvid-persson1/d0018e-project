//! Types produced by database operations.

use super::{Category, Comment, Id, Product, Review, Vendor};
use chrono::NaiveDateTime;
use derive_more::{Deref, Display, Into};
use regex::Regex;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use sqlx::Type;
use std::{
    fmt::{Display, Error as FmtError, Formatter},
    sync::LazyLock,
};

/// URL to an external resource, owned.
pub type Url = Box<str>;

/// Quantity of a product along with unit, e.g. "4.2 kg" or "8.15 dl".
///
/// This type is oblivious to any actual meaning behind the units, so it can't for example handle
/// conversions.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(Type))]
pub struct Amount {
    /// The quantity.
    pub quantity: Decimal,
    /// The unit.
    pub unit: Box<str>,
}

impl Display for Amount {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        let Self { quantity, unit } = self;
        write!(f, "{quantity:.2} {unit}")
    }
}

/// The rating of a product. Will be between 1 and 5.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Into, Deref, Serialize, Deserialize,
)]
#[expect(clippy::unsafe_derive_deserialize, reason = "TODO")]
// TODO: Derive `Deserialize` manually, disallowing out-of-range values.
pub struct Rating(u8);

impl Rating {
    /// Verifies the rating is in range and constructs a `Rating` on success.
    #[inline]
    pub fn new(r: u8) -> Option<Self> {
        Some(r).filter(|r| (1..=5).contains(r)).map(Self)
    }

    /// Constructs a `Rating` without verifying range.
    ///
    /// # Safety
    ///
    /// Must ensure `1 <= r <= 5`.
    #[inline]
    #[must_use]
    pub const unsafe fn new_unchecked(r: u8) -> Self {
        Self(r)
    }
}

/// The average rating of a product (between 1 and 5), as well as the number of ratings
/// contributing to that score.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[expect(clippy::unsafe_derive_deserialize, reason = "TODO")]
// TODO: Derive `Deserialize` manually, disallowing out-of-range values.
pub struct AverageRating {
    /// The average rating, unspecified if `count` is 0.
    rating: f32,
    /// The number of ratings contributing to the score.
    count: u32,
}

impl AverageRating {
    /// Verifies the rating is in range and constructs an `AverageRating` on success.
    #[inline]
    #[must_use]
    pub fn new(rating: f32, count: u32) -> Option<Self> {
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

    /// Constructs an `AverageRating` without verifying range.
    ///
    /// # Safety
    ///
    /// Must ensure `1 <= r <= 5`.
    #[inline]
    #[must_use]
    pub const unsafe fn new_unchecked(rating: f32, count: u32) -> Self {
        Self { rating, count }
    }

    /// Returns the average rating if there are any.
    #[inline]
    #[must_use]
    pub const fn rating(self) -> Option<f32> {
        if self.count > 0 {
            Some(self.rating)
        } else {
            None
        }
    }

    /// Returns the number of ratings.
    #[inline]
    #[must_use]
    pub const fn count(self) -> u32 {
        self.count
    }
}

impl Display for AverageRating {
    /// Formats the rating with a single decimal point.
    #[inline]
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
    Debug,
    Default,
    Display,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Into,
    Deref,
    Serialize,
    Deserialize,
)]
#[expect(clippy::unsafe_derive_deserialize, reason = "TODO")]
// TODO: Derive `Deserialize` manually, disallowing invalid values.
pub struct Username(Box<str>);

impl Username {
    /// Verifies the format and constructs a `Username` on success.
    #[inline]
    pub fn new(s: Box<str>) -> Option<Self> {
        static USERNAME_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"^[\w-]{3,20}$").expect("Failed to compile regex pattern.")
        });

        Some(s).filter(|s| USERNAME_PATTERN.is_match(s)).map(Self)
    }

    /// Constructs a `Username` without verifying format.
    ///
    /// # Safety
    ///
    /// Must ensure the string fits the format, see [type documentation](Self).
    #[inline]
    #[must_use]
    pub const unsafe fn new_unchecked(s: Box<str>) -> Self {
        Self(s)
    }
}

/// An overview of a product, for display on product cards.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductOverview {
    /// The ID of the product.
    pub id: Id<Product>,
    /// The name of the product.
    pub name: Box<str>,
    /// URL to an image to display on the product card.
    pub thumbnail: Url,
    /// The price of the product before any discounts.
    pub price: Decimal,
    /// A short description of the proudct,
    pub overview: Box<str>,
    /// How many units are in stock. This should not be displayed on the card directly, but may
    /// be used to display "low stock".
    pub in_stock: i32,
    /// How much of the product is included in one unit.
    pub amount_per_unit: Option<Amount>,
    /// The name of the vendor.
    pub vendor_name: Box<str>,
    /// The origin of the product. This may or may not be the name of a country.
    pub origin: Box<str>,
    /// The currently active special offer on the product, if any. This is a tuple
    /// `(deal, members_only)` where `members_only` indicates whether the special offer is
    /// available only to members.
    pub special_offer: Option<(Deal, bool)>,
}

/// Information about a product, for display on product pages.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProductInfo {
    /// The ID of the product.
    pub id: Id<Product>,
    /// The name of the product.
    pub name: Box<str>,
    /// URLs to images of the product.
    pub gallery: Box<[Url]>,
    /// The price of the product before any discounts.
    pub price: Decimal,
    /// A long description of the product.
    pub description: Box<str>,
    /// How many units are in stock. This should not be displayed on the page directly, but may
    /// be used to display "low stock".
    pub in_stock: u32,
    /// The category of the product and all of its parents, starting from the root.
    pub category: CategoryPath,
    /// How much of the product is included in one unit.
    pub amount_per_unit: Option<Amount>,
    /// Whether the product is visible to customers. Administrators should be able to see all
    /// products, and vendors should be able to see their own even if they are hidden.
    pub visible: bool,
    /// The ID of the vendor.
    pub vendor_id: Id<Vendor>,
    /// The name of the vendor.
    pub vendor_name: Box<str>,
    /// The origin of the product. This may or may not be the name of a country.
    pub origin: Box<str>,
    /// When the product was created. This refers to the entry for the product in the system, not
    /// the date any unit was manufactured.
    pub created_at: NaiveDateTime,
    /// When the product was last updated.
    pub updated_at: NaiveDateTime,
    /// The average rating of the product.
    pub rating: AverageRating,
    /// The currently active special offer on the product, if any. This is a tuple
    /// `(deal, members_only)` where `members_only` indicates whether the special offer is
    /// available only to members.
    pub special_offer: Option<(Deal, bool)>,
}

/// Details on the discount of a special offer.
///
/// This type is unaware of what product it belongs to or its pricing, and is therefore unable to
/// verify that it actually provides a discount. Nevertheless, attempting to insert such a special
/// offer into the database will result in an error.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

/// Errors created by [`SpecialOfferDeal::new`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpecialOfferDealError {
    /// The arguments did not represent a valid type of special offer.
    InvalidVariant,
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
    /// Returns [`InvalidVariant`](SpecialOfferDeal::InvalidVariant) if none of the above cases
    /// match, or [`NoDiscount`](SpecialOfferDeal::NoDiscount) if the deal is structurally valid
    /// but does not actually provide a discount compared to the price of the product.
    ///
    /// Silently overflows if either quantity is [`Some`] and negative.
    #[inline]
    pub fn new(
        new_price: Option<Decimal>,
        quantity1: Option<i32>,
        quantity2: Option<i32>,
        base_price: Decimal,
    ) -> Result<Option<Self>, SpecialOfferDealError> {
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
            _ => return Err(SpecialOfferDealError::InvalidVariant),
        };
        if deal.discount_average(base_price).is_some() {
            Ok(Some(deal))
        } else {
            Err(SpecialOfferDealError::NoDiscount(deal))
        }
    }

    /// Converts a `Deal` into the format used in the database.
    ///
    /// Specifically, this returns a tuple representing the columns  `new_price`, `quantity1` and
    /// `quantity2` respectively on success. If either quantity is greater than `i32::MAX`, `None`
    /// is returned.
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
    #[inline]
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

/// A category with its subcategories, for display in a tree.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategoryTree {
    /// The ID of the category.
    pub id: Id<Category>,
    /// The name of the category.
    pub name: Box<str>,
    /// All direct subcategories.
    pub subcategories: Vec<Self>,
}

/// A category with its supercategories, for display on product pages, starting from the root.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CategoryPath {
    /// The segments of the path. Each item is a tuple `(id, name)`.
    pub segments: Box<[(Id<Category>, Box<str>)]>,
}

/// A customer's own review of a product, for display on product pages.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnReview {
    /// The ID of the review.
    pub id: Id<Review>,
    /// The given rating of the product.
    pub rating: Rating,
    /// When the review was created.
    pub created_at: NaiveDateTime,
    /// When the review was last updated.
    pub updated_at: NaiveDateTime,
    /// The title of the review.
    pub title: Box<str>,
    /// The content of the review.
    pub content: Box<str>,
    /// Comment trees on the review.
    pub comments: Box<[CommentTree]>,
    /// The sum of all votes on the review, adding 1 per like and subtracting 1 per dislike.
    pub sum_votes: i32,
}

/// A review of a product, for display on product pages.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductReview {
    /// The ID of the review.
    pub id: Id<Review>,
    /// The username of the authoring customer.
    pub username: Username,
    /// The profile picture of the authoring customer.
    pub profile_picture: Url,
    /// The given rating of the product.
    pub rating: Rating,
    /// When the review was created.
    pub created_at: NaiveDateTime,
    /// When the review was last updated.
    pub updated_at: NaiveDateTime,
    /// The title of the review.
    pub title: Box<str>,
    /// The content of the review.
    pub content: Box<str>,
    /// Comment trees on the review.
    pub comments: Box<[CommentTree]>,
    /// The sum of all votes on the review, adding 1 per like and subtracting 1 per dislike.
    pub sum_votes: i32,
    /// The customer's own vote, if any.
    pub own_vote: Option<Vote>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// A vote on a review or comment.
pub enum Vote {
    /// The user liked the review/comment. Counts as 1 for tallying.
    Like,
    /// The user disliked the review/comment. Counts as -1 for tallying.
    Dislike,
}

/// A comment with its replies, for display in a tree.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommentTree {
    /// The ID of the comment.
    pub id: Id<Comment>,
    /// The username of the author.
    pub username: Username,
    /// The role of the author. See [`CommentRole`] for details.
    pub user_role: CommentRole,
    /// The content of the comment.
    pub content: Box<str>,
    /// When the comment was created.
    pub created_at: NaiveDateTime,
    /// When the comment was last updated.
    pub updated_at: NaiveDateTime,
    /// The sum of all votes on the review, adding 1 per like and subtracting 1 per dislike.
    pub sum_votes: i32,
    /// The customer's own vote, if any.
    pub own_vote: Option<Vote>,
    /// All direct replies to the comment.
    pub replies: Box<[Self]>,
}

/// The role of the user placing a comment. In some cases, a special badge should be displayed by
/// the comment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[expect(variant_size_differences, reason = "Difference is neglibile.")]
pub enum CommentRole {
    /// The author is a user. The original poster of the review should get a badge.
    User {
        /// Whether the user was the orignal poster of the review.
        original_poster: bool,
    },
    /// The author is a vendor. The vendor of the reviewed product should get a badge.
    Vendor {
        /// The ID of the vendor, to be compared with the product's vendor ID.
        id: Id<Vendor>,
    },
    /// The user is a site administrator. Administrators should always get a badge.
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
    // TODO: Some of these fields might not be used in the frontend and should then be removed.
    /// A short description of the proudct,
    pub product_overview: Box<str>,
    /// The origin of the product. This may or may not be the name of a country.
    pub product_origin: Box<str>,
    /// The name of the vendor.
    pub vendor_name: Box<str>,
}

/// A completed order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Order {
    /// The time of purchase.
    pub time: NaiveDateTime,
    /// Purchases included in this order.
    pub purchases: Box<[Purchase]>,
}

impl Order {
    /// Calculcates the total price of all purchases in the order.
    pub fn price(&self) -> Decimal {
        self.purchases.iter().map(|purchase| purchase.paid).sum()
    }
}
