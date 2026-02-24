//! Database functions to interact with reviews and comments.

use crate::database::{
    Comment, Customer, Id, Product, ProfilePicture, Rating, Review, Url, User, Username, Vendor,
    Vote,
};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;
#[cfg(feature = "server")]
use {
    crate::database::{QueryResultExt, RawId, Role, connection},
    sqlx::{query, query_as},
    std::cmp::Reverse,
    tokio::task::spawn,
};

/// A review of a product, for display on product pages.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductReview {
    /// The ID of the review.
    pub id: Id<Review>,
    /// The ID of the authoring customer.
    pub customer: Id<Customer>,
    /// The username of the authoring customer.
    pub username: Username,
    /// The profile picture of the authoring customer.
    pub profile_picture: ProfilePicture,
    /// The given rating of the product.
    pub rating: Rating,
    /// When the review was created.
    pub created_at: PrimitiveDateTime,
    /// When the review was last updated.
    pub updated_at: PrimitiveDateTime,
    /// The title of the review.
    pub title: Box<str>,
    /// The content of the review.
    pub content: Box<str>,
    /// Comment trees on the review.
    pub comments: Vec<CommentTree>,
    /// The sum of all votes on the review, adding 1 per like and subtracting 1 per dislike.
    pub sum_votes: i64,
    /// The customer's own vote, if any. Value is unspecified if a customer ID was not provided.
    pub own_vote: Option<Vote>,
}

/// A review of a product placed by a known customer, for display on product pages.
///
/// Note that the rating is included with [`ProductInfo`](crate::database::products::ProductInfo),
/// as the customer might have placed a rating without writing a review.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnReview {
    /// The ID of the review.
    pub id: Id<Review>,
    /// The given rating of the product.
    pub rating: Rating,
    /// When the review was created.
    pub created_at: PrimitiveDateTime,
    /// When the review was last updated.
    pub updated_at: PrimitiveDateTime,
    /// The title of the review.
    pub title: Box<str>,
    /// The content of the review.
    pub content: Box<str>,
    /// Comment trees on the review.
    pub comments: Vec<CommentTree>,
    /// The sum of all votes on the review, adding 1 per like and subtracting 1 per dislike.
    pub sum_votes: i64,
}

/// A comment with its replies in a tree.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommentTree {
    /// The ID of the comment.
    pub id: Id<Comment>,
    /// The username of the author.
    pub username: Username,
    /// The role of the author, see [`CommentRole`] for details.
    pub role: CommentRole,
    /// The content of the comment.
    pub content: Box<str>,
    /// When the comment was created.
    pub created_at: PrimitiveDateTime,
    /// When the comment was last updated.
    pub updated_at: PrimitiveDateTime,
    /// The sum of all votes on the review, adding 1 per like and subtracting 1 per dislike.
    pub sum_votes: i64,
    /// The customer's own vote, if any. Value is unspecified if a customer ID was not provided.
    pub own_vote: Option<Vote>,
    /// All direct replies to the comment.
    pub replies: Vec<Self>,
}

/// The role of the user placing a comment, to determine whether a special badge should be
/// displayed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[expect(
    variant_size_differences,
    reason = "Difference is negligible and these are not intended to be stored in large quantities."
)]
pub enum CommentRole {
    /// The author is a user. The author of the review should get a badge.
    User {
        /// Whether the user is the original poster of the review.
        author: bool,
    },
    /// The user is a vendor (ID provided). The owner of the product should get a badge.
    Vendor(Id<Vendor>),
    /// The user is a site administrator. Administrators should always get a badge.
    Administrator,
}

#[cfg(feature = "server")]
struct ReviewRepr {
    id: RawId,
    customer: RawId,
    username: String,
    profile_picture: String,
    rating: i32,
    created_at: PrimitiveDateTime,
    updated_at: PrimitiveDateTime,
    title: String,
    content: String,
    sum_votes: i64,
}

