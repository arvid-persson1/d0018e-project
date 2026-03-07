//use dioxus::prelude::*;
#[derive(Clone, Debug, PartialEq)]
pub struct CartItem {
    pub product_id: i32,
    pub name: String,
    pub price: f64,
    pub image_url: String,
    pub quantity: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GlobalState {
    pub cart: Vec<CartItem>,
    pub favorites: Vec<i32>,
    pub auth: FakeAuthState,
    /// Håller produkt-ID för varning om man försöker favoritmarkera utan inloggning
    pub fav_warning: Option<i32>,
}

impl Default for GlobalState {
    fn default() -> Self {
        Self {
            cart: Vec::new(),
            favorites: Vec::new(),
            auth: FakeAuthState { user: None },
            fav_warning: None,
        }
    }
}
impl GlobalState {
    pub fn add_to_cart(&mut self, product_id: i32, name: String, price: f64, image_url: String) {
        if let Some(item) = self.cart.iter_mut().find(|i| i.product_id == product_id) {
            item.quantity += 1;
        } else {
            self.cart.push(CartItem { product_id, name, price, image_url, quantity: 1 });
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
}

/// TODO(auth): Tas bort när riktig auth finns.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct FakeAuthState {
    pub user: Option<FakeUser>,
}

/// TODO(auth): Ersätts med data från databasen.
#[derive(Clone, PartialEq, Debug)]
pub struct FakeUser {
    pub id: i32,
    pub name: String,
    pub role: UserRole,
}

#[derive(Clone, PartialEq, Debug)]
pub enum UserRole {
    Customer,
    Vendor,
}

impl FakeAuthState {}