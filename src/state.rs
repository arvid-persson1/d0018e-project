use crate::database::Login;
//use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct CartItem {
    pub product_id: i32,
    pub name: String,
    pub price: f64,
    pub image_url: String,
    pub quantity: u32,
}

/// Global state shared across the app.
#[derive(Clone, Debug, Default)]
pub struct GlobalState {
    pub cart: Vec<CartItem>,
    pub favorites: Vec<i32>,
    pub login: Option<Login>,
}

impl GlobalState {
    pub fn add_to_cart(&mut self, product_id: i32, name: String, price: f64, image_url: String) {
        if let Some(item) = self.cart.iter_mut().find(|i| i.product_id == product_id) {
            item.quantity += 1;
        } else {
            self.cart.push(CartItem {
                product_id,
                name,
                price,
                image_url,
                quantity: 1,
            });
        }
    }

    pub fn set_quantity(&mut self, product_id: i32, quantity: u32) {
        if quantity == 0 {
            self.cart.retain(|i| i.product_id != product_id);
        } else if let Some(item) = self.cart.iter_mut().find(|i| i.product_id == product_id) {
            item.quantity = quantity;
        }
    }

    pub fn remove_from_cart(&mut self, product_id: i32) {
        self.cart.retain(|i| i.product_id != product_id);
    }

    pub fn cart_total(&self) -> f64 {
        self.cart.iter().map(|i| i.price * i.quantity as f64).sum()
    }

    pub fn cart_count(&self) -> usize {
        self.cart.iter().map(|i| i.quantity as usize).sum()
    }
    pub fn customer_id(&self) -> Option<crate::database::Id<crate::database::Customer>> {
        self.login.as_ref().and_then(|l| {
            if let crate::database::LoginId::Customer(id) = l.id {
                Some(id)
            } else {
                None
            }
        })
    }
}