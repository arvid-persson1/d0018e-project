//use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct GlobalState {
    pub cart_items: Vec<i32>,
    pub favorites: Vec<i32>,
}
