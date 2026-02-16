//! Database operations.
//!
//! [`Id`]s created by functions of this module are valid at the time of retrieval from the
//! database, but might of course be invalidated later as a result of deletions. It may even be the
//! case that an ID is invalidated in the time between the ID being fetched from the database and
//! the associated [`Future`] completing.

#![cfg_attr(feature = "server", expect(unused_variables, reason = "TODO"))]
#![cfg_attr(feature = "server", expect(clippy::todo, reason = "TODO"))]
#![cfg_attr(feature = "server", expect(clippy::unused_async, reason = "TODO"))]
#![allow(
    clippy::too_many_arguments,
    reason = "Consistency. Furthermore, marking an individual function doesn't work as the `#[server]` macro expands to more than a function, and the attribute is misplaced."
)]
#![allow(clippy::future_not_send, reason = "Violated by the `#[server]` macro.")]

#[cfg(feature = "server")]
use super::AverageRating;
use super::{
    Amount, Category, CategoryTree, Comment, Customer, CustomerReview, Deal, Id, Order, OwnReview,
    Product, ProductInfo, ProductOverview, ProductReview, Purchase, Rating, Review, SpecialOffer,
    Url, User, Username, Vendor, Vote, id,
};
use chrono::{NaiveDate, NaiveDateTime, Utc};
use dioxus::prelude::*;
#[cfg(feature = "server")]
use hashbrown::HashMap;
use rust_decimal::Decimal;
#[cfg(feature = "server")]
use sqlx::{PgPool as Pool, query, query_as};
use std::{cmp::Reverse, num::NonZeroU32};
#[cfg(feature = "server")]
use tokio::sync::OnceCell;

/// The shared connection to the database.
#[cfg(feature = "server")]
static CONNECTION: OnceCell<Pool> = OnceCell::const_new();

/// Initializes the database connection.
///
/// This function should be called once at program startup. Attempting to call any other database
/// function before this one will cause a panic. Calling this function multiple times does nothing.
///
/// # Panics
///
/// Panics if establishing a connection fails or if database startup code fails to run.
// TODO: Should this be a server function or just "a function that runs on the server"?
#[server]
pub async fn init_connection(url: String) -> Result<()> {
    if let Ok(()) = CONNECTION.set(
        Pool::connect(&url)
            .await
            .expect("Failed to establish a connection to the database"),
    ) {
        // TODO: Startup code.
    }

    Ok(())
}

/// Gets a handle to the database connection.
///
/// This is a convenience wrapper around `CONNECTION`, handling non-initialized state with a
/// custom panic message.
///
/// # Panics
///
/// Panics if the connection has not been initialized (see [`init_connection`]).
#[cfg(feature = "server")]
fn connection() -> &'static Pool {
    CONNECTION
        .get()
        .expect("Database connection not initialized.")
}

// TODO: Is it possible to have borrowed arguments in server functions?

/// Gets the hierarchy of categories as one or more trees.
///
/// # Errors
///
/// Fails if an error occurs during communication with the database.
#[server]
pub(crate) async fn category_trees() -> Result<Box<[CategoryTree]>> {
    #[derive(PartialEq, PartialOrd)]
    struct Category {
        parent: Option<id::Inner>,
        name: String,
        id: id::Inner,
    }

    let categories = query_as!(
        Category,
        "SELECT *
        FROM categories
        ORDER BY parent NULLS FIRST, NAME;",
    )
    .fetch_all(connection())
    .await?;

    // Ordering defined by order of fields. Names are unique so IDs will never be compared.
    debug_assert!(categories.is_sorted(), "Rows not sorted.");

    let mut map = HashMap::new();
    let mut roots = Vec::new();
    let mut it = categories.into_iter().peekable();

    while let Some(Category { name, id, .. }) = it.next_if(|c| c.parent.is_none()) {
        let node = CategoryTree {
            id: id.into(),
            name: name.into(),
            subcategories: Vec::new(),
        };
        roots.push(node);
        map.insert(id, node);
    }

    for Category { id, parent, name } in it {
        #[expect(
            clippy::unwrap_used,
            reason = "Nodes without parents have already been traversed in the previous loop."
        )]
        let parent = parent.unwrap();

        let node = CategoryTree {
            id: id.into(),
            name: name.into(),
            subcategories: Vec::new(),
        };
        map.get_mut(&parent)
            .expect("Invalid parent reference in database.")
            .subcategories
            .push(node);
        map.insert(id, node);
    }

    Ok(roots.into())
}

/// Gets products with active discounts sorted by best discounts, as defined by
/// [`discount_average`](SpecialOfferDeal::discount_average).
///
/// Only visible products with units in stock are considered.
///
/// # Errors
///
/// Fails if an error occurs during communication with the database.
#[server]
pub(crate) async fn best_discounts(limit: usize, offset: usize) -> Result<Box<[ProductOverview]>> {
    // TODO: Add tiebreaker for order?
    let products = query!(
        "SELECT id, name, thumbnail, price, overview, in_stock, amount_per_unit, origin,
                aso.new_price, aso.quantity1, aso.quantity2, aso.members_only,
                offers_discount(price, aso.new_price, aso.quantity1, aso.quantity2) AS discount,
                vendors.display_name AS vendor_name
        FROM products
        WHERE visible
          AND in_stock > 0
        LEFT JOIN active_special_offers aso
               ON products.id = aso.product
        JOIN vendors
          ON products.vendor = vendors.id
        ORDER BY discount DESC
        LIMIT $1
        OFFSET $2;",
        limit,
        offset,
    )
    .fetch_all(connection())
    .await?;

    products
        .into_iter()
        .map(|out| {
            let special_offer = if let Some(members_only) = out.members_only
                && let Some(deal) =
                    Deal::new(out.new_price, out.quantity1, out.quantity2, out.price)?
            {
                // TODO: Is this comparison too strict when considering rounding errors?
                debug_assert!(deal.discount_average(out.price) == Some(out.discount));
                Some((deal, members_only))
            } else {
                None
            };
            Ok(ProductOverview {
                id: out.id.into(),
                name: out.name.into(),
                thumbnail: out.thumbnail.into()?,
                price: out.price,
                overview: out.overview.into(),
                in_stock: out.in_stock,
                amount_per_unit: out.amount_per_unit,
                vendor_name: out.vendor_name.into(),
                origin: out.origin.into(),
                special_offer,
            })
        })
        .collect()
}

