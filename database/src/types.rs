//! Types produced by database operations.

use derive_more::{Deref, Display, Into};
use num_traits::cast::ToPrimitive as _;
use sqlx::types::{Decimal, chrono::NaiveDateTime};
use std::{
    fmt::{Display, Error as FmtError, Formatter},
    num::NonZeroU32,
};

use crate::id::{Category, Comment, Id, Product, Review, Vendor};

/// URL to an external resource, owned.
pub type Url = Box<str>;
/// URL to an external resource, borrowed.
pub type UrlRef<'a> = &'a str;

/// Quantity of a product along with unit, e.g. "4.2 kg" or "8.15 dl".
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Into, Deref)]
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

/// The average rating of a product. Will be between 1 and 5.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Into, Deref)]
pub struct AverageRating(f32);

impl AverageRating {
    /// Verifies the rating is in range and constructs an `AverageRating` on success.
    #[inline]
    pub fn new(r: f32) -> Option<Self> {
        Some(r).filter(|r| (1. ..=5.).contains(r)).map(Self)
    }

    /// Constructs an `AverageRating` without verifying range.
    ///
    /// # Safety
    ///
    /// Must ensure `1 <= r <= 5`.
    #[inline]
    #[must_use]
    pub const unsafe fn new_unchecked(r: f32) -> Self {
        Self(r)
    }
}

impl Display for AverageRating {
    /// Formats the rating with a single decimal point.
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        let Self(r) = self;
        write!(f, "{r:.1}")
    }
}

// NOTE: No `NonFutureTimestamp` or `NonFutureDate` is implemented here as "proper" usage could
// cause creating a valid instance, but not consuming it until the time has passed and the instance
// is invalidated. Hence, this validation is done only in the database.

/// A string known to be at most `N` chars (not bytes).
#[derive(Debug, Default, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Into, Deref)]
pub struct BoundedString<const N: usize>(Box<str>);

impl<const N: usize> BoundedString<N> {
    /// Verifies the character count in the string and constructs a `BoundedString` on success.
    #[inline]
    pub fn new(s: Box<str>) -> Option<Self> {
        Some(s).filter(|s| s.chars().count() <= N).map(Self)
    }

    /// Constructs a `BoundedString` without verifying character count.
    ///
    /// # Safety
    ///
    /// Must ensure `s.chars().count() <= N`.
    #[inline]
    #[must_use]
    pub const unsafe fn new_unchecked(s: Box<str>) -> Self {
        Self(s)
    }
}

/// An overview of a product, for display on product cards.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
    /// The name of the vendor. This is the potentially long display name, so it might have to
    /// be truncated.
    pub vendor_name: Box<str>,
    /// The origin of the product. This may or may not be the name of a country.
    pub origin: Box<str>,
    /// The currently active special offer on the product, if any.
    pub special_offer: Option<ProductSpecialOffer>,
}

/// Information about a product, for display on product pages.
#[derive(Clone, Debug, PartialEq)]
pub struct ProductInfo {
    /// The ID of the product.
    pub id: Id<Product>,
    /// The name of the product.
    pub name: Box<str>,
    /// URLs to images of the product.
    pub gallery: Box<[Url]>,
    /// URL to the image meant to be displayed on product cards. This image should not be displayed
    /// with those from [`gallery`](Self::gallery), but can be used as a fallback if the gallery is
    /// empty.
    pub thumbnail: Url,
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
    /// The currently active special offer on the product, if any.
    pub special_offer: Option<ProductSpecialOffer>,
    /// The customer's own review of the product, if any.
    pub own_review: Option<OwnReview>,
}

/// A special offer, offering some sort of discount on a product.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProductSpecialOffer {
    /// Whether the special offer is available only for members.
    pub members_only: bool,
    /// The limit of how many times each customer can get a discount from this offer, if any.
    pub limit_per_customer: Option<NonZeroU32>,
    /// End of the special offer, if any. Note that the start time is omitted, but is implied to be
    /// some time before now since the special offer was fetched.
    pub valid_until: Option<NaiveDateTime>,
    /// What the offer entails.
    pub deal: SpecialOfferDeal,
}

/// Details on the discount of a special offer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SpecialOfferDeal {
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

impl SpecialOfferDeal {
    /// Calculates the discount in percent as average per unit. If the deal doesn't actually offer
    /// a price reduction, `None` is returned.
    #[inline]
    #[must_use]
    pub fn discount_average(self, base_price: Decimal) -> Option<f32> {
        match self {
            Self::Discount { new_price }
                if new_price < base_price
                    && let Some(factor) = (new_price / base_price).to_f32() =>
            {
                Some(1. - factor)
            },
            Self::Batch { take, pay_for } if take > pay_for => {
                Some(1. - pay_for as f32 / take as f32)
            },
            Self::BatchPrice { take, pay }
                if take > 1
                    && let Some(pay) = pay.to_f32()
                    && let Some(base_price) = base_price.to_f32()
                    && pay * (take as f32) < base_price =>
            {
                Some(1. - pay / (base_price * take as f32))
            },
            Self::Discount { .. } | Self::Batch { .. } | Self::BatchPrice { .. } => None,
        }
    }
}

/// A category with its subcategories, for display in a tree.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CategoryTree {
    /// The ID of the category.
    pub id: Id<Category>,
    /// The name of the category.
    pub name: Box<str>,
    /// All direct subcategories.
    pub subcategories: Box<[Self]>,
}

/// A category with its supercategories, for display on product pages, starting from the root.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CategoryPath {
    /// The segments of the path. Each item is a tuple `(id, name)`.
    pub segments: Box<[(Id<Category>, Box<str>)]>,
}

/// A customer's own review of a product, for display on product pages.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnReview {
    /// The ID of the review.
    pub id: Id<Review>,
    /// The given rating of the product.
    pub rating: Rating,
    /// When the review was created.
    pub created_at: NaiveDateTime,
    /// When the review was last updated.
    pub updated_at: NaiveDateTime,
    /// The title and content of the review, if any.
    pub title_and_content: Option<(Box<str>, Box<str>)>,
    /// Comment trees on the review.
    pub comments: Box<[CommentTree]>,
    /// The sum of all votes on the review, adding 1 per like and subtracting 1 per dislike.
    pub sum_votes: i32,
}

/// A review of a product, for display on product pages.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProductReview {
    /// The ID of the review.
    pub id: Id<Review>,
    /// The username of the authoring customer.
    pub username: BoundedString<20>,
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReviewLog {
    /// The ID of the product.
    pub product: Id<Product>,
    /// URL to an image of the product.
    pub thumbnail: Url,
    /// The name of the product.
    pub product_name: Box<str>,
    /// The given rating of the product.
    pub rating: Rating,
    /// The title and content of the review, if any.
    pub title_and_content: Option<(Box<str>, Box<str>)>,
    /// The sum of all votes on the review, adding 1 per like and subtracting 1 per dislike.
    pub sum_votes: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// A vote on a review or comment.
pub enum Vote {
    /// The user liked the review/comment. Counts as 1 for tallying.
    Like,
    /// The user disliked the review/comment. Counts as -1 for tallying.
    Dislike,
}

/// A comment with its replies, for display in a tree.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CommentTree {
    /// The ID of the comment.
    pub id: Id<Comment>,
    /// The username of the author.
    pub username: BoundedString<20>,
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
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

/// A completed order.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Order {
    /// Overview of the product.
    product: ProductOverview,
    /// Total price of the order (not per product) at the time of purchase.
    price: Decimal,
    /// Amount per unit at the time of purchase.
    amount_per_unit: Option<Amount>,
    /// Number of units in the order.
    count: u32,
}