#[cfg(feature = "server")]
impl From<ReviewRepr> for ProductReview {
    fn from(
        ReviewRepr {
            id,
            customer,
            username,
            profile_picture,
            rating,
            created_at,
            updated_at,
            title,
            content,
            sum_votes,
        }: ReviewRepr,
    ) -> Self {
        Self {
            id: id.into(),
            customer: customer.into(),
            username: Username::new(username.into()).expect("Invalid username."),
            profile_picture: ProfilePicture::new(profile_picture.into()),
            rating: Rating::new(rating as u8).expect("Invalid rating."),
            created_at,
            updated_at,
            title: title.into(),
            content: content.into(),
            comments: Vec::new(),
            sum_votes,
            own_vote: None,
        }
    }
}

#[cfg(feature = "server")]
#[expect(dead_code, reason = "TODO")]
struct CommentRepr {
    id: RawId,
    parent: Option<RawId>,
    review: RawId,
    user_id: RawId,
    username: String,
    customer_pfp: Option<String>,
    vendor_pfp: Option<String>,
    role: Role,
    content: String,
    created_at: PrimitiveDateTime,
    updated_at: PrimitiveDateTime,
    sum_votes: i64,
}

#[cfg(feature = "server")]
struct OwnReviewRepr {
    id: RawId,
    rating: i32,
    created_at: PrimitiveDateTime,
    updated_at: PrimitiveDateTime,
    title: String,
    content: String,
    sum_votes: i64,
}

#[cfg(feature = "server")]
impl From<OwnReviewRepr> for OwnReview {
    fn from(
        OwnReviewRepr {
            id,
            rating,
            created_at,
            updated_at,
            title,
            content,
            sum_votes,
        }: OwnReviewRepr,
    ) -> Self {
        Self {
            id: id.into(),
            rating: Rating::new(rating as u8).expect("Invalid rating."),
            created_at,
            updated_at,
            title: title.into(),
            content: content.into(),
            comments: Vec::new(),
            sum_votes,
        }
    }
}

#[cfg(feature = "server")]
struct OtherReviewRepr {
    id: RawId,
    customer: RawId,
    username: String,
    profile_picture: String,
    rating: i32,
    created_at: PrimitiveDateTime,
    updated_at: PrimitiveDateTime,
    title: String,
    content: String,
    sum_votes: i64,
    own_vote: Option<Vote>,
}

#[cfg(feature = "server")]
impl From<OtherReviewRepr> for ProductReview {
    fn from(
        OtherReviewRepr {
            id,
            customer,
            username,
            profile_picture,
            rating,
            created_at,
            updated_at,
            title,
            content,
            sum_votes,
            own_vote,
        }: OtherReviewRepr,
    ) -> Self {
        Self {
            id: id.into(),
            customer: customer.into(),
            username: Username::new(username.into()).expect("Invalid username."),
            profile_picture: ProfilePicture::new(profile_picture.into()),
            rating: Rating::new(rating as u8).expect("Invalid rating."),
            created_at,
            updated_at,
            title: title.into(),
            content: content.into(),
            comments: Vec::new(),
            sum_votes,
            own_vote,
        }
    }
}

#[cfg(feature = "server")]
#[expect(dead_code, reason = "TODO")]
struct CommentReprCustomer {
    id: RawId,
    parent: Option<RawId>,
    review: RawId,
    user_id: RawId,
    username: String,
    customer_pfp: Option<String>,
    vendor_pfp: Option<String>,
    role: Role,
    content: String,
    created_at: PrimitiveDateTime,
    updated_at: PrimitiveDateTime,
    sum_votes: i64,
    own_vote: Option<Vote>,
}

/// Construct a [`ProfilePicture`] from its role-dependent representation in the database.
///
/// # Panics
///
/// Panics if the values do not uphold any of the database's invariants.
#[cfg(feature = "server")]
#[expect(clippy::unreachable, reason = "Database validation only.")]
#[expect(dead_code, reason = "TODO")]
fn build_pfp(
    role: Role,
    customer_pfp: Option<String>,
    vendor_pfp: Option<String>,
) -> ProfilePicture {
    match (role, customer_pfp, vendor_pfp) {
        (Role::Customer, Some(url), None) | (Role::Vendor, None, Some(url)) => {
            ProfilePicture::new(url.into())
        },
        (Role::Administrator, None, None) => ProfilePicture::admin(),
        _ => unreachable!("Database returned inconsistent profile picture data."),
    }
}

