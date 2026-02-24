//! Database functions for interacting with a customer's shopping cart.

use crate::database::{Customer, Id, Product};
use dioxus::prelude::*;
#[cfg(feature = "server")]
use {
    crate::database::{QueryResultExt, connection},
    sqlx::query,
};

/// Puts `number` units of a product in a customer's shopping cart, *overriding any number
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

/// Removes all products from a customer's cart that have been deleted since addition to the cart.
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

/// Completes an order for a customer, emptying their shopping cart.
///
/// # Errors
///
/// Fails if:
/// - `customer` is invalid.
/// - An error occurs during communication with the database.
#[server]
#[expect(unused_variables, reason = "TODO")]
#[expect(unused_mut, reason = "TODO")]
#[expect(unreachable_code, reason = "TODO")]
#[expect(clippy::map_err_ignore, reason = "TODO")]
pub async fn checkout(customer: Id<Customer>) -> Result<()> {
    let mut tx = connection().begin().await?;

    todo!();

    tx.commit().await.map_err(|_| todo!())
}
