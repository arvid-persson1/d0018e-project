//! Database operations.

use super::{
    Amount, Category, CategoryTree, Comment, Customer, CustomerReviews, Id, Order, Product,
    ProductInfo, ProductOverview, ProductReview, ProductSpecialOffer, Rating, Review, SpecialOffer,
    SpecialOfferDeal, Url, UrlRef, User, Vendor, Vote,
};
use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use sqlx::{PgPool as Pool, Result};

/// A wrapper for a connection to the database.
///
/// [`Id`]s created by methods on this are valid at the time of retrieval from the database, but
/// might of course be invalidated later as a result of deletions. It may even be the case that an
/// ID is invalidated in the time between the ID being fetched from the database and the associated
/// [`Future`] completing.
#[derive(Clone, Debug)]
pub struct Connection {
    /// The connection pool.
    #[expect(dead_code, reason = "TODO")]
    pool: Pool,
}

#[expect(unused_variables, reason = "TODO")]
#[expect(clippy::todo, reason = "TODO")]
#[expect(clippy::unused_async, reason = "TODO")]
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
    pub async fn best_discounts(
        &self,
        offset: usize,
        limit: usize,
    ) -> Result<Box<[ProductOverview]>> {
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
    pub async fn newest_products(
        &self,
        offset: usize,
        limit: usize,
    ) -> Result<Box<[ProductOverview]>> {
        todo!()
    }

    /// Gets a customer's favorites sorted by name.
    ///
    /// Includes products out of stock, but not invisible products.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `customer` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn favorites(
        &self,
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
    /// Fails if:
    /// - `product` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn product_info(&self, product: Id<Product>) -> Result<ProductInfo> {
        todo!()
    }

    /// Gets other products in the same category, sorted by best active discounts, then in random
    /// order.
    ///
    /// Only visible products with units in stock are considered.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `similar_to` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn similar_products(
        &self,
        similar_to: Id<Product>,
        offset: usize,
        limit: usize,
    ) -> Result<Box<[ProductOverview]>> {
        todo!()
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
    #[inline]
    pub async fn product_reviews(
        &self,
        product: Id<Product>,
        except_by: Option<Id<Customer>>,
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
    /// - `customer` or `product` is invalid.
    /// - `count > i32::MAX`.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_in_shopping_cart(
        &self,
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
    /// - `customer` or `product` is invalid.
    /// - The count overflows.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn increment_in_shopping_cart(
        &self,
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
    /// Fails if:
    /// - `customer` or `product` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn decrement_in_shopping_cart(
        &self,
        customer: Id<Customer>,
        product: Id<Product>,
    ) -> Result<()> {
        todo!()
    }

    /// Sets the "favorite" status of a product for a customer.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `customer` or `product` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_favorite(
        &self,
        customer: Id<Customer>,
        product: Id<Product>,
        favorite: bool,
    ) -> Result<()> {
        todo!()
    }

    /// Sets a customer's rating on a product, if any.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `customer` or `product` is invalid.
    /// - Attempting to remove rating (set to `None`) while having a review on the product.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_rating(
        &self,
        customer: Id<Customer>,
        product: Id<Product>,
        rating: Option<Rating>,
    ) -> Result<()> {
        todo!()
    }

    /// Creates a review on a product.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `customer` or `product` is invalid.
    /// - The customer has not placed a rating on the product.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn create_review(
        &self,
        customer: Id<Customer>,
        product: Id<Product>,
        title: &str,
        content: &str,
    ) -> Result<()> {
        todo!()
    }

    /// Updates a review.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `review` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn update_review(
        &self,
        review: Id<Review>,
        title: &str,
        content: &str,
    ) -> Result<()> {
        todo!()
    }

    /// Deletes a review and all comments on it.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `review` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn delete_review(&self, review: Id<Review>) -> Result<()> {
        todo!()
    }

    /// Creates a comment on a review.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `parent` or `user` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn create_comment<U: User>(
        &self,
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
    /// Fails if:
    /// - `parent` or `user` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn create_reply<U: User>(
        &self,
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
    /// Fails if:
    /// - `comment` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn delete_comment(&self, comment: Id<Comment>) -> Result<()> {
        todo!()
    }

    /// Sets the customer's vote status on a review. Setting `vote = None` removes the vote.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `customer` or `review` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_vote_review(
        &self,
        customer: Id<Customer>,
        review: Id<Review>,
        vote: Option<Vote>,
    ) -> Result<()> {
        todo!()
    }

    /// Sets the user's vote status on a comment. Setting `vote = None` removes the vote.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `user` or `review` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_vote_comment(
        &self,
        user: Id<Customer>,
        review: Id<Review>,
        vote: Option<Vote>,
    ) -> Result<()> {
        todo!()
    }

    /// Sets a customer's profile picture.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `customer` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_customer_profile_picture(
        &self,
        customer: Id<Customer>,
        url: UrlRef<'_>,
    ) -> Result<()> {
        todo!()
    }

    /// Sets a vendor's profile picture.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `vendor` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_vendor_profile_picture(
        &self,
        vendor: Id<Vendor>,
        url: UrlRef<'_>,
    ) -> Result<()> {
        todo!()
    }

    /// Sets a user's username.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `user` is invalid.
    /// - The username is too long, as defined by
    ///   [`USERNAME_MAX_LENGTH`](super::USERNAME_MAX_LENGTH).
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_username<U: User>(&self, user: Id<U>, username: &str) -> Result<()> {
        todo!()
    }

    /// Sets a user's email.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `user` is invalid.
    /// - An invalid email is provided.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_email<U: User>(&self, user: Id<U>, email: &str) -> Result<()> {
        todo!()
    }

    /// Sets a vendor's display name.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `vendor` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_vendor_display_name(
        &self,
        vendor: Id<Vendor>,
        display_name: &str,
    ) -> Result<()> {
        todo!()
    }

    /// Sets a vendor's description.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `vendor` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_vendor_description(
        &self,
        vendor: Id<Vendor>,
        description: &str,
    ) -> Result<()> {
        todo!()
    }

    /// Gets reviews made by a customer.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `customer` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn customer_reviews(
        &self,
        customer: Id<Customer>,
        offset: usize,
        limit: usize,
    ) -> Result<Box<[CustomerReviews]>> {
        todo!()
    }

    /// Gets orders made by a customer.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `customer` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn orders(
        &self,
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
    /// Fails if:
    /// - `vendor` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn vendor_products(
        &self,
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
    /// Fails if:
    /// - `vendor` or `category` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    #[expect(clippy::too_many_arguments, reason = "Consistency with other methods.")]
    pub async fn create_product(
        &self,
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
    /// Fails if:
    /// - `product` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_product_name(&self, product: Id<Product>, name: &str) -> Result<()> {
        todo!()
    }

    /// Set the thumbnail of a product.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `product` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_thumbnail(&self, product: Id<Product>, url: UrlRef<'_>) -> Result<()> {
        todo!()
    }

    /// Get the gallery of a product.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `product` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn gallery(&self, product: Id<Product>) -> Result<Box<[Url]>> {
        todo!()
    }

    /// Set the gallery of a product.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `product` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_gallery(&self, product: Id<Product>, gallery: &[UrlRef<'_>]) -> Result<()> {
        todo!()
    }

    /// Append to the gallery of a product.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `product` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn add_to_gallery(
        &self,
        product: Id<Product>,
        additions: &[UrlRef<'_>],
    ) -> Result<()> {
        todo!()
    }

    /// Set the price of a product.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `product` is invalid.
    /// - The new price is lower than one provided by an active special offer.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_price(&self, product: Id<Product>, price: Decimal) -> Result<()> {
        todo!()
    }

    /// Set the overview of a product.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `product` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_overview(&self, product: Id<Product>, overview: &str) -> Result<()> {
        todo!()
    }

    /// Set the description of a product.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `product` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_description(&self, product: Id<Product>, description: &str) -> Result<()> {
        todo!()
    }

    /// Set the category of a product.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `product` or `category` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_category(&self, product: Id<Product>, category: Id<Category>) -> Result<()> {
        todo!()
    }

    /// Set the amount per unit of a product.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `product` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_amount(&self, product: Id<Product>, amount: Option<Amount>) -> Result<()> {
        todo!()
    }

    /// Set the origin of a product.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `product` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_origin(&self, product: Id<Product>, origin: &str) -> Result<()> {
        todo!()
    }

    /// Add units to stock.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `product` is invalid.
    /// - The count overflows.
    /// - `expiry` is in the past.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn add_stock(
        &self,
        product: Id<Product>,
        count: u32,
        expiry: Option<NaiveDate>,
    ) -> Result<()> {
        todo!()
    }

    /// Set the visibility of a product.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `product` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_visibility(&self, product: Id<Product>, visible: bool) -> Result<()> {
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
    /// Fails if:
    /// - `user` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn delete_user<U: User>(&self, user: Id<U>) -> Result<()> {
        todo!()
    }

    /// Create a category.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `parent` (if [`Some`]) is invalid.
    /// - The category breaks a tree structure by creating a cycle.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn create_category(&self, name: &str, parent: Option<Id<Category>>) -> Result<()> {
        todo!()
    }

    /// Delete a category and all of its subcategories.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `category` is invalid.
    /// - Any products belong to the category.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn delete_category(&self, category: Id<Category>) -> Result<()> {
        todo!()
    }

    /// Create a special offer for a product.
    ///
    /// Special offers with an end time of `None` must be deleted or otherwise disabled manually.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `product` is invalid.
    /// - The special offer overlaps with an existing one.
    /// - The special offer does not actually provide a discount compared to the current price.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn create_special_offer(
        &self,
        product: Id<Product>,
        special_offer: &ProductSpecialOffer,
    ) -> Result<()> {
        todo!()
    }

    /// Deletes a special offer.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `special_offer` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn delete_special_offer(&self, special_offer: Id<SpecialOffer>) -> Result<()> {
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
    /// Fails if:
    /// - `special_offer` is invalid.
    /// - `count > i32::MAX`.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_special_offer_limit(
        &self,
        special_offer: Id<SpecialOffer>,
        limit_per_customer: u32,
    ) -> Result<()> {
        todo!()
    }

    /// Sets the "members only"-status of a special offer.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `special_offer` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_special_offer_members_only(
        &self,
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
    /// - `special_offer` is invalid.
    /// - The special offer now overlaps with an existing one.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_special_offer_start(
        &self,
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
    /// - `special_offer` is invalid.
    /// - The special offer now overlaps with an existing one.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_special_offer_end(
        &self,
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
    /// - `special_offer` is invalid.
    /// - The special offer overlaps with an existing one.
    /// - The special offer does not actually provide a discount.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_special_offer_active(
        &self,
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
    /// - `special_offer` is invalid.
    /// - The special offer does not actually provide a discount.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn set_special_offer_deal(
        &self,
        special_offer: Id<SpecialOffer>,
        deal: SpecialOfferDeal,
    ) -> Result<()> {
        todo!()
    }

    /// Completes an order for a customer, emptying their shopping cart.
    ///
    /// # Errors
    ///
    /// Fails if:
    /// - `customer` is invalid.
    /// - An error occurs during communication with the database.
    #[inline]
    pub async fn checkout(&self, customer: Id<Customer>) -> Result<()> {
        todo!()
    }
}
