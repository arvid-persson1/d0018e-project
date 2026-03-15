//! Database functions for interacting with a customer's shopping cart.

use crate::database::{Customer, Deal, Id, Product, SpecialOffer, Url};
use dioxus::prelude::*;
use hashbrown::HashMap;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use time::PrimitiveDateTime;
#[cfg(feature = "server")]
use {
    crate::database::{POOL, QueryResultExt},
    sqlx::{Type, query, query_as, query_scalar},
    std::num::{NonZero, TryFromIntError},
};

// TODO: Function to get items in cart for display on cart page, include prices with discounts,
// separate member/nonmember prices? Must include time of reading, see `checkout`.

#[cfg(feature = "server")]
struct CartCountRepr {
    product: i32,
    number: i32,
}

/// The contents of a customer's shopping cart.
///
/// Does not include deleted or invisible products.
pub type Counts = HashMap<Id<Product>, NonZeroU32>;

/// Get the contents of a customer's cart.
///
/// Ignores deleted or invisible products.
///
/// # Errors
///
/// Fails if:
/// - `customer` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn cart_counts(customer: Id<Customer>) -> Result<HashMap<Id<Product>, NonZeroU32>> {
    query_as!(
        CartCountRepr,
        r#"
        SELECT product AS "product!", number
        FROM shopping_cart_items
        WHERE customer = $1 AND product IS NOT NULL
        "#,
        customer.get(),
    )
    .fetch_all(&*POOL)
    .await
    .map(|items| {
        items
            .into_iter()
            .map(|CartCountRepr { product, number }| {
                (
                    product.into(),
                    u32::try_from(number)
                        .ok()
                        .and_then(NonZero::new)
                        .expect("Database returned non-positive number in cart."),
                )
            })
            .collect()
    })
    .map_err(Into::into)
}

/// Put `number` units of a product in a customer's shopping cart, *overriding any number
/// already there*. Setting `number = 0` removes the product from the shopping cart.
///
/// # Errors
///
/// Fails if:
/// - `customer` or `product` is invalid.
/// - `number > i32::MAX`.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_in_shopping_cart(
    customer: Id<Customer>,
    product: Id<Product>,
    number: u32,
) -> Result<()> {
    if number == 0 {
        query!(
            "
            DELETE FROM shopping_cart_items
            WHERE customer = $1 AND product = $2
            ",
            customer.get(),
            product.get(),
        )
        .execute(&*POOL)
        .await
        .map(QueryResultExt::expect_maybe)
        .map_err(Into::into)
    } else {
        query!(
            "
            INSERT INTO shopping_cart_items (customer, product, number)
            VALUES ($1, $2, $3::INT)
            ON CONFLICT (customer, product) DO UPDATE
            SET number = EXCLUDED.number
            ",
            customer.get(),
            product.get(),
            i32::try_from(number)?,
        )
        .execute(&*POOL)
        .await
        .map(QueryResultExt::allow_any)
        .map_err(Into::into)
    }
}

/// Remove all products from a customer's cart that have been deleted since addition to the cart.
///
/// # Errors
///
/// Fails if:
/// - `customer` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn remove_deleted_from_cart(customer: Id<Customer>) -> Result<()> {
    query!(
        "
        DELETE FROM shopping_cart_items
        WHERE customer = $1 AND product IS NULL
        ",
        customer.get(),
    )
    .execute(&*POOL)
    .await
    .map(QueryResultExt::allow_any)
    .map_err(Into::into)
}

/// A product in a user's shopping cart.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CartProduct {
    /// The ID of the product.
    pub id: Id<Product>,
    /// The name of the product.
    pub name: Box<str>,
    /// URL to an image to display on the product card.
    pub thumbnail: Url,
    /// The price of the product before any discounts.
    pub price: Decimal,
    /// How many units are in stock. This should not be displayed directly, but may be used
    /// together with `count` to display "low stock".
    pub in_stock: u32,
    /// How many units are in the cart.
    pub count: NonZeroU32,
    /// special offer ID
    pub special_offer_id: Option<Id<crate::database::SpecialOffer>>,
    /// The currently active special offer on the product, if any and if the user is eligible.
    pub special_offer_deal: Option<Deal>,
    /// How many more times the customer can benefit from the special offer, if there's a limit.
    /// Value is unspecified if `special_offer_deal` is `None`.
    pub special_offer_remaining_uses: Option<u32>,
    /// Whether the special offer only applies to members. Value is unspecified if
    /// `special_offer_deal` is `None`.
    pub special_offer_members_only: bool,
    /// Whether the customer has marked the product as a favorite. Value is unspecified if a
    /// customer ID was not provided.
    pub favorited: bool,
}

#[cfg(feature = "server")]
struct CartProductRepr {
    id: i32,
    name: String,
    thumbnail: Url,
    price: Decimal,
    in_stock: i32,
    count: i32,
    special_offer_id: Option<i32>,
    new_price: Option<Decimal>,
    quantity1: Option<i32>,
    quantity2: Option<i32>,
    members_only: bool,
    remaining_uses: Option<i32>,
    favorited: bool,
}

