//use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct GlobalState {
    pub cart_items: Vec<i32>,
    pub favorites: Vec<i32>,
    pub auth: FakeAuthState,
}

impl Default for GlobalState {
    fn default() -> Self {
        Self {
            cart_items: Vec::new(),
            favorites: Vec::new(),
            auth: FakeAuthState { user: None },
        }
    }
}

/// TODO(auth): Tas bort när riktig auth finns.
#[derive(Clone, PartialEq, Debug)]
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

impl FakeAuthState {
    
}