#![feature(if_let_guard)]
#![expect(unused_imports, reason = "TODO")]
#![expect(unused_variables, reason = "TODO")]
#![expect(dead_code, reason = "TODO")]
#![expect(clippy::unused_async, reason = "TODO")]
#![expect(clippy::todo, reason = "TODO")]

//! Database operations.

use sqlx::{
    PgPool as Pool, Result, query_file, query_file_as,
    types::{
        Decimal,
        chrono::{NaiveDate, NaiveDateTime},
    },
};

mod types;
pub use types::*;

mod id;
pub use id::*;

/// The maximum length of a username in characters (not bytes).
pub const USERNAME_MAX_LENGTH: usize = 20;

/// A wrapper for a connection to the database.
#[derive(Clone, Debug)]
pub struct Connection {
    /// The connection pool.
    pool: Pool,
}

impl Connection {
    /// Opens a connection and runs any startup code.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - A connection can't be established.
    /// - An error occurs in startup code.
    #[inline]
    pub async fn new(url: &str) -> Result<Self> {
        let conn = Self {
            pool: Pool::connect(url).await?,
        };

        // TODO: Set up compile-time verification.
        // query_file!("queries/startup.sql").execute(&conn).await?;

        Ok(conn)
    }

    /// Opens a connection without running any startup code.
    ///
    /// # Safety
    ///
    /// Not running startup code might break data integrity.
    ///
    /// # Errors
    ///
    /// Fails if a connection can't be established.
    #[inline]
    pub async unsafe fn new_raw(url: &str) -> Result<Self> {
        Pool::connect(url).await.map(|pool| Self { pool })
    }