/// Gets products with active discounts sorted by best discounts, as defined by
/// [`discount_average`](SpecialOfferDeal::discount_average).
///
/// This returns a sequence of tuples `(product_overview, favorited)` where `favorited` indicates
/// whether the customer has marked the product as a favorite. Only visible products with units in
/// stock are considered.
///
/// # Errors
///
/// Fails if an error occurs during communication with the database.
#[server]
pub(crate) async fn best_discounts_favorited(
    customer: Id<Customer>,
    limit: usize,
    offset: usize,
) -> Result<Box<[(ProductOverview, bool)]>> {
    // TODO: Add tiebreaker for order?
    let products = query!(
        "SELECT id, name, thumbnail, price, overview, in_stock, amount_per_unit, origin,
                aso.new_price, aso.quantity1, aso.quantity2, aso.members_only,
                offers_discount(price, aso.new_price, aso.quantity1, aso.quantity2) AS discount,
                vendors.display_name AS vendor_name,
                EXISTS(
                    SELECT 1
                    FROM customer_favorites cf
                    WHERE cf.customer = $1
                      AND cf.product = products.id
                ) AS favorited
        FROM products
        WHERE visible
          AND in_stock > 0
        LEFT JOIN active_special_offers aso
               ON products.id = aso.product
        JOIN vendors
          ON products.vendor = vendors.id
        ORDER BY discount DESC
        LIMIT $2
        OFFSET $3;",
        customer,
        limit,
        offset,
    )
    .fetch_all(connection())
    .await?;

    products
        .into_iter()
        .map(|out| {
            let special_offer = if let Some(members_only) = out.members_only
                && let Some(deal) =
                    Deal::new(out.new_price, out.quantity1, out.quantity2, out.price)?
            {
                // TODO: Is this comparison too strict when considering rounding errors?
                debug_assert!(deal.discount_average(out.price) == Some(out.discount));
                Some((deal, members_only))
            } else {
                None
            };
            Ok((
                ProductOverview {
                    id: out.id.into(),
                    name: out.name.into(),
                    thumbnail: out.thumbnail.into()?,
                    price: out.price,
                    overview: out.overview.into(),
                    in_stock: out.in_stock,
                    amount_per_unit: out.amount_per_unit,
                    vendor_name: out.vendor_name.into(),
                    origin: out.origin.into(),
                    special_offer,
                },
                out.favorited,
            ))
        })
        .collect()
}

/// Gets the most recently created products.
///
/// Only visible products with units in stock are considered.
///
/// # Errors
///
/// Fails if an error occurs during communication with the database.
#[server]
pub(crate) async fn newest_products(limit: usize, offset: usize) -> Result<Box<[ProductOverview]>> {
    let products = query!(
        "SELECT id, name, thumbnail, price, overview, in_stock, amount_per_unit, origin,
                aso.new_price, aso.quantity1, aso.quantity2, aso.members_only,
                vendors.display_name AS vendor_name
        FROM products
        WHERE visible
          AND in_stock > 0
        LEFT JOIN active_special_offers aso
               ON products.id = aso.product
        JOIN vendors
          ON products.vendor = vendors.id
        ORDER BY created_at DESC
        LIMIT $1
        OFFSET $2;",
        limit,
        offset,
    )
    .fetch_all(connection())
    .await?;

    products
        .into_iter()
        .map(|out| {
            let special_offer = if let Some(members_only) = out.members_only
                && let Some(deal) =
                    Deal::new(out.new_price, out.quantity1, out.quantity2, out.price)?
            {
                Some((deal, members_only))
            } else {
                None
            };
            Ok(ProductOverview {
                id: out.id.into(),
                name: out.name.into(),
                thumbnail: out.thumbnail.into()?,
                price: out.price,
                overview: out.overview.into(),
                in_stock: out.in_stock,
                amount_per_unit: out.amount_per_unit,
                vendor_name: out.vendor_name.into(),
                origin: out.origin.into(),
                special_offer,
            })
        })
        .collect()
}

/// Gets the most recently created products.
///
/// This returns a sequence of tuples `(product_overview, favorited)` where `favorited` indicates
/// whether the customer has marked the product as a favorite. Only visible products with units in
/// stock are considered.
///
/// # Errors
///
/// Fails if:
/// - `customer` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn newest_products_favorited(
    customer: Id<Customer>,
    limit: usize,
    offset: usize,
) -> Result<Box<[(ProductOverview, bool)]>> {
    let products = query!(
        "SELECT id, name, thumbnail, price, overview, in_stock, amount_per_unit, origin,
                aso.new_price, aso.quantity1, aso.quantity2, aso.members_only,
                vendors.display_name AS vendor_name,
                EXISTS(
                    SELECT 1
                    FROM customer_favorites cf
                    WHERE cf.customer = $1
                      AND cf.product = products.id
                ) AS favorited
        FROM products
        WHERE visible
          AND in_stock > 0
        LEFT JOIN active_special_offers aso
               ON products.id = aso.product
        JOIN vendors
          ON products.vendor = vendors.id
        ORDER BY created_at DESC
        LIMIT $2
        OFFSET $3;",
        customer,
        limit,
        offset,
    )
    .fetch_all(connection())
    .await?;

    products
        .into_iter()
        .map(|out| {
            let special_offer = if let Some(members_only) = out.members_only
                && let Some(deal) =
                    Deal::new(out.new_price, out.quantity1, out.quantity2, out.price)?
            {
                Some((deal, members_only))
            } else {
                None
            };
            Ok((
                ProductOverview {
                    id: out.id.into(),
                    name: out.name.into(),
                    thumbnail: out.thumbnail.into()?,
                    price: out.price,
                    overview: out.overview.into(),
                    in_stock: out.in_stock,
                    amount_per_unit: out.amount_per_unit,
                    vendor_name: out.vendor_name.into(),
                    origin: out.origin.into(),
                    special_offer,
                },
                out.favorited,
            ))
        })
        .collect()
}