/// Get reviews and associated comments for a product sorted by score, for display on product
/// pages.
///
/// # Errors
///
/// Fails if:
/// - `product` is invalid.
/// - `limit > i64::MAX`.
/// - `offset > i64::MAX`.
/// - An error occurs during communication with the database.
#[server]
#[expect(unreachable_code, reason = "TODO")]
#[expect(unused_mut, reason = "TODO")]
#[expect(unused_variables, reason = "TODO")]
#[expect(
    clippy::missing_panics_doc,
    reason = "Database validation and correctness checks only."
)]
pub async fn product_reviews(
    product: Id<Product>,
    limit: usize,
    offset: usize,
) -> Result<Box<[ProductReview]>> {
    let mut review_ids = Vec::with_capacity(limit);

    let mut tx = connection()
        .begin_with("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY;")
        .await?;

    let reviews = query_as!(
        ReviewRepr,
        r#"
        SELECT r.id, r.customer, username, profile_picture AS "profile_picture!", rating,
            r.created_at, r.updated_at, title, content,
            COALESCE(SUM(CASE review_votes.grade
                WHEN 'like' THEN 1
                WHEN 'dislike' THEN -1
            END), 0) AS "sum_votes!"
        FROM reviews r
        JOIN users ON users.id = r.customer
        JOIN customers ON customers.id = r.customer
        JOIN ratings ON ratings.product = $1 AND ratings.customer = r.customer
        LEFT JOIN review_votes ON review_votes.review = r.id
        WHERE r.product = $1
        GROUP BY r.id, username, profile_picture, rating
        ORDER BY "sum_votes!" DESC
        LIMIT $2
        OFFSET $3
        "#,
        product.get(),
        i64::try_from(limit)?,
        i64::try_from(offset)?,
    )
    .fetch_all(&mut *tx)
    .await?;

    let comments = query_as!(
        CommentRepr,
        r#"
        SELECT c.id, parent, review, user_id, username, role AS "role: Role", content, c.created_at, c.updated_at,
            customers.profile_picture AS customer_pfp, vendors.profile_picture AS vendor_pfp,
            COALESCE(SUM(CASE comment_votes.grade
                WHEN 'like' THEN 1
                WHEN 'dislike' THEN -1
            END), 0) AS "sum_votes!"
        FROM comments c
        JOIN users ON users.id = c.user_id
        LEFT JOIN customers ON customers.id = c.user_id
        LEFT JOIN vendors ON vendors.id = c.user_id
        LEFT JOIN comment_votes ON comment_votes.comment = c.id
        WHERE review = ANY($1)
        GROUP BY c.id, username, role, customer_pfp, vendor_pfp
        ORDER BY review, parent NULLS FIRST, "sum_votes!" DESC, created_at
        "#,
        &*reviews
            .iter()
            .map(|review| review.id)
            .collect_into(&mut review_ids),
    )
    .fetch_all(&mut *tx)
    .await?;

    // PERF: Can resolve in the background while heavier work is done here. Optimizes for the
    // success path.
    let commit = spawn(tx.commit());

    // TODO: After missing implementation below, can replace this call with more general
    // `Into::into` and remove type hints.
    let mut reviews = reviews
        .into_iter()
        .map(Into::<ProductReview>::into)
        .collect::<Box<_>>();

    // TODO: Construct comment trees, attach them to reviews.
    todo!();

    debug_assert!(
        {
            fn comments_sorted(comments: &[CommentTree]) -> bool {
                comments.is_sorted_by_key(|c| (Reverse(c.sum_votes), c.created_at))
                    && comments.iter().all(|c| comments_sorted(&c.replies))
            }
            reviews.iter().all(|r| comments_sorted(&r.comments))
        },
        "Comments not sorted."
    );

    // Outer error from `spawn`, inner from `commit`. A `JoinError` returned from `spawn`
    // should either be due to the task being cancelled, which it isn't, or due to the task
    // panicking, which it shouldn't.
    commit.await.expect("Unexpected error from spawned task.")?;
    Ok(reviews)
}

