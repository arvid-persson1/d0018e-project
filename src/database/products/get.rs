//! Database functions for getting product overviews to be displayed on product cards.

use crate::database::{Amount, Category, Customer, Deal, Id, Product, Url, Vendor};
use dioxus::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
#[cfg(feature = "server")]
use {
    crate::database::{
        RawId, connection,
        products::{build_amount, try_build_special_offer},
    },
    sqlx::query_as,
    std::cmp::Reverse,
};

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
    pub in_stock: NonZeroU32,
    /// How much of the product is included in one unit.
    pub amount_per_unit: Amount,
    /// The name of the vendor.
    pub vendor_name: Box<str>,
    /// The origin of the product. This may or may not be the name of a country.
    pub origin: Box<str>,
    /// The currently active special offer on the product if any, and whether it only applies to
    /// members.
    pub special_offer: Option<(Deal, bool)>,
    /// Whether the customer has marked the product as a favorite. Value is unspecified if a
    /// customer ID was not provided.
    pub favorited: bool,
}

/// An overview of a product with an active special offer, for display on product cards.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductOverviewDiscounted {
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
    pub in_stock: NonZeroU32,
    /// How much of the product is included in one unit.
    pub amount_per_unit: Amount,
    /// The name of the vendor.
    pub vendor_name: Box<str>,
    /// The origin of the product. This may or may not be the name of a country.
    pub origin: Box<str>,
    /// The currently active special offer on the product.
    pub special_offer_deal: Deal,
    /// Whether the special offer only applies to members.
    pub special_offer_members_only: bool,
    /// Whether the customer has marked the product as a favorite. Value is unspecified if a
    /// customer ID was not provided.
    pub favorited: bool,
}

/// An overview of a product owned by a known vendor, for display on product cards.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductOverviewVendor {
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
    pub in_stock: u32,
    /// How much of the product is included in one unit.
    pub amount_per_unit: Amount,
    /// The origin of the product. This may or may not be the name of a country.
    pub origin: Box<str>,
    /// The currently active special offer on the product if any, and whether it only applies to
    /// members.
    pub special_offer: Option<(Deal, bool)>,
    /// Whether the customer has marked the product as a favorite. Value is unspecified if a
    /// customer ID was not provided.
    pub favorited: bool,
}

/// An overview of a customer's favorite product, for display on product cards.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductOverviewFavorited {
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
    pub in_stock: u32,
    /// How much of the product is included in one unit.
    pub amount_per_unit: Amount,
    /// The name of the vendor.
    pub vendor_name: Box<str>,
    /// The origin of the product. This may or may not be the name of a country.
    pub origin: Box<str>,
    /// The currently active special offer on the product if any, and whether it only applies to
    /// members.
    pub special_offer: Option<(Deal, bool)>,
}

#[cfg(feature = "server")]
struct ProductRepr {
    id: RawId,
    name: String,
    thumbnail: String,
    price: Decimal,
    overview: String,
    in_stock: i32,
    origin: String,
    amount_per_unit: Decimal,
    measurement_unit: Option<String>,
    new_price: Option<Decimal>,
    quantity1: Option<i32>,
    quantity2: Option<i32>,
    members_only: Option<bool>,
    vendor_name: String,
    favorited: bool,
}

#[cfg(feature = "server")]
impl From<ProductRepr> for ProductOverview {
    fn from(
        ProductRepr {
            id,
            name,
            thumbnail,
            price,
            overview,
            in_stock,
            origin,
            amount_per_unit,
            measurement_unit,
            new_price,
            quantity1,
            quantity2,
            members_only,
            vendor_name,
            favorited,
        }: ProductRepr,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            thumbnail: thumbnail.into(),
            price,
            overview: overview.into(),
            in_stock: u32::try_from(in_stock)
                .expect("Database returned negative stock.")
                .try_into()
                .expect("Database returnd product with no stock."),
            amount_per_unit: build_amount(amount_per_unit, measurement_unit),
            vendor_name: vendor_name.into(),
            origin: origin.into(),
            special_offer: try_build_special_offer(
                new_price,
                quantity1,
                quantity2,
                members_only,
                price,
            ),
            favorited,
        }
    }
}