/// Gets a customer's favorites sorted by name.
///
/// Includes products out of stock, but not invisible products. Products are not sorted in any
/// particular order.
///
/// # Errors
///
/// Fails if:
/// - `customer` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn favorites(
    customer: Id<Customer>,
    limit: usize,
    offset: usize,
) -> Result<Box<[ProductOverview]>> {
    let products = query!(
        "SELECT id, name, thumbnail, price, overview, in_stock, amount_per_unit, origin,
                aso.new_price, aso.quantity1, aso.quantity2, aso.members_only,
                vendors.display_name AS vendor_name
        FROM products
        LEFT JOIN active_special_offers aso
               ON products.id = active_special_offers.product
        JOIN vendors
          ON products.vendor = vendors.id
        JOIN customer_favorites cf
          ON products.id = cf.product
        WHERE visible
          AND cf.customer = $1
        ORDER BY cf.created_at DESC
        LIMIT $2
        OFFSET $3;",
        customer,
        limit,
        offset,
    )
    .fetch_all(connection())
    .await?;

    products
        .into_iter()
        .map(|out| {
            let special_offer = if let Some(members_only) = out.members_only
                && let Some(deal) =
                    Deal::new(out.new_price, out.quantity1, out.quantity2, out.price)?
            {
                Some((deal, members_only))
            } else {
                None
            };
            Ok(ProductOverview {
                id: out.id.into(),
                name: out.name.into(),
                thumbnail: out.thumbnail.into()?,
                price: out.price,
                overview: out.overview.into(),
                in_stock: out.in_stock,
                amount_per_unit: out.amount_per_unit,
                vendor_name: out.vendor_name.into(),
                origin: out.origin.into(),
                special_offer,
            })
        })
        .collect()
}

/// Gets information about a product, for display on product pages.
///
/// If the gallery was empty, it will consist of a single copy of the thumbnail.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn product_info(product: Id<Product>) -> Result<ProductInfo> {
    let out = query!(
        "SELECT id, name, gallery, thumbnail, price, description, in_stock, amount_per_unit,
                origin, visible, p.created_at, p.updated_at,
                aso.new_price, aso.quantity1, aso.quantity2, aso.members_only,
                vendors.id AS vendor_id, vendors.display_name AS vendor_name,
                category_path(category) AS category_path,
                AVG(rating) AS average_rating, COUNT(rating) AS rating_count
        FROM products p
        WHERE id = $1
        LEFT JOIN active_special_offers aso
               ON p.id = active_special_offers.product
        JOIN vendors
          ON p.vendor = vendors.id
        LEFT JOIN ratings
               ON p.id = ratings.product;",
        product,
    )
    .fetch_one(connection())
    .await?;

    let gallery = if out.gallery.is_empty() {
        Box::new([out.thumbnail])
    } else {
        out.gallery
    };
    let special_offer = if let Some(members_only) = out.members_only
        && let Some(deal) = Deal::new(out.new_price, out.quantity1, out.quantity2, out.price)?
    {
        Some((deal, members_only))
    } else {
        None
    };
    Ok(ProductInfo {
        id: out.id.into(),
        name: out.name.into(),
        gallery,
        price: out.price,
        description: out.description.into(),
        in_stock: out.in_stock,
        category: out.category_path.into(),
        amount_per_unit: out.amount_per_unit,
        visible: out.visible,
        vendor_id: out.vendor_id.into(),
        vendor_name: out.vendor_name.into(),
        origin: out.origin.into(),
        created_at: out.created_at,
        updated_at: out.updated_at,
        rating: AverageRating::new(out.average_rating, out.rating_count)
            .expect("Invalid average rating."),
        special_offer,
    })
}

/// Gets information about a product, for display on product pages.
///
/// This returns a tuple `(product_info, own_review)` where `own_review` is the customer's own
/// review of the product, if any. If the gallery was empty, it will consist of a single copy of
/// the thumbnail.
///
/// # Errors
///
/// Fails if:
/// - `product` or `customer` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn product_info_review(
    customer: Id<Customer>,
    product: Id<Product>,
) -> Result<(ProductInfo, Option<OwnReview>)> {
    let out = query!(
        "SELECT id, name, gallery, thumbnail, price, description, in_stock, amount_per_unit,
                origin, visible, p.created_at, p.updated_at,
                aso.new_price, aso.quantity1, aso.quantity2, aso.members_only,
                vendors.id AS vendor_id, vendors.display_name AS vendor_name,
                r.id AS review_id, r.rating AS own_rating, r.created_at AS review_created_at,
                r.updated_at AS review.updated_at, r.title AS review_title,
                r.content AS review_content, TODO AS review_comments, TODO AS review_votes,
                TODO AS rating,
                TODO AS category,
        FROM products p
        WHERE id = $2
        LEFT JOIN active_special_offers aso
               ON p.id = active_special_offers.product
        JOIN vendors
          ON p.vendor = vendors.id
        JOIN reviews
          ON r.customer = $1
         AND p.id = r.product;",
        customer,
        product,
    )
    .fetch_one(connection())
    .await?;

    let gallery = if out.gallery.is_empty() {
        Box::new([out.thumbnail])
    } else {
        out.gallery
    };
    let special_offer = if let Some(members_only) = out.members_only
        && let Some(deal) = Deal::new(out.new_price, out.quantity1, out.quantity2, out.price)?
    {
        Some((deal, members_only))
    } else {
        None
    };
    let review = todo!();
    Ok((
        ProductInfo {
            id: out.id.into(),
            name: out.name.into(),
            gallery,
            price: out.price,
            description: out.description.into(),
            in_stock: out.in_stock,
            category: todo!(),
            amount_per_unit: out.amount_per_unit,
            visible: out.visible,
            vendor_id: out.vendor_id.into(),
            vendor_name: out.vendor_name.into(),
            origin: out.origin.into(),
            created_at: out.created_at,
            updated_at: out.updated_at,
            rating: todo!(),
            special_offer,
        },
        review,
    ))
}

