//! Database functions for interacting with a customer's shopping cart.

use crate::database::{Customer, Id, Product, SpecialOffer};
use dioxus::prelude::*;
use hashbrown::HashMap;
use std::num::NonZeroU32;
#[cfg(feature = "server")]
use {
    crate::database::{QueryResultExt, connection},
    sqlx::{query, query_as},
    std::num::NonZero,
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
pub type CartCounts = HashMap<Id<Product>, NonZeroU32>;

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
        "
        SELECT product, number
        FROM shopping_cart_items
        WHERE customer = $1 AND product IS NOT NULL
        ",
        customer.get(),
    )
    .fetch_all(connection())
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
        .execute(connection())
        .await
        .map(QueryResultExt::expect_maybe)
        .map_err(Into::into)
    } else if let Ok(number) = i32::try_from(number) {
        query!(
            "
            INSERT INTO shopping_cart_items (customer, product, number)
            VALUES ($1, $2, $3::INT)
            ON CONFLICT (customer, product) DO UPDATE
            SET number = EXCLUDED.number
            ",
            customer.get(),
            product.get(),
            number,
        )
        .execute(connection())
        .await
        .map(QueryResultExt::expect_one)
        .map_err(Into::into)
    } else {
        todo!()
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
    .execute(connection())
    .await
    .map(QueryResultExt::allow_any)
    .map_err(Into::into)
}

/// Complete an order for a customer, emptying their shopping cart.
///
/// Calls may also include the IDs of all special offer expected to be applied. This avoids the
/// case where the customer sees one price based on a special offer active at the time, but the
/// database calculates a different price because the previously mentioned offer has lapsed. If no
/// such list is provided, price will be based on whatever special offers are active at the time.
///
/// # Errors
///
/// Fails if:
/// - `customer` is invalid.
/// - Any offer in `expected_offers` has lapsed.
/// - The customer has any deleted or invisible products in their cart.
/// - The customer has more units in their cart than there are in stock.
/// - An error occurs during communication with the database.
#[server]
pub async fn checkout(
    customer: Id<Customer>,
    expected_offers: Option<Box<[Id<SpecialOffer>]>>,
) -> Result<()> {
    // This should be a no-op.
    let expected_offers =
        expected_offers.map(|ids| ids.into_iter().map(Id::get).collect::<Box<_>>());
    query!(
        "CALL checkout($1, $2)",
        customer.get(),
        expected_offers.as_deref(),
    )
    .execute(connection())
    .await
    .map(QueryResultExt::procedure)
    .map_err(Into::into)
}
