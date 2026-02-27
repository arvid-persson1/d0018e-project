//! Database functions for interacting with a customer's shopping cart.

use crate::database::{Customer, Id, Product};
use dioxus::prelude::*;
#[cfg(feature = "server")]
use {
    crate::database::{QueryResultExt, connection},
    sqlx::query,
};

// TODO: Function to get items in cart, include prices with discounts, separate member/nonmember
// prices?

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
/// # Errors
///
/// Fails if:
/// - `customer` is invalid.
/// - The customer has any deleted or invisible products in their cart.
/// - The customer has more units in their cart than there are in stock.
/// - An error occurs during communication with the database.
#[server]
pub async fn checkout(customer: Id<Customer>) -> Result<()> {
    query!("CALL checkout($1)", customer.get())
        .execute(connection())
        .await
        .map(QueryResultExt::procedure)
        .map_err(Into::into)
}
