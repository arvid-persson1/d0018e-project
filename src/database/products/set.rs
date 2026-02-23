//! Database functions for creating and editing products.

use crate::database::{Amount, Category, Customer, Id, Product, Rating, Url, Vendor};
use dioxus::prelude::*;
use rust_decimal::Decimal;
use std::num::NonZeroU32;
use time::Date;
#[cfg(feature = "server")]
use {
    crate::database::{QueryResultExt, connection},
    sqlx::{query, query_as},
    std::num::NonZeroI32,
};

/// Create a new product.
///
/// # Errors
///
/// Fails if:
/// - `vendor` or `category` is invalid.
/// - `name` is not unique.
/// - An error occurs during communication with the database.
#[server]
pub async fn create_product(
    vendor: Id<Vendor>,
    name: Box<str>,
    thumbnail: Url,
    gallery: Box<[Url]>,
    price: Decimal,
    overview: Box<str>,
    description: Box<str>,
    category: Id<Category>,
    amount: Amount,
    origin: Box<str>,
) -> Result<()> {
    query!(
        "
        INSERT INTO products (
            vendor, name, thumbnail, gallery, price, overview, description,
            origin, category, amount_per_unit, measurement_unit
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ",
        vendor.get(),
        &name,
        thumbnail as _,
        &*gallery as _,
        price as _,
        &overview,
        &description,
        &origin,
        category.get(),
        amount.quantity() as _,
        amount.unit(),
    )
    .execute(connection())
    .await
    .map(QueryResultExt::expect_one)
    .map_err(Into::into)
}

/// Set the name of a product.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_product_name(product: Id<Product>, name: Box<str>) -> Result<()> {
    query!(
        "
        UPDATE products
        SET name = $2
        WHERE id = $1
        ",
        product.get(),
        &name,
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}

/// Set the thumbnail of a product.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_thumbnail(product: Id<Product>, url: Url) -> Result<()> {
    query!(
        "
        UPDATE products
        SET thumbnail = $2
        WHERE id = $1
        ",
        product.get(),
        url as _,
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}

/// Get the gallery of a product.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn gallery(product: Id<Product>) -> Result<Box<[Url]>> {
    struct GalleryRepr {
        gallery: Vec<Url>,
    }

    query_as!(
        GalleryRepr,
        r#"
        SELECT gallery AS "gallery: _"
        FROM products
        WHERE id = $1
        "#,
        product.get(),
    )
    .fetch_one(connection())
    .await
    .map(|GalleryRepr { gallery }| gallery.into())
    .map_err(Into::into)
}

/// Set the gallery of a product.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_gallery(product: Id<Product>, gallery: Box<[Url]>) -> Result<()> {
    query!(
        "
        UPDATE products
        SET gallery = $2
        WHERE id = $1
        ",
        product.get(),
        &*gallery as _,
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}

/// Append to the gallery of a product.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn add_to_gallery(product: Id<Product>, additions: Box<[Url]>) -> Result<()> {
    query!(
        "
        UPDATE products
        SET gallery = gallery || $2
        WHERE id = $1
        ",
        product.get(),
        &*additions as _,
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}

/// Set the price of a product.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - The new price is lower than one provided by an active special offer.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_price(product: Id<Product>, price: Decimal) -> Result<()> {
    query!(
        "
        UPDATE products
        SET price = $2
        WHERE id = $1
        ",
        product.get(),
        price as _,
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}

/// Set the overview of a product.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_overview(product: Id<Product>, overview: Box<str>) -> Result<()> {
    query!(
        "
        UPDATE products
        SET overview = $2
        WHERE id = $1
        ",
        product.get(),
        &overview,
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}

/// Set the description of a product.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_description(product: Id<Product>, description: Box<str>) -> Result<()> {
    query!(
        "
        UPDATE products
        SET description = $2
        WHERE id = $1
        ",
        product.get(),
        &description,
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}

/// Set the category of a product.
///
/// # Errors
///
/// Fails if:
/// - `product` or `category` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_category(product: Id<Product>, category: Id<Category>) -> Result<()> {
    query!(
        "
        UPDATE products
        SET category = $2
        WHERE id = $1
        ",
        product.get(),
        category.get(),
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}