/// Get reviews and associated comments for a product sorted by score, for display on product
/// pages.
///
/// # Errors
///
/// Fails if:
/// - `customer` or `product` is invalid.
/// - `limit > i64::MAX`.
/// - `offset > i64::MAX`.
/// - An error occurs during communication with the database.
#[server]
#[expect(unreachable_code, reason = "TODO")]
#[expect(unused_mut, reason = "TODO")]
#[expect(unused_variables, reason = "TODO")]
#[expect(
    clippy::missing_panics_doc,
    reason = "Database validation and correctness checks only."
)]
pub async fn product_reviews_as(
    customer: Id<Customer>,
    product: Id<Product>,
    limit: usize,
    offset: usize,
) -> Result<(Option<OwnReview>, Box<[ProductReview]>)> {
    // Own review is fetched separately from usual limit.
    let mut review_ids = Vec::with_capacity(limit + 1);

    let mut tx = connection()
        .begin_with("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY;")
        .await?;

    let own_review = query_as!(
        OwnReviewRepr,
        r#"
        SELECT r.id, rating, created_at, updated_at, title, content,
            COALESCE(SUM(CASE review_votes.grade
                WHEN 'like' THEN 1
                WHEN 'dislike' THEN -1
            END), 0) AS "sum_votes!"
        FROM reviews r
        JOIN ratings ON ratings.product = $1 AND ratings.customer = r.customer
        LEFT JOIN review_votes ON review_votes.review = r.id
        WHERE r.customer = $1 AND r.product = $2
        GROUP BY r.id, rating
        "#,
        customer.get(),
        product.get(),
    )
    .fetch_optional(&mut *tx)
    .await?;
    if let Some(ref own_review) = own_review {
        review_ids.push(own_review.id);
    }

    let other_reviews = query_as!(
        OtherReviewRepr,
        r#"
        SELECT r.id, r.customer, username, profile_picture AS "profile_picture!", rating,
            r.created_at, r.updated_at, title, content,
            COALESCE(SUM(CASE review_votes.grade
                WHEN 'like' THEN 1
                WHEN 'dislike' THEN -1
            END), 0) AS "sum_votes!",
            (
                SELECT grade
                FROM review_votes
                WHERE customer = $1 AND review = r.id
            ) AS "own_vote: Vote"
        FROM reviews r
        JOIN users ON users.id = r.customer
        JOIN customers ON customers.id = r.customer
        JOIN ratings ON ratings.product = $1 AND ratings.customer = r.customer
        LEFT JOIN review_votes ON review_votes.review = r.id
        WHERE r.product = $2 AND r.customer != $1
        GROUP BY r.id, username, profile_picture, rating
        ORDER BY "sum_votes!" DESC
        LIMIT $3
        OFFSET $4
        "#,
        customer.get(),
        product.get(),
        i64::try_from(limit)?,
        i64::try_from(offset)?,
    )
    .fetch_all(&mut *tx)
    .await?;

    let comments = query_as!(
        CommentReprCustomer,
        r#"
        SELECT c.id, parent, review, user_id, username, role AS "role: Role", content, c.created_at, c.updated_at,
            customers.profile_picture AS customer_pfp, vendors.profile_picture AS vendor_pfp,
            COALESCE(SUM(CASE comment_votes.grade
                WHEN 'like' THEN 1
                WHEN 'dislike' THEN -1
            END), 0) AS "sum_votes!",
            (
                SELECT grade
                FROM comment_votes
                WHERE customer = $1 AND comment = c.id
            ) AS "own_vote: Vote"
        FROM comments c
        JOIN users ON users.id = c.user_id
        LEFT JOIN customers ON customers.id = c.user_id
        LEFT JOIN vendors ON vendors.id = c.user_id
        LEFT JOIN comment_votes ON comment_votes.comment = c.id
        WHERE review = ANY($2)
        GROUP BY c.id, username, role, customer_pfp, vendor_pfp
        ORDER BY review, parent NULLS FIRST, "sum_votes!" DESC, created_at
        "#,
        customer.get(),
        &*other_reviews
            .iter()
            .map(|review| review.id)
            .collect_into(&mut review_ids),
    )
    .fetch_all(&mut *tx)
    .await?;

    // PERF: Can resolve in the background while heavier work is done here. Optimizes for the
    // success path.
    let commit = spawn(tx.commit());

    // TODO: After missing implementation below, can replace this call with more general
    // `Into::into`.
    let own_review = own_review.map(Into::<OwnReview>::into);

    let mut other_reviews = other_reviews
        .into_iter()
        .map(Into::<ProductReview>::into)
        .collect::<Box<_>>();

    // TODO: Construct comment trees, attach them to reviews.
    todo!();

    debug_assert!(
        {
            fn comments_sorted(comments: &[CommentTree]) -> bool {
                comments.is_sorted_by_key(|c| (Reverse(c.sum_votes), c.created_at))
                    && comments.iter().all(|c| comments_sorted(&c.replies))
            }
            own_review
                .as_ref()
                .is_none_or(|r| comments_sorted(&r.comments))
                && other_reviews.iter().all(|r| comments_sorted(&r.comments))
        },
        "Comments not sorted."
    );

    // Outer error from `spawn`, inner from `commit`. A `JoinError` returned from `spawn`
    // should either be due to the task being cancelled, which it isn't, or due to the task
    // panicking, which it shouldn't.
    commit.await.expect("Unexpected error from spawned task.")?;
    Ok((own_review, other_reviews))
}