    /// Gets the hierarchy of categories as one or more trees.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn category_trees(&self) -> Result<Box<[CategoryTree]>> {
        todo!()
    }

    /// Gets products with active discounts sorted by best discounts, as defined by
    /// [`discount_average`](SpecialOfferDeal::discount_average).
    ///
    /// Only visible products with units in stock are considered.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn best_discounts(offset: usize, limit: usize) -> Result<Box<[ProductOverview]>> {
        todo!()
    }

    /// Gets the most recently created products.
    ///
    /// Only visible products with units in stock are considered.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn newest_products(offset: usize, limit: usize) -> Result<Box<[ProductOverview]>> {
        todo!()
    }

    /// Gets a user's favorites sorted by name.
    ///
    /// Includes products out of stock, but not invisible products.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn favorites(
        customer: Id<Customer>,
        offset: usize,
        limit: usize,
    ) -> Result<Box<[ProductOverview]>> {
        todo!()
    }

    /// Gets information about a product, for display on product pages.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn product_info(product: Id<Product>) -> Result<ProductInfo> {
        todo!()
    }

    /// Gets other products in the same category, sorted by best active discounts, then in random
    /// order.
    ///
    /// Only visible products with units in stock are considered.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn similar_products(
        similar_to_product: Id<Product>,
        offset: usize,
        limit: usize,
    ) -> Result<Box<[ProductOverview]>> {
        todo!()
    }

    /// Gets others' reviews for a product.
    ///
    /// Only includes visible reviews, i.e. those with a title and content.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn product_reviews(
        product: Id<Product>,
        offset: usize,
        limit: usize,
    ) -> Result<Box<[ProductReview]>> {
        todo!()
    }

    /// Puts `count` units of a product in a customer's shopping cart, overriding any number
    /// already there. Setting `count = 0` removes the product from the shopping cart.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `count > i32::MAX`.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_in_shopping_cart(
        customer: Id<Customer>,
        product: Id<Product>,
        count: u32,
    ) -> Result<()> {
        todo!()
    }

    /// Increment the number of a product in a customer's shopping cart. Does nothing if the
    /// product was not already present in the shopping cart.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - The count overflows.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn increment_in_shopping_cart(
        customer: Id<Customer>,
        product: Id<Product>,
    ) -> Result<()> {
        todo!()
    }

    /// Decrement the number of a product in a customer's shopping cart. Decrementing to 0 removes
    /// the product from the shopping cart.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn decrement_in_shopping_cart(
        customer: Id<Customer>,
        product: Id<Product>,
    ) -> Result<()> {
        todo!()
    }

    /// Sets the "favorite" status of a product for a customer.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn set_favorite(
        customer: Id<Customer>,
        product: Id<Product>,
        favorite: bool,
    ) -> Result<()> {
        todo!()
    }

    /// Creates a visible review, i.e. one with title and content that shows up in the list of
    /// reviews.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn create_review_full(
        product: Id<Product>,
        customer: Id<Product>,
        rating: Rating,
        title: &str,
        content: &str,
    ) -> Result<()> {
        todo!()
    }

    /// Creates an invisible review, i.e. one without title and content that only contributes to
    /// rating tallies.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn create_review_empty(
        product: Id<Product>,
        customer: Id<Customer>,
        rating: Rating,
    ) -> Result<()> {
        todo!()
    }

    /// Deletes a review and all comments on it.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn delete_review(review: Id<Review>) -> Result<()> {
        todo!()
    }

    /// Creates a comment on a review.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn create_comment<U: UserSuper>(
        parent: Id<Review>,
        user: Id<U>,
        content: &str,
    ) -> Result<()> {
        todo!()
    }

    /// Creates a comment on another comment.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn create_reply<U: UserSuper>(
        parent: Id<Comment>,
        user: Id<U>,
        content: &str,
    ) -> Result<()> {
        todo!()
    }

    /// Deletes a comment and all replies to it.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn delete_comment(comment: Id<Comment>) -> Result<()> {
        todo!()
    }

    /// Sets the user's vote status on a review. Setting `vote = None` removes the vote.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn set_vote_review<U: UserSuper>(
        user: Id<U>,
        review: Id<Review>,
        vote: Option<Vote>,
    ) -> Result<()> {
        todo!()
    }

    /// Sets the user's vote status on a comment. Setting `vote = None` removes the vote.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn set_vote_comment<U: UserSuper>(
        user: Id<U>,
        review: Id<Comment>,
        vote: Option<Vote>,
    ) -> Result<()> {
        todo!()
    }

    /// Sets a customer's profile picture.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn set_customer_profile_picture(
        customer: Id<Customer>,
        url: UrlRef<'_>,
    ) -> Result<()> {
        todo!()
    }

    /// Sets a vendor's profile picture.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn set_vendor_profile_picture(vendor: Id<Vendor>, url: UrlRef<'_>) -> Result<()> {
        todo!()
    }

    /// Sets a user's username.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - The username is too long, as defined by [`USERNAME_MAX_LENGTH`].
    /// - An error occurs during communication with the database
    #[inline]
    pub async fn set_username<U: UserSuper>(user: Id<U>, username: &str) -> Result<()> {
        todo!()
    }

    /// Sets a user's email.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - An invalid email is provided.
    /// - An error occurs during communication with the database
    #[inline]
    pub async fn set_email<U: UserSuper>(user: Id<U>, email: &str) -> Result<()> {
        todo!()
    }

    /// Sets a vendor's display name.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn set_vendor_display_name(vendor: Id<Vendor>, display_name: &str) -> Result<()> {
        todo!()
    }

    /// Sets a vendor's description.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn set_vendor_description(vendor: Id<Vendor>, description: &str) -> Result<()> {
        todo!()
    }

    /// Gets reviews made by a customer.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn review_log(
        customer: Id<Customer>,
        offset: usize,
        limit: usize,
    ) -> Result<Box<[ReviewLog]>> {
        todo!()
    }

    /// Gets orders made by a customer.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn orders(
        customer: Id<Customer>,
        offset: usize,
        limit: usize,
    ) -> Result<Box<[Order]>> {
        todo!()
    }

    /// Gets products owned by a vendor.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn vendor_products(
        vendor: Id<Vendor>,
        offset: usize,
        limit: usize,
        include_invisible: bool,
    ) -> Result<Box<[ProductOverview]>> {
        todo!()
    }

    /// Create a new product.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    #[expect(
        clippy::too_many_arguments,
        reason = "Intent is clearly communicated through names and types."
    )]
    pub async fn create_product(
        vendor: Id<Vendor>,
        name: &str,
        thumbnail: UrlRef<'_>,
        gallery: &[UrlRef<'_>],
        price: Decimal,
        overview: &str,
        description: &str,
        category: Id<Category>,
        amount_per_unit: Option<Amount>,
        origin: &str,
    ) -> Result<()> {
        todo!()
    }

    /// Set the name of a product.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn set_product_name(product: Id<Product>, name: &str) -> Result<()> {
        todo!()
    }

    /// Set the thumbnail of a product.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn set_thumbnail(product: Id<Product>, url: UrlRef<'_>) -> Result<()> {
        todo!()
    }

    /// Get the gallery of a product.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn gallery(product: Id<Product>) -> Result<Box<[Url]>> {
        todo!()
    }

    /// Set the gallery of a product.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn set_gallery(product: Id<Product>, gallery: &[UrlRef<'_>]) -> Result<()> {
        todo!()
    }

    /// Append to the gallery of a product.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn add_to_gallery(product: Id<Product>, additions: &[UrlRef<'_>]) -> Result<()> {
        todo!()
    }

    /// Set the price of a product.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - The new price is lower than one provided by an active special offer.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_price(product: Id<Product>, price: Decimal) -> Result<()> {
        todo!()
    }

    /// Set the overview of a product.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn set_overview(product: Id<Product>, overview: &str) -> Result<()> {
        todo!()
    }

    /// Set the description of a product.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn set_description(product: Id<Product>, description: &str) -> Result<()> {
        todo!()
    }

    /// Set the category of a product.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn set_category(product: Id<Product>, category: Id<Category>) -> Result<()> {
        todo!()
    }

    /// Set the amount per unit of a product.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn set_amount(product: Id<Product>, amount: Option<Amount>) -> Result<()> {
        todo!()
    }

    /// Set the origin of a product.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn set_origin(product: Id<Product>, origin: &str) -> Result<()> {
        todo!()
    }

    /// Add units to stock.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - The expiry date is in the past.
    /// - An error occurs during communication with the database
    #[inline]
    pub async fn add_stock(product: Id<Product>, count: u32, expiry: NaiveDate) -> Result<()> {
        todo!()
    }

    /// Set the visibility of a product.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn set_visibility(product: Id<Product>, visible: bool) -> Result<()> {
        todo!()
    }

    /// Mark a user as deleted.
    ///
    /// This deletes their reviews, shopping cart, favorites and votes (if they were a customer),
    /// products (if they were a vendor) as well as their comments. Order history is kept if they
    /// were a customer.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn delete_user<U: UserSuper>(user: Id<U>) -> Result<()> {
        todo!()
    }

    /// Create a category.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - The category breaks a tree structure by creating a cycle.
    /// - An error occurs during communication with the database-
    #[inline]
    pub async fn create_category(name: &str, supercategory: Option<Id<Category>>) -> Result<()> {
        todo!()
    }

    /// Delete a category and all of its subcategories.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - Any products belong to the category.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn delete_category(category: Id<Category>) -> Result<()> {
        todo!()
    }

    /// Create a special offer for a product.
    ///
    /// Special offers with an end time of `None` must be deleted or otherwise disabled manually.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - The special offer overlaps with an existing one.
    /// - The special offer does not actually provide a discount compared to the current price.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn create_special_offer(
        product: Id<Product>,
        special_offer: &ProductSpecialOffer,
    ) -> Result<()> {
        todo!()
    }

    /// Deletes a special offer.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn delete_special_offer(special_offer: Id<SpecialOffer>) -> Result<()> {
        todo!()
    }

    /// Sets the limit per customer of a special offer.
    ///
    /// This might make it so that some customers have already used the special offer more times
    /// than are allowed by the new limit. These customers are restricted from further usage unless
    /// the limit is increased, but no changes are made to order history.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn set_special_offer_limit(
        special_offer: Id<SpecialOffer>,
        limit_per_customer: u32,
    ) -> Result<()> {
        todo!()
    }

    /// Sets the "members only"-status of a special offer.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn set_special_offer_members_only(
        special_offer: Id<SpecialOffer>,
        members_only: bool,
    ) -> Result<()> {
        todo!()
    }

    /// Sets the start time of a special offer.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - The special offer now overlaps with an existing one.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_special_offer_start(
        special_offer: Id<SpecialOffer>,
        valid_from: NaiveDateTime,
    ) -> Result<()> {
        todo!()
    }

    /// Sets the end time of a special offer.
    ///
    /// Special offers with an end time of `None` must be deleted or otherwise disabled manually.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - The special offer now overlaps with an existing one.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_special_offer_end(
        special_offer: Id<SpecialOffer>,
        valid_until: Option<NaiveDateTime>,
    ) -> Result<()> {
        todo!()
    }

    /// Sets the activity status of a special offer.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - The special offer overlaps with an existing one.
    /// - The special offer does not actually provide a discount.
    /// - An error occurs during communication with the database
    #[inline]
    pub async fn set_special_offer_active(
        special_offer: Id<SpecialOffer>,
        active: bool,
    ) -> Result<()> {
        todo!()
    }

    /// Sets the deal of a special offer.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - The special offer does not actually provide a discount.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_special_offer_deal(
        special_offer: Id<SpecialOffer>,
        deal: SpecialOfferDeal,
    ) -> Result<()> {
        todo!()
    }

    /// Completes an order for a customer, emptying their shopping cart.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during communication with the database.
    #[inline]
    pub async fn complete_order(customer: Id<Customer>) -> Result<()> {
        todo!()
    }
}
