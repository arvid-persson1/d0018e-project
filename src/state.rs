//use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct GlobalState {
    pub cart_count: i32,
    pub favorites: Vec<i32>,
}