#[cfg(feature = "server")]
struct ProductReprDiscounted {
    id: RawId,
    name: String,
    thumbnail: String,
    price: Decimal,
    overview: String,
    in_stock: i32,
    origin: String,
    amount_per_unit: Decimal,
    measurement_unit: Option<String>,
    new_price: Option<Decimal>,
    quantity1: Option<i32>,
    quantity2: Option<i32>,
    members_only: bool,
    vendor_name: String,
    favorited: bool,
}

#[cfg(feature = "server")]
impl From<ProductReprDiscounted> for ProductOverviewDiscounted {
    fn from(
        ProductReprDiscounted {
            id,
            name,
            thumbnail,
            price,
            overview,
            in_stock,
            origin,
            amount_per_unit,
            measurement_unit,
            new_price,
            quantity1,
            quantity2,
            members_only,
            vendor_name,
            favorited,
        }: ProductReprDiscounted,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            thumbnail: thumbnail.into(),
            price,
            overview: overview.into(),
            in_stock: u32::try_from(in_stock)
                .expect("Database returned negative stock.")
                .try_into()
                .expect("Database returnd product with no stock."),
            amount_per_unit: build_amount(amount_per_unit, measurement_unit),
            vendor_name: vendor_name.into(),
            origin: origin.into(),
            special_offer_deal: Deal::new(new_price, quantity1, quantity2, price)
                .expect("Database returned invalid special offer."),
            special_offer_members_only: members_only,
            favorited,
        }
    }
}

#[cfg(feature = "server")]
struct ProductReprVendor {
    id: RawId,
    name: String,
    thumbnail: String,
    price: Decimal,
    overview: String,
    in_stock: i32,
    origin: String,
    amount_per_unit: Decimal,
    measurement_unit: Option<String>,
    new_price: Option<Decimal>,
    quantity1: Option<i32>,
    quantity2: Option<i32>,
    members_only: Option<bool>,
    favorited: bool,
}

#[cfg(feature = "server")]
impl From<ProductReprVendor> for ProductOverviewVendor {
    fn from(
        ProductReprVendor {
            id,
            name,
            thumbnail,
            price,
            overview,
            in_stock,
            origin,
            amount_per_unit,
            measurement_unit,
            new_price,
            quantity1,
            quantity2,
            members_only,
            favorited,
        }: ProductReprVendor,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            thumbnail: thumbnail.into(),
            price,
            overview: overview.into(),
            in_stock: in_stock
                .try_into()
                .expect("Database returned negative stock."),
            amount_per_unit: build_amount(amount_per_unit, measurement_unit),
            origin: origin.into(),
            special_offer: try_build_special_offer(
                new_price,
                quantity1,
                quantity2,
                members_only,
                price,
            ),
            favorited,
        }
    }
}

#[cfg(feature = "server")]
struct ProductReprFavorited {
    id: RawId,
    name: String,
    thumbnail: String,
    price: Decimal,
    overview: String,
    in_stock: i32,
    origin: String,
    amount_per_unit: Decimal,
    measurement_unit: Option<String>,
    new_price: Option<Decimal>,
    quantity1: Option<i32>,
    quantity2: Option<i32>,
    members_only: Option<bool>,
    vendor_name: String,
}

#[cfg(feature = "server")]
impl From<ProductReprFavorited> for ProductOverviewFavorited {
    fn from(
        ProductReprFavorited {
            id,
            name,
            thumbnail,
            price,
            overview,
            in_stock,
            origin,
            amount_per_unit,
            measurement_unit,
            new_price,
            quantity1,
            quantity2,
            members_only,
            vendor_name,
        }: ProductReprFavorited,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            thumbnail: thumbnail.into(),
            price,
            overview: overview.into(),
            in_stock: in_stock
                .try_into()
                .expect("Database returned negative stock."),
            amount_per_unit: build_amount(amount_per_unit, measurement_unit),
            vendor_name: vendor_name.into(),
            origin: origin.into(),
            special_offer: try_build_special_offer(
                new_price,
                quantity1,
                quantity2,
                members_only,
                price,
            ),
        }
    }
}

