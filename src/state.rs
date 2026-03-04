//use dioxus::prelude::*;
//! Global application state.
use crate::database::{Id, Product};
use std::num::NonZeroU32;
use hashbrown::HashMap;

/// Global state shared across the app.
#[derive(Clone, Debug, PartialEq)]
pub struct GlobalState {
    /// Cart items: product ID
    pub cart_items: HashMap<Id<Product>, NonZeroU32>,
    /// Favorited product IDs
    pub favorites: Vec<Id<Product>>,
}

impl GlobalState {
    /// Get the total number of items in the cart
    #[allow(dead_code)]
    pub fn cart_total(&self) -> u32 {
        self.cart_items.values().map(|n| n.get()).sum()
    }

    /// Get the quantity of a specific product in the cart
    #[must_use]
    pub fn cart_count(&self, id: Id<Product>) -> u32 {
        self.cart_items.get(&id).map_or(0, |n| n.get())
    }

    /// Add one unit of a product to the cart
    pub fn add_to_cart(&mut self, id: Id<Product>) {
        use std::num::NonZero;
        let entry = self.cart_items.entry(id).or_insert(NonZero::new(1).unwrap());
        *entry = NonZero::new(entry.get() + 1).unwrap_or(*entry);
    }

    /// Remove one unit of a product from the cart
    /// Removes if count reaches 0
    pub fn remove_from_cart(&mut self, id: Id<Product>) {
        use std::num::NonZero;
        if let Some(count) = self.cart_items.get_mut(&id) {
            if count.get() <= 1 {
                let _ = self.cart_items.remove(&id);
            } else {
                *count = NonZero::new(count.get() - 1).unwrap();
            }
        }
    }

    /// Toggle favorite status for a product
    pub fn toggle_favorite(&mut self, id: Id<Product>) {
        if self.favorites.contains(&id) {
            self.favorites.retain(|&x| x != id);
        } else {
            self.favorites.push(id);
        }
    }
}

