//! Database functiosn for fetching products.

mod get;
pub use get::*;

mod set;
pub use set::*;

mod info;
pub use info::*;

#[cfg(feature = "server")]
use {
    crate::database::{Amount, AverageRating, Deal},
    rust_decimal::Decimal,
};

/// Construct an [`Amount`] from its representation in the database.
///
/// # Panics
///
/// Panics if the values do not uphold any of the database's invariants.
#[cfg(feature = "server")]
fn build_amount(amount_per_unit: Decimal, measurement_unit: Option<String>) -> Amount {
    Amount::new(amount_per_unit, measurement_unit.map(Into::into))
        .expect("Database returned invalid amount.")
}

/// Construct a [`Deal`] from its representation in the database, accepting that it may be
/// [`None`].
///
/// # Panics
///
/// Panics if the values do not uphold any of the database's invariants.
#[cfg(feature = "server")]
#[expect(clippy::unreachable, reason = "Database validation only.")]
fn try_build_special_offer(
    new_price: Option<Decimal>,
    quantity1: Option<i32>,
    quantity2: Option<i32>,
    members_only: Option<bool>,
    price: Decimal,
) -> Option<(Deal, bool)> {
    match (
        Deal::try_new(new_price, quantity1, quantity2, price)
            .expect("Database returned invalid special offer."),
        members_only,
    ) {
        (Some(deal), Some(members_only)) => Some((deal, members_only)),
        (None, None) => None,
        _ => unreachable!("Database returned inconsistent special offer data."),
    }
}

/// Construct an [`AverageRating`] from its representation in the database.
///
/// # Panics
///
/// Panics if the values do not uphold any of the database's invariants.
#[cfg(feature = "server")]
#[expect(clippy::unreachable, reason = "Database validation only.")]
fn build_average_rating(average_rating: Option<f64>, rating_count: Option<i64>) -> AverageRating {
    match (average_rating, rating_count) {
        (Some(average_rating), Some(rating_count)) => AverageRating::new(
            average_rating,
            rating_count
                .try_into()
                .expect("Database returned negative rating count."),
        )
        .expect("Database returned invalid average rating."),
        (None, None) => AverageRating::default(),
        _ => unreachable!("Database returned inconsistent average rating data."),
    }
}