/// Get the most recently created products.
///
/// Only visible products with units in stock are considered.
///
/// # Errors
///
/// Fails if:
/// - `limit > i64::MAX`.
/// - `offset > i64::MAX`.
/// - An error occurs during communication with the database.
#[server]
pub async fn newest_products(
    customer: Option<Id<Customer>>,
    limit: usize,
    offset: usize,
) -> Result<Box<[ProductOverview]>> {
    query_as!(
        ProductRepr,
        r#"
        SELECT p.id, name, thumbnail, price, overview, in_stock, origin, amount_per_unit, measurement_unit,
            aso.new_price, aso.quantity1, aso.quantity2, aso.members_only,
            vendors.display_name AS vendor_name,
            EXISTS(
                SELECT 1
                FROM customer_favorites cf
                WHERE cf.customer = $1 AND cf.product = p.id
            ) AS "favorited!"
        FROM products p
        LEFT JOIN active_special_offers aso ON aso.product = p.id
        JOIN vendors ON vendors.id = p.vendor
        WHERE visible AND in_stock > 0
        ORDER BY created_at DESC
        LIMIT $2
        OFFSET $3
        "#,
        customer.map(Id::get),
        i64::try_from(limit)?,
        i64::try_from(offset)?,
    )
    .fetch_all(connection())
    .await
    .map(|products| products.into_iter().map(Into::into).collect())
    .map_err(Into::into)
}

// PERF: Discount-based sorting of products is currently not supported by an index. If the
// performance hit is significant, an `active_discount` column should be added to `products` and
// updated using triggers. However, this does require considering time factors since discounts
// change "on their own" due to special offers running expiring.

/// Get other products in the same category as a given one sorted by best discounts, as defined by
/// [`discount_average`](Deal::discount_average).
///
/// Only visible products with units in stock are considered.
///
/// # Errors
///
/// Fails if:
/// - `limit > i64::MAX`.
/// - `offset > i64::MAX`.
/// - An error occurs during communication with the database.
#[server]
pub async fn similar_products(
    customer: Option<Id<Customer>>,
    category: Id<Category>,
    except: Id<Product>,
    limit: usize,
    offset: usize,
) -> Result<Box<[ProductOverview]>> {
    query_as!(
        ProductRepr,
        r#"
        SELECT p.id, name, thumbnail, price, overview, in_stock, origin, amount_per_unit, measurement_unit,
            aso.new_price, aso.quantity1, aso.quantity2, aso.members_only,
            vendors.display_name AS vendor_name,
            EXISTS(
                SELECT 1
                FROM customer_favorites cf
                WHERE cf.customer = $1 AND cf.product = p.id
            ) AS "favorited!"
        FROM products p
        LEFT JOIN active_special_offers aso ON aso.product = p.id
        JOIN vendors ON vendors.id = p.vendor
        WHERE visible AND category = $2 AND in_stock > 0 AND p.id != $3
        ORDER BY offers_discount(price, aso.new_price, aso.quantity1, aso.quantity2) DESC
        LIMIT $4
        OFFSET $5
        "#,
        customer.map(Id::get),
        category.get(),
        except.get(),
        i64::try_from(limit)?,
        i64::try_from(offset)?,
    )
    .fetch_all(connection())
    .await
    .map(|products| products.into_iter().map(Into::<ProductOverview>::into).collect::<Box<_>>())
    .inspect(|products| {
        debug_assert!(
            products.is_sorted_by_key(|ProductOverview { special_offer, price, .. }|
                Reverse(special_offer.map(|(deal, _)| deal.discount_average(*price)))
            )
        );
    })
    .map_err(Into::into)
}