/// Gets other products in the same category, sorted by best active discounts, then in random
/// order for fairness.
///
/// Only visible products with units in stock are considered.
///
/// # Errors
///
/// Fails if:
/// - `similar_to` is invalid.
/// - An error occurs during communication with the database.
// TODO: Consider deterministic ordering based on "popularity", for example using sales statistics.
#[server]
pub(crate) async fn similar_products(
    category: Id<Category>,
    except: Id<Product>,
    limit: usize,
    offset: usize,
) -> Result<Box<[ProductOverview]>> {
    let products = query!(
        "SELECT id, name, thumbnail, price, overview, in_stock, amount_per_unit, origin,
                aso.new_price, aso.quantity1, aso.quantity2, aso.members_only,
                offers_discount(price, aso.new_price, aso.quantity1, aso.quantity2) AS discount,
                vendors.display_name AS vendor_name
        FROM products
        WHERE visible
          AND category = $1
          AND in_stock > 0
          AND id != $2
        LEFT JOIN active_special_offers aso
               ON products.id = active_special_offers.product
        JOIN vendors
          ON products.vendor = vendors.id
        ORDER BY discount DESC, random()
        LIMIT $3
        OFFSET $4;",
        category,
        except,
        limit,
        offset,
    )
    .fetch_all(connection())
    .await?;

    products
        .into_iter()
        .map(|out| {
            let special_offer = if let Some(members_only) = out.members_only
                && let Some(deal) =
                    Deal::new(out.new_price, out.quantity1, out.quantity2, out.price)?
            {
                Some((deal, members_only))
            } else {
                None
            };
            Ok(ProductOverview {
                id: out.id.into(),
                name: out.name.into(),
                thumbnail: out.thumbnail.into()?,
                price: out.price,
                overview: out.overview.into(),
                in_stock: out.in_stock,
                amount_per_unit: out.amount_per_unit,
                vendor_name: out.vendor_name.into(),
                origin: out.origin.into(),
                special_offer,
            })
        })
        .collect()
}

/// Gets other products in the same category, sorted by best active discounts, then in random
/// order for fairness.
///
/// This returns a sequence of tuples `(product_overview, favorited)` where `favorited` indicates
/// whether the customer has marked the product as a favorite. Only visible products with units in
/// stock are considered.
///
/// # Errors
///
/// Fails if:
/// - `similar_to` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn similar_products_favorited(
    customer: Id<Customer>,
    category: Id<Category>,
    except: Id<Product>,
    limit: usize,
    offset: usize,
) -> Result<Box<[ProductOverview]>> {
    let products = query!(
        "SELECT id, name, thumbnail, price, overview, in_stock, amount_per_unit, origin,
                aso.new_price, aso.quantity1, aso.quantity2, aso.members_only,
                offers_discount(price, aso.new_price, aso.quantity1, aso.quantity2) AS discount,
                vendors.display_name AS vendor_name,
                EXISTS(
                    SELECT 1
                    FROM customer_favorites cf
                    WHERE cf.customer = $1
                      AND cf.product = products.id
                ) AS favorited
        FROM products
        WHERE visible
          AND category = $2
          AND in_stock > 0
          AND id != $3
        LEFT JOIN active_special_offers aso
               ON products.id = active_special_offers.product
        JOIN vendors
          ON products.vendor = vendors.id
        ORDER BY discount DESC, random()
        LIMIT $4
        OFFSET $5;",
        customer,
        category,
        except,
        limit,
        offset,
    )
    .fetch_all(connection())
    .await?;

    products
        .into_iter()
        .map(|out| {
            let special_offer = if let Some(members_only) = out.members_only
                && let Some(deal) =
                    Deal::new(out.new_price, out.quantity1, out.quantity2, out.price)?
            {
                Some((deal, members_only))
            } else {
                None
            };
            Ok((
                ProductOverview {
                    id: out.id.into(),
                    name: out.name.into(),
                    thumbnail: out.thumbnail.into()?,
                    price: out.price,
                    overview: out.overview.into(),
                    in_stock: out.in_stock,
                    amount_per_unit: out.amount_per_unit,
                    vendor_name: out.vendor_name.into(),
                    origin: out.origin.into(),
                    special_offer,
                },
                out.favorited,
            ))
        })
        .collect()
}

/// Gets reviews for a product, possibly excluding the one made by a specific customer if it
/// exists. The inteded use is in conjunction with [`product_info`](Self::product_info), having
/// that fetch the current user's own review, and this exclude it.
///
/// # Errors
///
/// Fails if:
/// - `product` or `except_by` (if [`Some`]) is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn product_reviews(
    except_by: Option<Id<Customer>>,
    product: Id<Product>,
    limit: usize,
    offset: usize,
) -> Result<Box<[ProductReview]>> {
    todo!()
}

// TODO: Do all of the functions to update a single row need to verify that a single row is
// updated? That is, if the IDs these queries reference don't exist, does execution fail silently
// or return an error (the desired outcome)?

/// Puts `number` units of a product in a customer's shopping cart, overriding any number
/// already there. Setting `number = 0` removes the product from the shopping cart.
///
/// # Errors
///
/// Fails if:
/// - `customer` or `product` is invalid.
/// - `number > i32::MAX`.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn set_in_shopping_cart(
    customer: Id<Customer>,
    product: Id<Product>,
    number: u32,
) -> Result<()> {
    if number == 0 {
        query!(
            "DELETE FROM shopping_cart_items
            WHERE customer = $1
              AND product = $2;",
            customer,
            product,
        )
    } else if let Ok(number) = i32::try_from(number) {
        query!(
            "INSERT INTO shopping_cart_items (customer, product, number)
            VALUES ($1, $2, $3)
            ON CONFLICT (customer, product) DO UPDATE
            SET number = EXCLUDED.number;",
            customer,
            product,
            number,
        )
    } else {
        todo!()
    }
    .execute(connection())
    .await
    .map(|_| ())
}

/// Removes all products from a customer's cart that have been deleted since addition to the cart.
///
/// # Errors
///
/// Fails if:
/// - `customer` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn remove_deleted_from_cart(customer: Id<Customer>) -> Result<()> {
    query!(
        "DELETE FROM shopping_cart_items
        WHERE customer = $1
          AND product IS NULL;",
        customer,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Sets the "favorite" status of a product for a customer.
///
/// # Errors
///
/// Fails if:
/// - `customer` or `product` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn set_favorite(
    customer: Id<Customer>,
    product: Id<Product>,
    favorite: bool,
) -> Result<()> {
    if favorite {
        query!(
            "INSERT INTO customer_favorites (customer, product)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING;",
            customer,
            product,
        )
    } else {
        query!(
            "DELETE FROM customer_favorites
            WHERE customer = $1
              AND product = $2;",
            customer,
            product,
        )
    }
    .execute(connection())
    .await
    .map(|_| ())
}