/// Creates a review on a product.
///
/// # Errors
///
/// Fails if:
/// - `customer` or `product` is invalid.
/// - The customer already has a review on the product.
/// - The customer has not placed a rating on the product.
/// - The customer is not allowed to place reviews.
/// - An error occurs during communication with the database.
#[server]
pub async fn create_review(
    customer: Id<Customer>,
    product: Id<Product>,
    title: Box<str>,
    content: Box<str>,
) -> Result<()> {
    query!(
        "
        INSERT INTO reviews (customer, product, title, content)
        VALUES ($1, $2, $3, $4)
        ",
        customer.get(),
        product.get(),
        &title,
        &content,
    )
    .execute(connection())
    .await
    .map(QueryResultExt::expect_one)
    .map_err(Into::into)
}

/// Updates a review.
///
/// # Errors
///
/// Fails if:
/// - `review` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn update_review(review: Id<Review>, title: Box<str>, content: Box<str>) -> Result<()> {
    query!(
        "
        UPDATE reviews
        SET title = $2, content = $3
        WHERE id = $1
        ",
        review.get(),
        &title,
        &content,
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
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
pub async fn delete_review(review: Id<Review>) -> Result<()> {
    query!(
        "
        DELETE FROM reviews
        WHERE id = $1
        ",
        review.get(),
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}

/// Creates a comment on a review.
///
/// # Errors
///
/// Fails if:
/// - `user` or `parent` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn create_comment(user: Id<User>, parent: Id<Review>, content: Box<str>) -> Result<()> {
    query!(
        "
        INSERT INTO comments (user_id, review, content)
        VALUES ($1, $2, $3)
        ",
        user.get(),
        parent.get(),
        &content,
    )
    .execute(connection())
    .await
    .map(QueryResultExt::expect_one)
    .map_err(Into::into)
}

/// Creates a comment on another comment.
///
/// # Errors
///
/// Fails if:
/// - `user` or `parent` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn create_reply(user: Id<User>, parent: Id<Comment>, content: Box<str>) -> Result<()> {
    query!(
        "
        INSERT INTO comments (user_id, review, parent, content)
        VALUES ($1, (SELECT review FROM comments WHERE id = $1), $2, $3)
        ",
        user.get(),
        parent.get(),
        &content,
    )
    .execute(connection())
    .await
    .map(QueryResultExt::expect_one)
    .map_err(Into::into)
}

/// Deletes a comment and all replies to it.
///
/// # Errors
///
/// Fails if:
/// - `comment` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn delete_comment(comment: Id<Comment>) -> Result<()> {
    query!(
        "
        DELETE FROM comments
        WHERE id = $1
        ",
        comment.get(),
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}