/// Get products with active discounts sorted by best discounts, as defined by
/// [`discount_average`](Deal::discount_average).
///
/// Only visible products with units in stock are considered.
///
/// # Errors
///
/// Fails if:
/// - `limit > i64::MAX`.
/// - `offset > i64::MAX`.
/// - An error occurs during communication with the database.
#[server]
pub async fn best_discounts(
    customer: Option<Id<Customer>>,
    limit: usize,
    offset: usize,
) -> Result<Box<[ProductOverviewDiscounted]>> {
    query_as!(
        ProductReprDiscounted,
        r#"
        SELECT p.id, name, thumbnail, price, overview, in_stock, origin, amount_per_unit, measurement_unit,
            aso.new_price, aso.quantity1, aso.quantity2, aso.members_only AS "members_only!",
            vendors.display_name AS vendor_name,
            EXISTS(
                SELECT 1
                FROM customer_favorites cf
                WHERE cf.customer = $1 AND cf.product = p.id
            ) AS "favorited!"
        FROM products p
        JOIN active_special_offers aso ON aso.product = p.id
        JOIN vendors ON vendors.id = p.vendor
        WHERE visible AND in_stock > 0
        ORDER BY offers_discount(price, aso.new_price, aso.quantity1, aso.quantity2) DESC
        LIMIT $2
        OFFSET $3
        "#,
        customer.map(Id::get),
        i64::try_from(limit)?,
        i64::try_from(offset)?,
    )
    .fetch_all(connection())
    .await
    .map(|products| products.into_iter().map(Into::<ProductOverviewDiscounted>::into).collect::<Box<_>>())
    .inspect(|products| {
        debug_assert!(
            products.is_sorted_by_key(|ProductOverviewDiscounted { special_offer_deal, price, .. }|
                Reverse(special_offer_deal.discount_average(*price))
            )
        );
    })
    .map_err(Into::into)
}

/// Get products owned by a given vendor sorted by best discounts as defined by
/// [`discount_average`](Deal::discount_average), then name.
///
/// Only visible products are considered, but includes products out of stock.
///
/// # Errors
///
/// Fails if:
/// - `limit > i64::MAX`.
/// - `offset > i64::MAX`.
/// - An error occurs during communication with the database.
#[server]
pub async fn vendor_products(
    customer: Option<Id<Customer>>,
    vendor: Id<Vendor>,
    limit: usize,
    offset: usize,
    include_invisible: bool,
) -> Result<Box<[ProductOverviewVendor]>> {
    query_as!(
        ProductReprVendor,
        r#"
        SELECT p.id, name, thumbnail, price, overview, in_stock, origin, amount_per_unit, measurement_unit,
            aso.new_price, aso.quantity1, aso.quantity2, aso.members_only,
            EXISTS(
                SELECT 1
                FROM customer_favorites cf
                WHERE cf.customer = $1 AND cf.product = p.id
            ) AS "favorited!"
        FROM products p
        LEFT JOIN active_special_offers aso ON aso.product = p.id
        WHERE (p.visible OR $5) AND p.vendor = $2 AND p.in_stock > 0
        ORDER BY offers_discount(p.price, aso.new_price, aso.quantity1, aso.quantity2) DESC, p.name
        LIMIT $3
        OFFSET $4
        "#,
        customer.map(Id::get),
        vendor.get(),
        i64::try_from(limit)?,
        i64::try_from(offset)?,
        include_invisible,
    )
    .fetch_all(connection())
    .await
    .map(|products| products.into_iter().map(Into::<ProductOverviewVendor>::into).collect::<Box<_>>())
    .inspect(|products| {
        debug_assert!(
            products.is_sorted_by_key(|ProductOverviewVendor { special_offer, price, .. }|
                Reverse(special_offer.map(|(deal, _)| deal.discount_average(*price)))
            )
        );
    })
    .map_err(Into::into)
}

/// Get all products a customer has marked as favorites sorted by name.
///
/// Only visible products are considered, but may incldue products out of stock.
///
/// # Errors
///
/// Fails if:
/// - `limit > i64::MAX`.
/// - `offset > i64::MAX`.
/// - An error occurs during communication with the database.
#[server]
pub async fn favorites(
    customer: Id<Customer>,
    limit: usize,
    offset: usize,
) -> Result<Box<[ProductOverviewFavorited]>> {
    query_as!(
        ProductReprFavorited,
        r#"
        SELECT p.id, name, thumbnail, price, overview, in_stock, origin, amount_per_unit, measurement_unit,
            aso.new_price, aso.quantity1, aso.quantity2, aso.members_only,
            vendors.display_name AS vendor_name
        FROM products p
        LEFT JOIN active_special_offers aso ON aso.product = p.id
        JOIN vendors ON vendors.id = p.vendor
        JOIN customer_favorites cf ON cf.product = p.id
        WHERE visible AND cf.customer = $1
        ORDER BY name
        LIMIT $2
        OFFSET $3
        "#,
        customer.get(),
        i64::try_from(limit)?,
        i64::try_from(offset)?,
    )
    .fetch_all(connection())
    .await
    .map(|products| products.into_iter().map(Into::into).collect())
    .map_err(Into::into)
}