/// Sets a customer's rating on a product, if any.
///
/// # Errors
///
/// Fails if:
/// - `customer` or `product` is invalid.
/// - Attempting to remove rating (set to `None`) while having a review on the product.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn set_rating(
    customer: Id<Customer>,
    product: Id<Product>,
    rating: Option<Rating>,
) -> Result<()> {
    if let Some(rating) = rating {
        query!(
            "INSERT INTO ratings (customer, product, rating)
            VALUES ($1, $2, $3)
            ON CONFLICT (customer, product) DO UPDATE
            SET rating = EXCLUDED.rating;",
            customer,
            product,
            rating,
        )
    } else {
        query!(
            "DELETE FROM ratings
            WHERE customer = $1
              AND product = $2;",
            customer,
            product,
        )
    }
    .execute(connection())
    .await
    .map(|_| ())
}

/// Creates a review on a product.
///
/// # Errors
///
/// Fails if:
/// - `customer` or `product` is invalid.
/// - The customer already has a review on the product.
/// - The customer has not placed a rating on the product.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn create_review(
    customer: Id<Customer>,
    product: Id<Product>,
    title: Box<str>,
    content: Box<str>,
) -> Result<()> {
    query!(
        "INSERT INTO reviews (customer, product, title, content)
        VALUES ($1, $2, $3, $4);",
        customer,
        product,
        title,
        content,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Updates a review.
///
/// # Errors
///
/// Fails if:
/// - `review` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn update_review(
    review: Id<Review>,
    title: Box<str>,
    content: Box<str>,
) -> Result<()> {
    query!(
        "UPDATE reviews
        SET title = $2,
            content = $3
        WHERE review = $1;",
        review,
        title,
        content,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Deletes a review and all comments on it.
///
/// As this action cannot be undone and might delete a large number of comments, it should be
/// associated with a proper warning in the frontend.
///
/// # Errors
///
/// Fails if:
/// - `review` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn delete_review(review: Id<Review>) -> Result<()> {
    query!(
        "DELETE FROM reviews
        WHERE id = $1;",
        review,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Creates a comment on a review.
///
/// # Errors
///
/// Fails if:
/// - `parent` or `user` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn create_comment(
    user: Id<User>,
    parent: Id<Review>,
    content: Box<str>,
) -> Result<()> {
    query!(
        "INSERT INTO comments (user, review, content)
        VALUES ($1, $2, $3);",
        user,
        parent,
        content,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Creates a comment on another comment.
///
/// # Errors
///
/// Fails if:
/// - `parent` or `user` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn create_reply(
    user: Id<User>,
    parent: Id<Comment>,
    content: Box<str>,
) -> Result<()> {
    query!(
        "INSERT INTO comments (user, review, parent, content)
        VALUES (
            $1,
            (
                SELECT review
                FROM comments
                WHERE id = $1
            ),
            $2,
            $3
        );",
        user,
        parent,
        content,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Deletes a comment and all replies to it.
///
/// # Errors
///
/// Fails if:
/// - `comment` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn delete_comment(comment: Id<Comment>) -> Result<()> {
    query!(
        "DELETE FROM comments
        WHERE id = $1;",
        comment,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Sets the customer's vote status on a review. Setting `vote = None` removes the vote.
///
/// # Errors
///
/// Fails if:
/// - `customer` or `review` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn set_vote_review(
    customer: Id<Customer>,
    review: Id<Review>,
    vote: Option<Vote>,
) -> Result<()> {
    if let Some(vote) = vote {
        query!(
            "INSERT INTO review_votes (customer, review, grade)
            VALUES ($1, $2, $3);",
            customer,
            review,
            vote,
        )
    } else {
        query!(
            "DELETE FROM review_votes
            WHERE customer = $1
              AND review = $2;",
            customer,
            review,
        )
    }
    .execute(connection())
    .await
    .map(|_| ())
}

/// Sets the user's vote status on a comment. Setting `vote = None` removes the vote.
///
/// # Errors
///
/// Fails if:
/// - `user` or `comment` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn set_vote_comment(
    user: Id<Customer>,
    comment: Id<Comment>,
    vote: Option<Vote>,
) -> Result<()> {
    if let Some(vote) = vote {
        query!(
            "INSERT INTO comment_votes (customer, comment, grade)
            VALUES ($1, $2, $3);",
            customer,
            comment,
            vote,
        )
    } else {
        query!(
            "DELETE FROM comment_votes
            WHERE customer = $1
              AND comment = $2;",
            customer,
            comment,
        )
    }
    .execute(connection())
    .await
    .map(|_| ())
}

/// Sets a customer's profile picture.
///
/// # Errors
///
/// Fails if:
/// - `customer` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn set_customer_profile_picture(customer: Id<Customer>, url: Url) -> Result<()> {
    query!(
        "UPDATE customers
        SET profile_picture = $2
        WHERE id = $1;",
        customer,
        url,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Sets a vendor's profile picture.
///
/// # Errors
///
/// Fails if:
/// - `vendor` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn set_vendor_profile_picture(vendor: Id<Vendor>, url: Url) -> Result<()> {
    query!(
        "UPDATE vendors
        SET profile_picture = $2
        WHERE id = $1;",
        customer,
        url,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Sets a user's username.
///
/// # Errors
///
/// Fails if:
/// - `user` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn set_username(user: Id<User>, username: Username) -> Result<()> {
    query!(
        "UPDATE users
        SET username = $2
        WHERE id = $1;",
        user,
        username,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Sets a user's email.
///
/// # Errors
///
/// Fails if:
/// - `user` is invalid.
/// - An invalid email is provided.
/// - An error occurs during communication with the database.
// TODO: Use a proper email library and replace the argument type. Update database-side validation
// if necessary. Currently, validation is handled in the database only.
#[server]
pub(crate) async fn set_email(user: Id<User>, email: Box<str>) -> Result<()> {
    query!(
        "UPDATE users
        SET email = $2
        WHERE id = $1;",
        user,
        email,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Sets a vendor's display name.
///
/// # Errors
///
/// Fails if:
/// - `vendor` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn set_vendor_display_name(
    vendor: Id<Vendor>,
    display_name: Box<str>,
) -> Result<()> {
    query!(
        "UPDATE vendors
        SET display_name = $2
        WHERE id = $1;",
        vendor,
        display_name,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Sets a vendor's description.
///
/// # Errors
///
/// Fails if:
/// - `vendor` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn set_vendor_description(
    vendor: Id<Vendor>,
    description: Box<str>,
) -> Result<()> {
    query!(
        "UPDATE vendors
        SET description = $2
        WHERE id = $1;",
        vendor,
        description,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Gets reviews made by a customer.
///
/// # Errors
///
/// Fails if:
/// - `customer` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn customer_reviews(
    customer: Id<Customer>,
    limit: usize,
    offset: usize,
) -> Result<Box<[CustomerReview]>> {
    query!(
        "SELECT product, thumbnail, rating, title, content,
                product.name AS product_name
        FROM reviews
        WHERE customer = $1
        JOIN products
          ON reviews.product = products.id
        LIMIT $2
        OFFSET $3;",
        customer,
        limit,
        offset
    )
    .fetch_all(connection())
    .await
    .map(|reviews| {
        reviews
            .into_iter()
            .map(|out| CustomerReview {
                product: out.product.into(),
                thumbnail: out.thumbnail.into(),
                product_name: out.product_name.into(),
                rating: Rating::new(out.rating).expect("Invalid rating."),
                title: out.title.into(),
                content: out.content.into(),
            })
            .collect()
    })
}

/// Gets orders made by a customer.
///
/// # Errors
///
/// Fails if:
/// - `customer` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn orders(
    customer: Id<Customer>,
    limit: usize,
    offset: usize,
) -> Result<Box<[Order]>> {
    let raw = query!(
        "SELECT time, paid, special_offer_used, amount_per_unit, number,
                p.name AS product_name, p.thumbnail, p.origin AS product_origin,
                vendors.display_name AS vendor_name
        FROM orders
        WHERE customer = $1
        JOIN products p
          ON orders.product = p.id
        JOIN vendors
          ON p.vendor = vendors.id
        ORDER BY time DESC
        LIMIT $2
        OFFSET $3;",
        customer,
        limit,
        offset
    )
    .fetch_all(connection())
    .await?;

    debug_assert!(
        raw.is_sorted_by_key(|purchase| Reverse(purchase.time)),
        "Rows not sorted."
    );

    let orders = raw
        .into_iter()
        .map(|out| {
            let purchase = Purchase {
                paid: out.paid,
                special_offer_used: out.special_offer_used,
                amount_per_unit: out.amount,
                number: out.number,
                product_name: out.product_name.into(),
                thumbnail: out.thumbnail.into(),
                product_overview: out.product_overview.into(),
                product_origin: out.product_origin.into(),
                vendor_name: out.vendor_name.into(),
            };
            (out.time, purchase)
        })
        .fold(Vec::new(), |mut acc, (time, purchase)| {
            if let Some(last) = acc.last_mut()
                && time == last.time
            {
                last.purchases.push(purchase);
            } else {
                acc.push(Order {
                    time,
                    purchases: vec![purchase],
                });
            }
            acc
        })
        .map(Into::into);
    Ok(orders)
}

/// Gets products owned by a vendor.
///
/// Invisible products should be included if the accessor is that same vendor.
///
/// # Errors
///
/// Fails if:
/// - `vendor` is invalid.
/// - An error occurs during communication with the database.
// TODO: Since all product overviews will have the same vendor, should the vendor's name be
// omitted? Currently, since the vendor's name will be known when navigating to their page, it is
// accepted as a parameter and cloned instead of fetched from the database, but even better might
// be to create a `ProductOverview` but without vendor information entirely. The same would then
// also be done with all functions that return "favorited" statuses.
#[server]
pub(crate) async fn vendor_products(
    vendor: Id<Vendor>,
    vendor_name: Box<str>,
    limit: usize,
    offset: usize,
    include_invisible: bool,
) -> Result<Box<[ProductOverview]>> {
    let products = query!(
        "SELECT id, name, thumbnail, price, overview, in_stock, amount_per_unit, origin,
                aso.new_price, aso.quantity1, aso.quantity2, aso.members_only
        FROM products
        WHERE (visible OR $4)
          AND vendor = $1
          AND in_stock > 0
        LEFT JOIN active_special_offers aso
               ON products.id = aso.product
        ORDER BY created_at DESC
        LIMIT $2
        OFFSET $3;",
        vendor,
        limit,
        offset,
        include_invisible,
    )
    .fetch_all(connection())
    .await?;

    products
        .into_iter()
        .map(|out| {
            let special_offer = if let Some(members_only) = out.members_only
                && let Some(deal) =
                    Deal::new(out.new_price, out.quantity1, out.quantity2, out.price)?
            {
                // TODO: Is this comparison too strict when considering rounding errors?
                debug_assert!(deal.discount_average(out.price) == Some(out.discount));
                Some((deal, members_only))
            } else {
                None
            };
            Ok(ProductOverview {
                id: out.id.into(),
                name: out.name.into(),
                thumbnail: out.thumbnail.into()?,
                price: out.price,
                overview: out.overview.into(),
                in_stock: out.in_stock,
                amount_per_unit: out.amount_per_unit,
                vendor_name: out.vendor_name.into(),
                origin: out.origin.into(),
                special_offer,
            })
        })
        .collect()
}

/// Gets products owned by a vendor.
///
/// This returns a sequence of tuples `(product_overview, favorited)` where `favorited` indicates
/// whether the customer has marked the product as a favorite. Only visible products with units in
/// stock are considered.
///
/// # Errors
///
/// Fails if:
/// - `vendor` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn vendor_products_favorited(
    customer: Id<Customer>,
    vendor: Id<Vendor>,
    vendor_name: Box<str>,
    limit: usize,
    offset: usize,
    include_invisible: bool,
) -> Result<Box<[(ProductOverview, bool)]>> {
    let products = query!(
        "SELECT id, name, thumbnail, price, overview, in_stock, amount_per_unit, origin,
                aso.new_price, aso.quantity1, aso.quantity2, aso.members_only,
                EXISTS(
                    SELECT 1
                    FROM customer_favorites cf
                    WHERE cf.customer = $1
                      AND cf.product = products.id
                ) AS favorited
        FROM products
        WHERE (visible OR $5)
          AND vendor = $2
          AND in_stock > 0
        LEFT JOIN active_special_offers aso
               ON products.id = aso.product
        ORDER BY created_at DESC
        LIMIT $3
        OFFSET $4;",
        customer,
        vendor,
        limit,
        offset,
        include_invisible,
    )
    .fetch_all(connection())
    .await?;

    products
        .into_iter()
        .map(|out| {
            let special_offer = if let Some(members_only) = out.members_only
                && let Some(deal) =
                    Deal::new(out.new_price, out.quantity1, out.quantity2, out.price)?
            {
                // TODO: Is this comparison too strict when considering rounding errors?
                debug_assert!(deal.discount_average(out.price) == Some(out.discount));
                Some((deal, members_only))
            } else {
                None
            };
            Ok((
                ProductOverview {
                    id: out.id.into(),
                    name: out.name.into(),
                    thumbnail: out.thumbnail.into()?,
                    price: out.price,
                    overview: out.overview.into(),
                    in_stock: out.in_stock,
                    amount_per_unit: out.amount_per_unit,
                    vendor_name: out.vendor_name.into(),
                    origin: out.origin.into(),
                    special_offer,
                },
                out.favorited,
            ))
        })
        .collect()
}

/// Create a new product.
///
/// # Errors
///
/// Fails if:
/// - `vendor` or `category` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn create_product(
    vendor: Id<Vendor>,
    name: Box<str>,
    thumbnail: Url,
    gallery: Box<[Url]>,
    price: Decimal,
    overview: Box<str>,
    description: Box<str>,
    category: Id<Category>,
    amount_per_unit: Option<Amount>,
    origin: Box<str>,
) -> Result<()> {
    query!(
        "INSERT INTO products (vendor, name, thumbnail, gallery, price, overview,
                               description, category, amount_per_unit, origin)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10);",
        vendor,
        name,
        thumbnail,
        gallery,
        price,
        overview,
        description,
        category,
        amount_per_unit,
        origin,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Set the name of a product.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn set_product_name(product: Id<Product>, name: Box<str>) -> Result<()> {
    query!(
        "UPDATE products
        SET name = $2
        WHERE id = $1;",
        product,
        name,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Set the thumbnail of a product.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn set_thumbnail(product: Id<Product>, url: Url) -> Result<()> {
    query!(
        "UPDATE products
        SET thumbnail = $2
        WHERE id = $1;",
        product,
        url,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Get the gallery of a product.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn gallery(product: Id<Product>) -> Result<Box<[Url]>> {
    query!(
        "SELECT gallery
        FROM products
        WHERE id = $1;",
        product,
    )
    .fetch_one(connection())
    .await
}

/// Set the gallery of a product.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn set_gallery(product: Id<Product>, gallery: Box<[Url]>) -> Result<()> {
    query!(
        "UPDATE products
        SET gallery = $2
        WHERE id = $1;",
        product,
        gallery,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Append to the gallery of a product.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn add_to_gallery(product: Id<Product>, additions: Box<[Url]>) -> Result<()> {
    query!(
        "UPDATE products
        SET gallery = gallery || $2
        WHERE id = $1;",
        product,
        additions,
    )
    .execute(connection())
    .await
    .map(|_| ())
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
pub(crate) async fn set_price(product: Id<Product>, price: Decimal) -> Result<()> {
    query!(
        "UPDATE products
        SET price = $2
        WHERE id = $1;",
        product,
        price,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Set the overview of a product.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn set_overview(product: Id<Product>, overview: Box<str>) -> Result<()> {
    query!(
        "UPDATE products
        SET overview = $2
        WHERE id = $1;",
        product,
        overview,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Set the description of a product.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn set_description(product: Id<Product>, description: Box<str>) -> Result<()> {
    query!(
        "UPDATE products
        SET description = $2
        WHERE id = $1;",
        product,
        description,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Set the category of a product.
///
/// # Errors
///
/// Fails if:
/// - `product` or `category` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn set_category(product: Id<Product>, category: Id<Category>) -> Result<()> {
    query!(
        "UPDATE products
        SET category = $2
        WHERE id = $1;",
        product,
        category,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Set the amount per unit of a product.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn set_amount(product: Id<Product>, amount: Option<Amount>) -> Result<()> {
    query!(
        "UPDATE products
        SET amount = $2
        WHERE id = $1;",
        product,
        amount,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Set the origin of a product.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn set_origin(product: Id<Product>, origin: Box<str>) -> Result<()> {
    query!(
        "UPDATE products
        SET origin = $2
        WHERE id = $1;",
        product,
        origin,
    )
    .execute(connection())
    .await
    .map(|_| ())
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
pub(crate) async fn add_stock(
    product: Id<Product>,
    expiry: Option<NaiveDate>,
    number: NonZeroU32,
) -> Result<()> {
    if let Some(expiry) = expiry
        && expiry < Utc::now().date_naive()
    {
        todo!()
    }

    let number = i32::try_from(number.get())?;

    let mut tx = connection().begin().await?;

    query!(
        "UPDATE products
        SET in_stock = in_stock + $2
        WHERE id = $1;",
        product,
        number
    )
    .execute(tx)
    .await?
    .map(|_| ());

    query!(
        "INSERT INTO expiries (product, expiry, number)
        VALUES ($1, $2, $3);",
        product,
        expiry,
        number
    )
    .execute(tx)
    .await?
    .map(|_| ());

    tx.commit().await
}

/// Set the visibility of a product.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn set_visibility(product: Id<Product>, visible: bool) -> Result<()> {
    // TODO: This might have to change. Currently, it is possible for a customer to have
    // now-invisible products in their cart and as such purchase them. Should they be removed from
    // all carts upon being made invisible, and if so, should customers be made aware of this?
    query!(
        "UPDATE products
        SET visible = $2
        WHERE id = $1;",
        product,
        visible,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Mark a user as deleted.
///
/// This deletes their reviews, shopping cart, favorites, votes and more (if they were a customer),
/// products (if they were a vendor), as well as their comments. Order history is kept if they
/// were a customer.
///
/// # Errors
///
/// Fails if:
/// - `user` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn delete_user(user: Id<User>) -> Result<()> {
    let mut tx = connection().begin().await?;

    // NOTE: Soft deletion. Possible corresponding row in role-specific table is also kept.
    query!(
        "UPDATE users
        SET deleted = true
        WHERE id = $1;",
        user
    )
    .execute(tx)
    .await?
    .map(|_| ());

    // Does nothing if user is not a vendor.
    query!(
        "DELETE FROM products
        WHERE vendor = $1;",
        user
    )
    .execute(tx)
    .await?
    .map(|_| ());

    // Does nothing if user is not a customer.
    query!(
        "DELETE FROM special_offer_uses
        WHERE customer = $1;",
        user
    )
    .execute(tx)
    .await?
    .map(|_| ());

    // Does nothing if user is not a customer.
    // NOTE: Must be done before deleting rating.
    query!(
        "DELETE FROM reviews
        WHERE customer = $1;",
        user
    )
    .execute(tx)
    .await?
    .map(|_| ());

    // Does nothing if user is not a customer.
    query!(
        "DELETE FROM ratings
        WHERE customer = $1;",
        user
    )
    .execute(tx)
    .await?
    .map(|_| ());

    // Does nothing if user is not a customer.
    query!(
        "DELETE FROM review_votes
        WHERE customer = $1;",
        user
    )
    .execute(tx)
    .await?
    .map(|_| ());

    query!(
        "DELETE FROM comments
        WHERE user_id = $1;",
        user
    )
    .execute(tx)
    .await?
    .map(|_| ());

    // Does nothing if user is not a customer.
    query!(
        "DELETE FROM comment_votes
        WHERE customer = $1;",
        user
    )
    .execute(tx)
    .await?
    .map(|_| ());

    // Does nothing if user is not a customer.
    query!(
        "DELETE FROM shopping_cart_items
        WHERE customer = $1;",
        user
    )
    .execute(tx)
    .await?
    .map(|_| ());

    // Does nothing if user is not a customer.
    query!(
        "DELETE FROM customer_favorites
        WHERE customer = $1;",
        user
    )
    .execute(tx)
    .await?
    .map(|_| ());

    tx.commit().await
}

/// Create a category.
///
/// # Errors
///
/// Fails if:
/// - `parent` (if [`Some`]) is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn create_category(parent: Option<Id<Category>>, name: Box<str>) -> Result<()> {
    query!(
        "INSERT INTO categories (parent, name)
        VALUES ($1, $2);",
        parent,
        name
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Delete a category and all of its subcategories.
///
/// # Errors
///
/// Fails if:
/// - `category` is invalid.
/// - Any products belong to the category.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn delete_category(category: Id<Category>) -> Result<()> {
    query!(
        "DELETE FROM categories
        WHERE id = $1;",
        category
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Create a special offer for a product.
///
/// Special offers with an end time of `None` must be deleted or otherwise disabled manually.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - `valid_until` is in the past.
/// - The special offer overlaps with an existing one.
/// - The special offer does not actually provide a discount compared to the current price.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn create_special_offer(
    product: Id<Product>,
    deal: Deal,
    member_only: bool,
    limit_per_customer: Option<NonZeroU32>,
    valid_from: NaiveDateTime,
    valid_until: Option<NaiveDateTime>,
) -> Result<()> {
    if let Some(valid_until) = valid_until
        && valid_until < Utc::now().naive_utc()
    {
        todo!()
    }

    let (new_price, quantity1, quantity2) = deal.database_repr().ok_or(todo!())?;

    query!(
        "INSERT INTO special_offers (product, members_only, limit_per_customer, valid_from,
                                     valid_until, new_price, quantity1, quantity2)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8);",
        product,
        members_only,
        limit_per_customer,
        valid_from,
        valid_until,
        new_price,
        quantity1,
        quantity2,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Deletes a special offer.
///
/// # Errors
///
/// Fails if:
/// - `special_offer` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn delete_special_offer(special_offer: Id<SpecialOffer>) -> Result<()> {
    query!(
        "DELETE FROM special_offers
        WHERE id = $1;",
        special_offer
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Sets the limit per customer of a special offer.
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
pub(crate) async fn set_special_offer_limit(
    special_offer: Id<SpecialOffer>,
    limit_per_customer: NonZeroU32,
) -> Result<()> {
    let limit_per_customer = i32::try_from(limit_per_customer.get())?;

    query!(
        "UPDATE special_offers
        SET limit_per_customer = $2
        WHERE id = $1;",
        special_offer,
        limit_per_customer,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Sets the "members only"-status of a special offer.
///
/// # Errors
///
/// Fails if:
/// - `special_offer` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn set_special_offer_members_only(
    special_offer: Id<SpecialOffer>,
    members_only: bool,
) -> Result<()> {
    query!(
        "UPDATE special_offers
        SET members_only = $2
        WHERE id = $1;",
        special_offer,
        members_only,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Sets the start time of a special offer.
///
/// # Errors
///
/// Fails if:
/// - `special_offer` is invalid.
/// - `valid_from` is in the past (set it to now if the intent is to activate it).
/// - The special offer now overlaps with an existing one.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn set_special_offer_start(
    special_offer: Id<SpecialOffer>,
    valid_from: NaiveDateTime,
) -> Result<()> {
    if valid_from < Utc::now().naive_utc() {
        todo!()
    }

    query!(
        "UPDATE special_offers
        SET valid_from = $2
        WHERE id = $1;",
        special_offer,
        valid_from,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Sets the end time of a special offer.
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
pub(crate) async fn set_special_offer_end(
    special_offer: Id<SpecialOffer>,
    valid_until: Option<NaiveDateTime>,
) -> Result<()> {
    if let Some(valid_until) = valid_until
        && valid_until < Utc::now().naive_utc()
    {
        todo!()
    }

    query!(
        "UPDATE special_offers
        SET valid_until = $2
        WHERE id = $1;",
        special_offer,
        valid_until,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Sets the deal of a special offer.
///
/// # Errors
///
/// Fails if:
/// - `special_offer` is invalid.
/// - The special offer does not actually provide a discount.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn set_special_offer_deal(
    special_offer: Id<SpecialOffer>,
    deal: Deal,
) -> Result<()> {
    let (new_price, quantity1, quantity2) = deal.database_repr().ok_or(todo!())?;

    query!(
        "UPDATE special_offers
        SET new_price = $2,
            quantity1 = $3,
            quantity2 = $4
        WHERE id = $1;",
        special_offer,
        new_price,
        quantity1,
        quantity2,
    )
    .execute(connection())
    .await
    .map(|_| ())
}

/// Completes an order for a customer, emptying their shopping cart.
///
/// # Errors
///
/// Fails if:
/// - `customer` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub(crate) async fn checkout(customer: Id<Customer>) -> Result<()> {
    let mut tx = connection().begin().await?;

    todo!();

    tx.commit().await
}
