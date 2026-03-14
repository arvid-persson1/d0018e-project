//! Database functions for interacting with product information to be displayed on product
//! pages.

use crate::database::{
    Amount, AverageRating, Category, Customer, Deal, Id, Order, Product, Rating, Url, Vendor,
};
use dioxus::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use time::PrimitiveDateTime;
#[cfg(feature = "server")]
use {
    crate::database::{POOL, QueryResultExt as _, RawId},
    sqlx::{Type, query, query_as},
    std::{cmp::Reverse, num::NonZero},
};

/// Information about a product, for display on product pages.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    /// The id and name of the product's category and all of its parents, starting from the root.
    pub category: Box<[(Id<Category>, Box<str>)]>,
    /// How much of the product is included in one unit.
    pub amount_per_unit: Amount,
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
    pub created_at: PrimitiveDateTime,
    /// When the product was last updated.
    pub updated_at: PrimitiveDateTime,
    /// The average rating of the product.
    pub rating: AverageRating,
    /// The currently active special offer on the product, if any.
    pub special_offer_deal: Option<Deal>,
    /// How many times each customer can benefit from the special offer, if there's a limit. Value
    /// is unspecified if `special_offer_deal` is `None`.
    pub special_offer_limit_per_customer: Option<NonZeroU32>,
    /// Whether the special offer only applies to members. Value is unspecified if
    /// `special_offer_deal` is `None`.
    pub special_offer_members_only: bool,
    /// Whether the customer has marked the product as a favorite. Value is unspecified if a
    /// customer ID was not provided.
    pub favorited: bool,
    /// The customer's rating of the product. Value is unspecified if a customer ID was not
    /// provided.
    pub own_rating: Rating,
    /// Whether the customer has ever bought the product. Customers are not able to rate products
    /// they have not bought. Value is unspecified if a customer ID was not provided.
    pub has_purchased: bool,
}

#[cfg(feature = "server")]
#[derive(Type)]
struct CategoryPathSegment {
    id: RawId,
    name: String,
}

#[cfg(feature = "server")]
struct ProductInfoRepr {
    name: String,
    gallery: Vec<Url>,
    thumbnail: String,
    price: Decimal,
    description: String,
    in_stock: i32,
    amount_per_unit: Decimal,
    measurement_unit: Option<String>,
    origin: String,
    visible: bool,
    created_at: PrimitiveDateTime,
    updated_at: PrimitiveDateTime,
    new_price: Option<Decimal>,
    quantity1: Option<i32>,
    quantity2: Option<i32>,
    limit_per_customer: Option<i32>,
    members_only: bool,
    vendor_id: RawId,
    vendor_name: String,
    category_path: Vec<CategoryPathSegment>,
    average_rating: Option<f64>,
    rating_count: i64,
    favorited: bool,
    own_rating: Option<i32>,
    has_purchased: bool,
}

#[cfg(feature = "server")]
impl ProductInfo {
    #[expect(clippy::missing_panics_doc, reason = "Database validation only.")]
    fn from_repr(
        id: Id<Product>,
        ProductInfoRepr {
            name,
            gallery,
            thumbnail,
            price,
            description,
            in_stock,
            amount_per_unit,
            measurement_unit,
            origin,
            visible,
            created_at,
            updated_at,
            new_price,
            quantity1,
            quantity2,
            members_only,
            limit_per_customer,
            vendor_id,
            vendor_name,
            category_path,
            average_rating,
            rating_count,
            favorited,
            own_rating,
            has_purchased,
        }: ProductInfoRepr,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            gallery: if gallery.is_empty() {
                vec![thumbnail.into()].into()
            } else {
                gallery.into_iter().map(Into::into).collect()
            },
            price,
            description: description.into(),
            in_stock: in_stock
                .try_into()
                .expect("Database returned negative stock."),
            category: category_path
                .into_iter()
                .map(|CategoryPathSegment { id, name }| (id.into(), name.into()))
                .collect(),
            amount_per_unit: Amount::from_repr(amount_per_unit, measurement_unit),
            visible,
            vendor_id: vendor_id.into(),
            vendor_name: vendor_name.into(),
            origin: origin.into(),
            created_at,
            updated_at,
            rating: AverageRating::from_repr(average_rating, rating_count),
            special_offer_deal: Deal::try_from_repr(new_price, quantity1, quantity2, price)
                .expect("Database returned invalid special offer."),
            special_offer_limit_per_customer: limit_per_customer.map(|l| {
                u32::try_from(l)
                    .ok()
                    .and_then(NonZeroU32::new)
                    .expect("Database returned non-positive limit.")
            }),
            special_offer_members_only: members_only,
            favorited,
            own_rating: Rating::new(own_rating.map_or(1, |r| {
                r.try_into().expect("Database returned invalid own rating.")
            }))
            .expect("Database returned invalid rating."),
            has_purchased,
        }
    }
}

