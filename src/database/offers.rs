//! Database functions for interacting with special offers.

use crate::database::{Deal, Id, Product, SpecialOffer};
use dioxus::prelude::*;
use std::num::NonZeroU32;
use time::PrimitiveDateTime;
#[cfg(feature = "server")]
use {
    crate::database::{QueryResultExt, connection},
    sqlx::query,
};

/// Create a special offer for a product.
///
/// Special offers with an end time of `None` must be deleted or otherwise disabled manually.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - `take > i32::MAX` (if [`Batch`] or [`BatchPrice`]).
/// - `pay_for > i32::MAX` (if [`BatchPrice`]).
/// - `limit_per_customer > i32::MAX` (if [`Some`]).
/// - `valid_until` is in the past.
/// - The special offer overlaps with an existing one.
/// - The special offer does not actually provide a discount compared to the current price.
/// - An error occurs during communication with the database.
#[server]
pub async fn create_special_offer(
    product: Id<Product>,
    deal: Deal,
    members_only: bool,
    limit_per_customer: Option<NonZeroU32>,
    valid_from: PrimitiveDateTime,
    valid_until: Option<PrimitiveDateTime>,
) -> Result<()> {
    // NOTE: `valid_until` intentionally not checked for being in the past as even then the database
    // might see it at a later time where it then is in the past.

    let (new_price, quantity1, quantity2) = deal.database_repr().ok_or_else(|| todo!())?;

    query!(
        "
        INSERT INTO special_offers (
            product, members_only, limit_per_customer, valid_from, valid_until,
            new_price, quantity1, quantity2
        )
        VALUES ($1, $2, $3::INT, $4, $5, $6::DECIMAL(10, 2), $7, $8)
        ",
        product.get(),
        members_only,
        limit_per_customer
            .map(|l| i32::try_from(l.get()))
            .transpose()?,
        valid_from,
        valid_until,
        new_price,
        quantity1,
        quantity2,
    )
    .execute(connection())
    .await
    .map(QueryResultExt::expect_one)
    .map_err(Into::into)
}

/// Set the limit per customer of a special offer.
///
/// This might make it so that some customers have already used the special offer more times
/// than are allowed by the new limit. These customers are restricted from further usage unless
/// the limit is increased, but no changes are made to order history.
///
/// # Errors
///
/// Fails if:
/// - `special_offer` is invalid.
/// - `limit_per_customer > i32::MAX`.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_special_offer_limit(
    special_offer: Id<SpecialOffer>,
    limit_per_customer: NonZeroU32,
) -> Result<()> {
    query!(
        "
        UPDATE special_offers
        SET limit_per_customer = $2::INT
        WHERE id = $1
        ",
        special_offer.get(),
        i32::try_from(limit_per_customer.get())?,
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}

/// Set the "members only"-status of a special offer.
///
/// # Errors
///
/// Fails if:
/// - `special_offer` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_special_offer_members_only(
    special_offer: Id<SpecialOffer>,
    members_only: bool,
) -> Result<()> {
    query!(
        "
        UPDATE special_offers
        SET members_only = $2
        WHERE id = $1
        ",
        special_offer.get(),
        members_only,
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}

/// Set the start time of a special offer.
///
/// # Errors
///
/// Fails if:
/// - `special_offer` is invalid.
/// - `valid_from` is in the past (see [`set_special_offer_start_now`] if the intent is to activate
///   it).
/// - The special offer now overlaps with an existing one.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_special_offer_start(
    special_offer: Id<SpecialOffer>,
    valid_from: PrimitiveDateTime,
) -> Result<()> {
    // NOTE: `valid_from` intentionally not checked for being in the past as even then the database
    // might see it at a later time where it then is in the past.

    query!(
        "
        UPDATE special_offers
        SET valid_from = $2
        WHERE id = $1
        ",
        special_offer.get(),
        valid_from,
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}

/// Set the start time of a special offer to "now".
///
/// # Errors
///
/// Fails if:
/// - `special_offer` is invalid.
/// - Another special offer is already active.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_special_offer_start_now(special_offer: Id<SpecialOffer>) -> Result<()> {
    query!(
        "
        UPDATE special_offers
        SET valid_from = CURRENT_TIMESTAMP
        WHERE id = $1
        ",
        special_offer.get(),
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}

/// Set the end time of a special offer.
///
/// Special offers with an end time of `None` must be deleted or otherwise disabled manually.
///
/// # Errors
///
/// Fails if:
/// - `special_offer` is invalid.
/// - `valid_until` is in the past (see [`delete_special_offer`] if the intent is to delete it).
/// - The special offer now overlaps with an existing one.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_special_offer_end(
    special_offer: Id<SpecialOffer>,
    valid_until: Option<PrimitiveDateTime>,
) -> Result<()> {
    // NOTE: `valid_until` intentionally not checked for being in the past as even then the database
    // might see it at a later time where it then is in the past.

    query!(
        "
        UPDATE special_offers
        SET valid_until = $2
        WHERE id = $1
        ",
        special_offer.get(),
        valid_until,
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}

/// Delete a special offer.
///
/// # Errors
///
/// Fails if:
/// - `special_offer` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn delete_special_offer(special_offer: Id<SpecialOffer>) -> Result<()> {
    query!(
        "
        DELETE FROM special_offers
        WHERE id = $1
        ",
        special_offer.get(),
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}

/// Set the deal of a special offer.
///
/// # Errors
///
/// Fails if:
/// - `special_offer` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_special_offer_deal(special_offer: Id<SpecialOffer>, deal: Deal) -> Result<()> {
    let (new_price, quantity1, quantity2) = deal.database_repr().ok_or_else(|| todo!())?;

    query!(
        "
        UPDATE special_offers
        SET new_price = $2::DECIMAL(10, 2), quantity1 = $3, quantity2 = $4
        WHERE id = $1
        ",
        special_offer.get(),
        new_price,
        quantity1,
        quantity2,
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}
