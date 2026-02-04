use crate::database::ProductOverview;
use dioxus::prelude::*;

/// A product card.
#[component]
#[expect(unused_variables, reason = "TODO")]
pub fn ProductCard(
    ProductOverview {
        id,
        name,
        thumbnail,
        price,
        overview,
        in_stock,
        amount_per_unit,
        vendor_name,
        origin,
        special_offer,
    }: ProductOverview,
) -> Element {
    rsx! { "{name}" }
}