#[cfg(feature = "server")]
impl From<CartProductRepr> for CartProduct {
    fn from(
        CartProductRepr {
            id,
            name,
            thumbnail,
            price,
            in_stock,
            count,
            special_offer_id,
            new_price,
            quantity1,
            quantity2,
            members_only,
            remaining_uses,
            favorited,
        }: CartProductRepr,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            thumbnail,
            price,
            in_stock: in_stock
                .try_into()
                .expect("Database returned negative stock."),
            count: u32::try_from(count)
                .ok()
                .and_then(|count| count.try_into().ok())
                .expect("Database returned non-positive cart item count."),
            special_offer_id: special_offer_id.map(Into::into),
            special_offer_deal: Deal::try_from_repr(new_price, quantity1, quantity2, price)
                .expect("Database returned invalid special offer."),
            special_offer_remaining_uses: remaining_uses.map(|uses| {
                uses.try_into()
                    .expect("Database returned negative remaining uses.")
            }),
            special_offer_members_only: members_only,
            favorited,
        }
    }
}

/// Get the contents of a customer's cart, as well as the time when that data was known to be
/// valid.
///
/// The timestamp returned from this function is the one that should be passed to [`checkout`].
#[server]
pub async fn cart_products(
    customer: Id<Customer>,
) -> Result<(Box<[CartProduct]>, PrimitiveDateTime)> {
    let mut tx = POOL.begin().await?;

    let time = query_scalar!(r#"SELECT CURRENT_TIMESTAMP::TIMESTAMP AS "time!""#)
        .fetch_one(&mut *tx)
        .await?;

    let products = query_as!(
        CartProductRepr,
        r#"
        SELECT p.id, name, thumbnail, price, in_stock, s.number AS count,
            aso.id AS special_offer_id,
            new_price, quantity1, quantity2, COALESCE(members_only, FALSE) AS "members_only!",
            limit_per_customer - COALESCE(sou.number, 0) AS remaining_uses,
            EXISTS (
                SELECT 1
                FROM customer_favorites cf
                WHERE cf.customer = $1 AND cf.product = p.id
            ) AS "favorited!"
        FROM shopping_cart_items s
        JOIN products p ON p.id = s.product
        JOIN customers ON customers.id = $1
        LEFT JOIN active_special_offers aso ON aso.product = s.product
            AND (NOT members_only OR member_since IS NOT NULL)
        LEFT JOIN special_offer_uses sou ON special_offer = aso.id AND sou.customer = $1
        WHERE s.customer = $1 AND s.number > 0
        "#,
        customer.get()
    )
    .fetch_all(&mut *tx)
    .await?
    .into_iter()
    .map(Into::into)
    .collect();

    tx.commit().await?;

    Ok((products, time))
}

/// An item a customer wants to check out with.
///
/// This is to ensure the customer proceeds with what they see in the cart, which might be
/// different from what the database knows about due to concurrent accesses, expired offers or
/// other forms of stale data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckoutItem {
    /// The ID of the product.
    pub product: Id<Product>,
    /// The number of units.
    pub number: NonZeroU32,
    /// The special offer the customer expects to be applied, if any.
    pub special_offer: Option<Id<SpecialOffer>>,
    /// The price the customer expects to pay.
    pub expected_price: Decimal,
}

#[cfg(feature = "server")]
#[derive(Type)]
#[sqlx(type_name = "CHECKOUT_ITEM")]
struct CheckoutItemRepr {
    product: i32,
    number: i32,
    special_offer: Option<i32>,
    expected_price: Decimal,
}

#[cfg(feature = "server")]
impl TryFrom<CheckoutItem> for CheckoutItemRepr {
    type Error = TryFromIntError;

    fn try_from(
        CheckoutItem {
            product,
            number,
            special_offer,
            expected_price,
        }: CheckoutItem,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            product: product.get(),
            number: number.get().try_into()?,
            special_offer: special_offer.map(Id::get),
            expected_price,
        })
    }
}

/// Complete an order for a customer.
///
/// Requires specifying the exact contents of the cart as the customer sees it, as well as the time
/// that data was loaded. This is to deny checkout frm proceeding with stale data. The time should
/// be the one returned from [`cart_products`].
///
/// # Errors
///
/// Fails if:
/// - `customer` is invalid.
/// - Any data in `items` is stale, including:
///   - A product having changed (e.g. new name or price).
///   - A product no longer having enough stock.
///   - A product no longer being visible.
///   - A special offer having expired.
///   - The customer no longer being eligible for a special offer due to a membership change.
///   - The customer not being able to apply a special offer enough times to achieve the expected
///     price due to e.g. a concurrent checkout with the same account.
/// - `seen_at` is in the future.
/// - An error occurs during communication with the database.
#[server]
pub async fn checkout(
    customer: Id<Customer>,
    items: Vec<CheckoutItem>,
    seen_at: PrimitiveDateTime,
) -> Result<()> {
    let items = items
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<Box<_>, _>>()?;
    query!(
        "CALL checkout($1, $2, ($3::TIMESTAMP)::NONFUTURE_TIMESTAMP)",
        customer.get(),
        &items as &[CheckoutItemRepr],
        seen_at,
    )
    .execute(&*POOL)
    .await
    .map(QueryResultExt::allow_any)
    .map_err(Into::into)
}
