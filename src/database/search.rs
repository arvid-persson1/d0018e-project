//! Database functions for performing text searches.

use crate::database::{Id, Product, Url};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use {crate::database::POOL, sqlx::query_as};

/// A product matching a search query.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchResult {
    /// The ID of the product.
    pub id: Id<Product>,
    /// The name of the product.
    pub name: Box<str>,
    /// URL to an image to display on the product card.
    pub thumbnail: Url,
}

#[cfg(feature = "server")]
struct SearchResultRepr {
    id: i32,
    name: String,
    thumbnail: Url,
}

#[cfg(feature = "server")]
impl From<SearchResultRepr> for SearchResult {
    fn from(
        SearchResultRepr {
            id,
            name,
            thumbnail,
        }: SearchResultRepr,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            thumbnail,
        }
    }
}

/// Search for products by name, category and description.
///
/// # Errors
///
/// Fails if:
/// - `limit > i64::MAX`.
/// - An error occurs during communication with the database.
#[server]
pub async fn search_products(query: Box<str>, limit: usize) -> Result<Vec<SearchResult>> {
    query_as!(
        SearchResultRepr,
        "
        WITH search AS (
            SELECT plainto_tsquery('english', $1) AS query
        )
        SELECT id, name, thumbnail
        FROM products, search
        WHERE search_vector @@ query
        ORDER BY ts_rank(search_vector, query) DESC
        LIMIT $2
        ",
        &*query,
        i64::try_from(limit)?,
    )
    .fetch_all(&*POOL)
    .await
    .map(|results| results.into_iter().map(Into::into).collect())
    .map_err(Into::into)
}

// TODO: Remove.
#[cfg(feature = "web")]
use gloo_timers as _;
#[cfg(false)]
mod usage {
    use super::*;
    use dioxus_core::Task;
    use std::time::Duration;
    // TODO: Is this the proper way to handle sleep? The client has to sleep, but only the
    // server can compile `tokio`. Does the server need to know about the sleep at all?
    #[cfg(feature = "web")]
    use gloo_timers::future::sleep;
    #[cfg(feature = "server")]
    use tokio::time::sleep;

    #[component]
    fn SearchBar() -> Element {
        const LIMIT: usize = 10;
        const DEBOUNCE_DELAY: Duration = Duration::from_millis(300);

        let mut raw = use_signal(String::new);
        let mut debounced = use_signal(String::new);
        let mut debounce_task = use_signal(|| None::<Task>);
        _ = use_effect(move || {
            if let Some(task) = debounce_task.take() {
                task.cancel();
            }

            let query = raw();
            let task = spawn(async move {
                sleep(DEBOUNCE_DELAY).await;
                debounced.set(query)
            });
            debounce_task.set(Some(task));
        });
        let products = use_resource(move || {
            let query = debounced();
            async move {
                if query.trim().is_empty() {
                    Ok(Vec::new())
                } else {
                    search_products(query.into(), LIMIT).await
                }
            }
        });

        rsx! {
            div { id: "search-container",
                input {
                    class: "search-input",
                    placeholder: "Search products...",
                    value: "{raw}",
                    oninput: move |event| raw.set(event.value().trim().into()),
                }

                div { id: "search-results",
                    match products() {
                        Some(Ok(products)) => {
                            if products.is_empty() {
                                rsx! { "No products found." }
                            } else {
                                rsx! {
                                    ul {
                                        for product in products {
                                            li { "{product.name}" }
                                        }
                                    }
                                }
                            }
                        }
                        Some(Err(_)) => todo!(),
                        None => rsx! { "Loading..." },
                    }
                }
            }
        }
    }
}