/// Get information about a product, for display on product pages.
///
/// If the gallery was empty, it will consist of a single copy of the thumbnail.
///
/// # Errors
///
/// Fails if:
/// - `customer` (if [`Some`]) or `product` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn product_info(
    customer: Option<Id<Customer>>,
    product: Id<Product>,
) -> Result<ProductInfo> {
    query_as!(
        ProductInfoRepr,
        r#"
        SELECT name, thumbnail, price, p.description, in_stock, origin,
            gallery AS "gallery: Vec<Url>", amount_per_unit, measurement_unit, visible,
            created_at, p.updated_at, new_price, quantity1, quantity2,
            COALESCE(members_only, FALSE) AS "members_only!", limit_per_customer,
            vendors.id AS vendor_id, vendors.display_name AS vendor_name,
            category_path(category) AS "category_path!: Vec<CategoryPathSegment>",
            AVG(ratings.rating::FLOAT) AS average_rating,
            COUNT(ratings.rating) AS "rating_count!",
            EXISTS (
                SELECT 1
                FROM customer_favorites cf
                WHERE cf.customer = $1 AND cf.product = p.id
            ) AS "favorited!",
            (
                SELECT rating
                FROM ratings
                WHERE customer = $1 AND product = $2
            ) AS own_rating,
            EXISTS (
                SELECT 1
                FROM orders
                WHERE customer = $1 AND product = $2
            ) AS "has_purchased!"
        FROM products p
        LEFT JOIN active_special_offers ON product = p.id
        JOIN vendors ON vendors.id = vendor
        LEFT JOIN ratings ON ratings.product = p.id
        WHERE p.id = $2
        GROUP BY p.id, vendors.id, new_price, quantity1, quantity2, members_only, limit_per_customer
        "#,
        customer.map(Id::get),
        product.get()
    )
    .fetch_one(&*POOL)
    .await
    .map(|repr| ProductInfo::from_repr(product, repr))
    .map_err(Into::into)
}

/// A customer's order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderInfo {
    /// The time of purchase.
    pub time: PrimitiveDateTime,
    /// Purchases included in this order.
    pub purchases: Vec<Purchase>,
}

/// A record of a customer's purchase.
///
/// This does not include a timestamp, as they are intended to be grouped by timestamp in an
/// [`Order`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Purchase {
    /// How much was paid.
    pub paid: Decimal,
    /// How many units were purchased.
    pub number: NonZeroU32,
    /// The name of the product.
    pub product_name: Box<str>,
    /// URL to an image of the product.
    pub thumbnail: Url,
    /// The name of the vendor.
    pub vendor_name: Box<str>,
    /// Whether the product has changed since the order was made. This should be marked by an
    /// indicator.
    pub product_changed: bool,
    /// The status of the purchase.
    pub status: OrderStatus,
}

#[cfg(feature = "server")]
struct PurchaseRepr {
    time: PrimitiveDateTime,
    paid: Decimal,
    number: i32,
    product_name: String,
    thumbnail: String,
    vendor_name: String,
    product_changed: bool,
    status: OrderStatus,
}

#[cfg(feature = "server")]
impl From<PurchaseRepr> for Purchase {
    fn from(
        PurchaseRepr {
            time: _,
            paid,
            number,
            product_name,
            thumbnail,
            vendor_name,
            product_changed,
            status,
        }: PurchaseRepr,
    ) -> Self {
        Self {
            paid,
            number: u32::try_from(number)
                .ok()
                .and_then(NonZeroU32::new)
                .expect("Database returned non-positive number in order."),
            product_name: product_name.into(),
            thumbnail: thumbnail.into(),
            vendor_name: vendor_name.into(),
            product_changed,
            status,
        }
    }
}

/// The status of an order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(Type))]
#[cfg_attr(
    feature = "server",
    sqlx(type_name = "order_status", rename_all = "lowercase")
)]
pub enum OrderStatus {
    /// Order has been placed.
    Pending,
    /// Vendor has sent the order.
    Shipped,
    /// Customer has received the order; completed.
    Received,
}