/// Set the amount per unit of a product.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_amount(product: Id<Product>, amount: Amount) -> Result<()> {
    query!(
        "
        UPDATE products
        SET amount_per_unit = $2, measurement_unit = $3
        WHERE id = $1
        ",
        product.get(),
        amount.quantity() as _,
        amount.unit(),
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}

/// Set the origin of a product.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_origin(product: Id<Product>, origin: Box<str>) -> Result<()> {
    query!(
        "
        UPDATE products
        SET origin = $2
        WHERE id = $1
        ",
        product.get(),
        &origin,
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}

/// Add units to stock.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - `expiry` is in the past.
/// - The number overflows.
/// - An error occurs during communication with the database.
#[server]
pub async fn add_stock(
    product: Id<Product>,
    expiry: Option<Date>,
    number: NonZeroU32,
) -> Result<()> {
    // NOTE: `expiry` intentionally not checked for being in the past as even then the database
    // might see it at a later time where it then is in the past.

    let number = i32::try_from(number.get())?;

    let mut tx = connection().begin().await?;

    query!(
        "
        UPDATE products
        SET in_stock = in_stock + $2
        WHERE id = $1
        ",
        product.get(),
        number
    )
    .execute(&mut *tx)
    .await?
    .by_unique_key(|| todo!())?;

    query!(
        "
        INSERT INTO expiries (product, expiry, number)
        VALUES ($1, $2, $3)
        ",
        product.get(),
        expiry as _,
        number as _,
    )
    .execute(&mut *tx)
    .await?
    .expect_one();

    tx.commit().await.map_err(|_err| todo!())
}

/// Set the visibility of a product.
///
/// To any customers who had the product in their carts, a product being made invisible is
/// identical to it having been deleted.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_visibility(product: Id<Product>, visible: bool) -> Result<()> {
    let mut tx = connection().begin().await?;

    query!(
        "
        UPDATE products
        SET visible = $2
        WHERE id = $1
        ",
        product.get(),
        visible,
    )
    .execute(&mut *tx)
    .await?
    .by_unique_key(|| todo!())?;

    query!(
        "
        UPDATE shopping_cart_items
        SET product = NULL
        WHERE product = $1
        ",
        product.get(),
    )
    .execute(&mut *tx)
    .await?
    .allow_any();

    tx.commit().await.map_err(|_err| todo!())
}

/// Sets the "favorite" status of a product for a customer.
///
/// # Errors
///
/// Fails if:
/// - `customer` or `product` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_favorite(
    customer: Id<Customer>,
    product: Id<Product>,
    favorite: bool,
) -> Result<()> {
    if favorite {
        query!(
            "
            INSERT INTO customer_favorites (customer, product)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            ",
            customer.get(),
            product.get(),
        )
    } else {
        query!(
            "
            DELETE FROM customer_favorites
            WHERE customer = $1 AND product = $2
            ",
            customer.get(),
            product.get(),
        )
    }
    .execute(connection())
    .await
    .map(QueryResultExt::expect_maybe)
    .map_err(Into::into)
}

/// Sets a customer's rating on a product.
///
/// # Errors
///
/// Fails if:
/// - `customer` or `product` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_rating(
    customer: Id<Customer>,
    product: Id<Product>,
    rating: Rating,
) -> Result<()> {
    query!(
        "
        INSERT INTO ratings (customer, product, rating)
        VALUES ($1, $2, $3)
        ON CONFLICT (customer, product) DO UPDATE
        SET rating = EXCLUDED.rating
        ",
        customer.get(),
        product.get(),
        NonZeroI32::from(rating.get()) as _,
    )
    .execute(connection())
    .await
    .map(QueryResultExt::expect_one)
    .map_err(Into::into)
}

/// Removes a customer's rating on a product.
///
/// # Errors
///
/// Fails if:
/// - `customer` or `product` is invalid.
/// - Customer has a review on the product.
/// - An error occurs during communication with the database.
#[server]
pub async fn remove_rating(customer: Id<Customer>, product: Id<Product>) -> Result<()> {
    query!(
        "
        DELETE FROM ratings
        WHERE customer = $1 AND product = $2
        ",
        customer.get(),
        product.get(),
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}
