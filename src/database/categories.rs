//! Database functions for interacting with categories.

use crate::database::{Category, Id};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use {
    crate::database::{QueryResultExt, RawId, connection},
    hashbrown::HashMap,
    sqlx::{query, query_as},
};

#[cfg(feature = "server")]
#[derive(PartialEq, PartialOrd)]
struct CategoryRepr {
    parent: Option<RawId>,
    name: String,
    id: RawId,
}

/// A category with its subcategories, for display in a tree.
///
/// Created by [`category_trees`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategoryTree {
    /// The ID of the category.
    pub id: Id<Category>,
    /// The name of the category.
    pub name: Box<str>,
    /// All direct subcategories.
    pub subcategories: Vec<Self>,
}

#[cfg(feature = "server")]
#[expect(clippy::type_complexity, reason = "Only used once, internally.")]
fn build_tree(
    id: Id<Category>,
    name: Box<str>,
    by_parent: &mut HashMap<Id<Category>, Vec<(Id<Category>, Box<str>)>>,
) -> CategoryTree {
    CategoryTree {
        id,
        name,
        subcategories: by_parent
            .remove(&id)
            .unwrap_or_default()
            .into_iter()
            .map(|(id, name)| build_tree(id, name, by_parent))
            .collect(),
    }
}

/// Get the hierarchy of categories as a forest, with roots and each subtree sorted by name.
///
/// # Errors
///
/// Fails if an error occurs during communication with the database.
#[server]
#[expect(clippy::missing_panics_doc, reason = "See note.")]
pub async fn category_trees() -> Result<Box<[CategoryTree]>> {
    let categories = query_as!(
        CategoryRepr,
        "SELECT *
        FROM categories
        ORDER BY parent NULLS FIRST, NAME",
    )
    .fetch_all(connection())
    .await?;
    // Ordering defined by order of fields. Names are unique so IDs will never be compared.
    debug_assert!(categories.is_sorted(), "Rows not sorted.");

    let mut roots = Vec::new();
    let mut by_parent = HashMap::<_, Vec<_>>::new();
    let mut iter = categories.into_iter().peekable();

    while let Some(CategoryRepr { id, name, .. }) =
        iter.next_if(|category| category.parent.is_none())
    {
        roots.push((id.into(), name.into()));
    }

    for CategoryRepr { id, parent, name } in iter {
        #[expect(
            clippy::unwrap_used,
            reason = "Nodes without parents have already been traversed in the previous loop."
        )]
        by_parent
            .entry(parent.unwrap().into())
            .or_default()
            .push((id.into(), name.into()));
    }

    // TODO: Verify sorted output.
    Ok(roots
        .into_iter()
        .map(|(id, name)| build_tree(id, name, &mut by_parent))
        .collect())
}

/// Create a category.
///
/// # Errors
///
/// Fails if:
/// - `parent` (if [`Some`]) is invalid.
/// - `name` is not unique.
/// - An error occurs during communication with the database.
#[server]
pub async fn create_category(parent: Option<Id<Category>>, name: Box<str>) -> Result<()> {
    query!(
        "
        INSERT INTO categories (parent, name)
        VALUES ($1, $2)
        ",
        parent.map(Id::get),
        &name
    )
    .execute(connection())
    .await
    .map(QueryResultExt::expect_one)
    .map_err(Into::into)
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
pub async fn delete_category(category: Id<Category>) -> Result<()> {
    query!(
        "
        DELETE FROM categories
        WHERE id = $1
        ",
        category.get()
    )
    .execute(connection())
    .await?
    .by_unique_key(|| todo!())
}