/// Get orders made by a customer sorted by recency.
///
/// # Errors
///
/// Fails if:
/// - `customer` is invalid.
/// - `limit > i64::MAX`.
/// - `offset > i64::MAX`.
/// - An error occurs during communication with the database.
#[server]
pub async fn customer_orders(
    customer: Id<Customer>,
    limit: usize,
    offset: usize,
) -> Result<Box<[OrderInfo]>> {
    let orders = query_as!(
        PurchaseRepr,
        r#"
        SELECT time, paid, number, status AS "status: OrderStatus", p.name AS product_name, p.thumbnail,
            display_name AS vendor_name, time > updated_at AS "product_changed!"
        FROM orders o
        JOIN products p ON p.id = o.product
        JOIN vendors ON vendors.id = p.vendor
        WHERE customer = $1
        ORDER BY time DESC
        LIMIT $2
        OFFSET $3
        "#,
        customer.get(),
        i64::try_from(limit)?,
        i64::try_from(offset)?,
    )
    .fetch_all(&*POOL)
    .await?
    .into_iter()
    .map(|purchase| (purchase.time, Purchase::from(purchase)))
    .fold(Vec::<OrderInfo>::new(), |mut acc, (time, purchase)| {
        if let Some(last) = acc.last_mut()
            && time == last.time
        {
            last.purchases.push(purchase);
        } else {
            acc.push(OrderInfo {
                time,
                purchases: vec![purchase],
            });
        }
        acc
    });

    debug_assert!(
        orders.is_sorted_by_key(|order| Reverse(order.time)),
        "Orders not sorted."
    );
    Ok(orders.into())
}

/// A vendor's view of an order of one of their products.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderVendorView {
    /// The time of purchase.
    pub time: PrimitiveDateTime,
    /// The product purchased.
    pub product: Id<Product>,
    /// The name of the product.
    pub product_name: Box<str>,
    /// How many units were purchased.
    pub number: NonZeroU32,
    /// Whether the product has changed since the order was made. This should be marked by an
    /// indicator.
    pub product_changed: bool,
    /// The status of the purchase.
    pub status: OrderStatus,
}

#[cfg(feature = "server")]
struct OrderVendorViewRepr {
    time: PrimitiveDateTime,
    product: i32,
    product_name: String,
    number: i32,
    product_changed: bool,
    status: OrderStatus,
}

#[cfg(feature = "server")]
impl From<OrderVendorViewRepr> for OrderVendorView {
    fn from(
        OrderVendorViewRepr {
            time,
            product,
            product_name,
            number,
            product_changed,
            status,
        }: OrderVendorViewRepr,
    ) -> Self {
        Self {
            time,
            product: product.into(),
            product_name: product_name.into(),
            number: u32::try_from(number)
                .ok()
                .and_then(NonZero::new)
                .expect("Database returned non-positive number in order."),
            product_changed,
            status,
        }
    }
}

/// Get orders for a vendor's products sorted by recency.
///
/// # Errors
///
/// Fails if:
/// - `vendor` is invalid.
/// - `limit > i64::MAX`.
/// - `offset > i64::MAX`.
/// - An error occurs during communication with the database.
#[server]
pub async fn vendor_orders(
    vendor: Id<Vendor>,
    limit: usize,
    offset: usize,
) -> Result<Box<[OrderVendorView]>> {
    query_as!(
        OrderVendorViewRepr,
        r#"
        SELECT time, number, status AS "status: OrderStatus",
            p.id AS product, p.name AS product_name, time > updated_at AS "product_changed!"
        FROM orders o
        JOIN products p ON p.id = o.product
        WHERE p.vendor = $1
        ORDER BY time DESC
        LIMIT $2
        OFFSET $3
        "#,
        vendor.get(),
        i64::try_from(limit)?,
        i64::try_from(offset)?,
    )
    .fetch_all(&*POOL)
    .await
    .map(|orders| orders.into_iter().map(Into::into).collect())
    .map_err(Into::into)
}

#[server]
pub async fn set_status(order: Id<Order>, status: OrderStatus) -> Result<()> {
    query!(
        "UPDATE orders SET status = $2 WHERE id = $1",
        order.get(),
        status as OrderStatus,
    )
    .execute(&*POOL)
    .await?
    .by_unique_key()
    .map_err(Into::into)
}