/// Sets the customer's vote status on a review. Setting `vote = None` removes the vote.
///
/// # Errors
///
/// Fails if:
/// - `customer` or `review` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_vote_review(
    customer: Id<Customer>,
    review: Id<Review>,
    vote: Option<Vote>,
) -> Result<()> {
    if let Some(vote) = vote {
        query!(
            "
            INSERT INTO review_votes (customer, review, grade)
            VALUES ($1, $2, $3)
            ON CONFLICT (customer, review) DO UPDATE
            SET grade = EXCLUDED.grade
            ",
            customer.get(),
            review.get(),
            vote as Vote,
        )
        .execute(connection())
        .await
        .map(QueryResultExt::expect_one)
        .map_err(Into::into)
    } else {
        query!(
            "
            DELETE FROM review_votes
            WHERE customer = $1 AND review = $2
            ",
            customer.get(),
            review.get(),
        )
        .execute(connection())
        .await?
        .by_unique_key(|| todo!())
    }
}

/// Sets the customer's vote status on a comment. Setting `vote = None` removes the vote.
///
/// # Errors
///
/// Fails if:
/// - `customer` or `comment` is invalid.
/// - An error occurs during communication with the database.
#[server]
pub async fn set_vote_comment(
    customer: Id<Customer>,
    comment: Id<Comment>,
    vote: Option<Vote>,
) -> Result<()> {
    if let Some(vote) = vote {
        query!(
            "
            INSERT INTO comment_votes (customer, comment, grade)
            VALUES ($1, $2, $3)
            ON CONFLICT (customer, comment) DO UPDATE
            SET grade = EXCLUDED.grade
            ",
            customer.get(),
            comment.get(),
            vote as Vote,
        )
        .execute(connection())
        .await
        .map(QueryResultExt::expect_one)
        .map_err(Into::into)
    } else {
        query!(
            "
            DELETE FROM comment_votes
            WHERE customer = $1 AND comment = $2
            ",
            customer.get(),
            comment.get(),
        )
        .execute(connection())
        .await?
        .by_unique_key(|| todo!())
    }
}

/// A review by a known customer, for display on profile pages.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomerReview {
    /// The product the review belongs to.
    product: Id<Product>,
    /// The name of the product.
    product_name: Box<str>,
    /// The thumbnail of the product.
    thumbnail: Url,
    /// The rating associated with the review.
    rating: Rating,
    /// The title of the review.
    title: Box<str>,
    /// The content of the review.
    content: Box<str>,
}

#[cfg(feature = "server")]
struct CustomerReviewRepr {
    product: RawId,
    thumbnail: String,
    rating: i32,
    title: String,
    content: String,
    product_name: String,
}

#[cfg(feature = "server")]
impl From<CustomerReviewRepr> for CustomerReview {
    fn from(
        CustomerReviewRepr {
            product,
            thumbnail,
            rating,
            title,
            content,
            product_name,
        }: CustomerReviewRepr,
    ) -> Self {
        Self {
            product: product.into(),
            product_name: product_name.into(),
            rating: Rating::new(rating as u8).expect("Invalid rating."),
            title: title.into(),
            content: content.into(),
            thumbnail: thumbnail.into(),
        }
    }
}

/// Get reviews made by a customer, sorted by most recently updated.
///
/// # Errors
///
/// Fails if:
/// - `customer` is invalid.
/// - `limit > i64::MAX`.
/// - `offset > i64::MAX`.
/// - An error occurs during communication with the database.
#[server]
pub async fn customer_reviews(
    customer: Id<Customer>,
    limit: usize,
    offset: usize,
) -> Result<Box<[CustomerReview]>> {
    query_as!(
        CustomerReviewRepr,
        "
        SELECT r.product, thumbnail, rating, title, content, products.name AS product_name
        FROM reviews r
        JOIN products ON products.id = r.product
        JOIN ratings ON ratings.customer = $1 AND ratings.product = r.product
        WHERE r.customer = $1
        ORDER BY r.updated_at DESC
        LIMIT $2
        OFFSET $3
        ",
        customer.get(),
        i64::try_from(limit)?,
        i64::try_from(offset)?,
    )
    .fetch_all(connection())
    .await
    .map(|reviews| reviews.into_iter().map(Into::into).collect())
    .map_err(Into::into)
}
