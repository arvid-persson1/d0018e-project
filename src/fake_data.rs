//use dioxus::prelude::*;

// Class for mock Product to test with

#[derive(Clone, PartialEq, Debug)]
pub struct ProductInfo {
    pub id: i32,
    pub name: String,
    pub price: f64,
    pub comparison_price: String,
    pub image_url: String,
    pub description: String,
    pub category_id: i32,
}

pub fn get_fake_products() -> Vec<ProductInfo> {
    vec![
        ProductInfo {
            id: 1,
            name: "test produkt 1".to_string(),
            price: 24.90,
            comparison_price: "55 kr/kg".to_string(),
            image_url: "https://images.unsplash.com/photo-1571771894821-ad996211fdf4?w=500".to_string(),
            description: "beskrvining".to_string(),
            category_id: 2, // Frukt & Grönt
        },
        ProductInfo {
            id: 1,
            name: "test produkt 1".to_string(),
            price: 24.90,
            comparison_price: "55 kr/kg".to_string(),
            image_url: "https://images.unsplash.com/photo-1571771894821-ad996211fdf4?w=500".to_string(),
            description: "beskrvining".to_string(),
            category_id: 2, // Frukt & Grönt
        },
        ProductInfo {
            id: 1,
            name: "test produkt 1".to_string(),
            price: 24.90,
            comparison_price: "55 kr/kg".to_string(),
            image_url: "https://images.unsplash.com/photo-1571771894821-ad996211fdf4?w=500".to_string(),
            description: "beskrvining".to_string(),
            category_id: 2, // Frukt & Grönt
        },
        ProductInfo {
            id: 1,
            name: "test produkt 1".to_string(),
            price: 24.90,
            comparison_price: "55 kr/kg".to_string(),
            image_url: "https://images.unsplash.com/photo-1571771894821-ad996211fdf4?w=500".to_string(),
            description: "beskrvining".to_string(),
            category_id: 2, // Frukt & Grönt
        },
        ProductInfo {
            id: 1,
            name: "test produkt 1".to_string(),
            price: 24.90,
            comparison_price: "55 kr/kg".to_string(),
            image_url: "https://images.unsplash.com/photo-1571771894821-ad996211fdf4?w=500".to_string(),
            description: "beskrvining".to_string(),
            category_id: 2, // Frukt & Grönt
        },
        ProductInfo {
            id: 1,
            name: "test produkt 1".to_string(),
            price: 24.90,
            comparison_price: "55 kr/kg".to_string(),
            image_url: "https://images.unsplash.com/photo-1571771894821-ad996211fdf4?w=500".to_string(),
            description: "beskrvining".to_string(),
            category_id: 2, // Frukt & Grönt
        },
        ProductInfo {
            id: 1,
            name: "test produkt 1".to_string(),
            price: 24.90,
            comparison_price: "55 kr/kg".to_string(),
            image_url: "https://images.unsplash.com/photo-1571771894821-ad996211fdf4?w=500".to_string(),
            description: "beskrvining".to_string(),
            category_id: 2, // Frukt & Grönt
        },
        ProductInfo {
            id: 1,
            name: "test produkt 1".to_string(),
            price: 24.90,
            comparison_price: "55 kr/kg".to_string(),
            image_url: "https://images.unsplash.com/photo-1571771894821-ad996211fdf4?w=500".to_string(),
            description: "beskrvining".to_string(),
            category_id: 2, // Frukt & Grönt
        },
        ProductInfo {
            id: 1,
            name: "test produkt 1".to_string(),
            price: 24.90,
            comparison_price: "55 kr/kg".to_string(),
            image_url: "https://images.unsplash.com/photo-1571771894821-ad996211fdf4?w=500".to_string(),
            description: "beskrvining".to_string(),
            category_id: 2, // Frukt & Grönt
        },
        ProductInfo {
            id: 1,
            name: "test produkt 1".to_string(),
            price: 24.90,
            comparison_price: "55 kr/kg".to_string(),
            image_url: "https://images.unsplash.com/photo-1571771894821-ad996211fdf4?w=500".to_string(),
            description: "beskrvining".to_string(),
            category_id: 2, // Frukt & Grönt
        },
        ProductInfo {
            id: 1,
            name: "test produkt 1".to_string(),
            price: 24.90,
            comparison_price: "55 kr/kg".to_string(),
            image_url: "https://images.unsplash.com/photo-1571771894821-ad996211fdf4?w=500".to_string(),
            description: "beskrvining".to_string(),
            category_id: 2, // Frukt & Grönt
        },
        ProductInfo {
            id: 1,
            name: "test produkt 1".to_string(),
            price: 24.90,
            comparison_price: "55 kr/kg".to_string(),
            image_url: "https://images.unsplash.com/photo-1571771894821-ad996211fdf4?w=500".to_string(),
            description: "beskrvining".to_string(),
            category_id: 2, // Frukt & Grönt
        },
        ProductInfo {
            id: 1,
            name: "test produkt 1".to_string(),
            price: 24.90,
            comparison_price: "55 kr/kg".to_string(),
            image_url: "https://images.unsplash.com/photo-1571771894821-ad996211fdf4?w=500".to_string(),
            description: "beskrvining".to_string(),
            category_id: 2, // Frukt & Grönt
        },
        ProductInfo {
            id: 1,
            name: "test produkt 1".to_string(),
            price: 24.90,
            comparison_price: "55 kr/kg".to_string(),
            image_url: "https://images.unsplash.com/photo-1571771894821-ad996211fdf4?w=500".to_string(),
            description: "beskrvining".to_string(),
            category_id: 2, // Frukt & Grönt
        },
        ProductInfo {
            id: 1,
            name: "test produkt 1".to_string(),
            price: 24.90,
            comparison_price: "55 kr/kg".to_string(),
            image_url: "https://images.unsplash.com/photo-1571771894821-ad996211fdf4?w=500".to_string(),
            description: "beskrvining".to_string(),
            category_id: 2, // Frukt & Grönt
        },
        ProductInfo {
            id: 1,
            name: "test produkt 1".to_string(),
            price: 24.90,
            comparison_price: "55 kr/kg".to_string(),
            image_url: "https://images.unsplash.com/photo-1571771894821-ad996211fdf4?w=500".to_string(),
            description: "beskrvining".to_string(),
            category_id: 2, // Frukt & Grönt
        },
        ProductInfo {
            id: 1,
            name: "test produkt 1".to_string(),
            price: 24.90,
            comparison_price: "55 kr/kg".to_string(),
            image_url: "https://images.unsplash.com/photo-1571771894821-ad996211fdf4?w=500".to_string(),
            description: "beskrvining".to_string(),
            category_id: 2, // Frukt & Grönt
        },
        ProductInfo {
            id: 1,
            name: "test produkt 1".to_string(),
            price: 24.90,
            comparison_price: "55 kr/kg".to_string(),
            image_url: "https://images.unsplash.com/photo-1571771894821-ad996211fdf4?w=500".to_string(),
            description: "beskrvining".to_string(),
            category_id: 2, // Frukt & Grönt
        },
        ProductInfo {
            id: 1,
            name: "test produkt 1".to_string(),
            price: 24.90,
            comparison_price: "55 kr/kg".to_string(),
            image_url: "https://images.unsplash.com/photo-1571771894821-ad996211fdf4?w=500".to_string(),
            description: "beskrvining".to_string(),
            category_id: 2, // Frukt & Grönt
        },
        ProductInfo {
            id: 1,
            name: "test produkt 1".to_string(),
            price: 24.90,
            comparison_price: "55 kr/kg".to_string(),
            image_url: "https://images.unsplash.com/photo-1571771894821-ad996211fdf4?w=500".to_string(),
            description: "beskrvining".to_string(),
            category_id: 2, // Frukt & Grönt
        },
        ProductInfo {
            id: 1,
            name: "test produkt 1".to_string(),
            price: 24.90,
            comparison_price: "55 kr/kg".to_string(),
            image_url: "https://images.unsplash.com/photo-1571771894821-ad996211fdf4?w=500".to_string(),
            description: "beskrvining".to_string(),
            category_id: 2, // Frukt & Grönt
        },
        ProductInfo {
            id: 1,
            name: "test produkt 1".to_string(),
            price: 24.90,
            comparison_price: "55 kr/kg".to_string(),
            image_url: "https://images.unsplash.com/photo-1571771894821-ad996211fdf4?w=500".to_string(),
            description: "beskrvining".to_string(),
            category_id: 2, // Frukt & Grönt
        },
        ProductInfo {
            id: 2,
            name: "test produkt 2".to_string(),
            price: 18.50,
            comparison_price: "30 kr/l".to_string(),
            image_url: "https://images.unsplash.com/photo-1563636619-e910ef49e9cf?w=500".to_string(),
            description: "beskrivning".to_string(),
            category_id: 1, // Mejeri
        },
        // Lägg till fler produkter här om du vill testa
    ]
}