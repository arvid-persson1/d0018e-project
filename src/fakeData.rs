use dioxus::prelude::*;

// Class for mock Product to test with

#[derive(Clone, PartialEq, Debug)]
pub struct ProductInfo {
    pub id: i32,
    pub name: String,
    pub price: f64,
    pub comparison_price: String,
    pub image_url: String,
    pub description: String,
}

pub fn getFakeProducts() -> Vec<ProductInfo> {
    vec![
        ProductInfo {
            id: 1,
            name: "test produkt 1".to_string(),
            price: 24.90,
            comparison_price: "30 kr/kg".to_string(),
            image_url: "https://via.placeholder.com/400".to_string(),
            description: "beskrivning av produkt 1".to_string(),
        },
        ProductInfo {
            id: 2,
            name: "test produkt 2".to_string(),
            price: 79.89,
            comparison_price: "119 kr/st".to_string(),
            image_url: "https://via.placeholder.com/400".to_string(),
            description: "beskrivning av produkt 2".to_string(),
        },
    ]
}
