use crate::Id;
use crate::database::Vendor;
use dioxus::prelude::*;

// The profile page for a vendor.
// TODO(db): Replace with real vendor data
// SELECT vendors.*, users.username, users.created_at FROM vendors
// JOIN users ON vendors.id = users.id WHERE vendors.id = $id

/// The profile page for a vendor.
/// # Arguments
/// * `id` - The vendor ID.
#[allow(clippy::same_name_method, reason = "Dioxus macro limitation")]
#[component]
pub fn VendorProfile(id: Id<Vendor>) -> Element {
    // use_effect(move || {
    //     spawn(async move {
    //         match get_vendor(id).await {
    //             Ok(v) => vendor.set(Some(v)),
    //             Err(_) => {},
    //         }
    //     })
    // });

    // TODO(db): use_resource to fetch vendor by id
    // let vendor_data = use_resource(move || async move { get_vendor(id).await });

    rsx! {
        div { class: "min-h-screen bg-gray-50",
            div { class: "max-w-6xl mx-auto p-6",

                // Header
                h1 { class: "text-3xl font-black text-gray-900 mb-8",
                    "Gården AB"
                                // TODO(db): Replace with vendor.name
                }

                div { class: "flex gap-8",

                    // Left sidebar
                    div { class: "w-64 flex-shrink-0",
                        div { class: "bg-white rounded-2xl shadow-sm p-6 sticky top-6",

                            // profile pic
                            div { class: "flex flex-col items-center mb-6",
                                div { class: "w-24 h-24 rounded-full bg-green-100 flex items-center justify-center mb-3 overflow-hidden",
                                    i { class: "fa-solid fa-store text-4xl text-green-700" }
                                }
                                p {
                                    class: "font-bold text-gray-900 text-center",
                                    "Gården AB"
                                                                // TODO(db): Replace with vendor.name
                                }
                            }

                            // status
                            div { class: "space-y-3 text-sm",
                                div { class: "flex items-center gap-2 text-green-700",
                                    i { class: "fa-solid fa-circle-check" }
                                    span { "Verifierad säljare" }
                                }

                                // rating
                                div {
                                    class: "flex items-center gap-2 text-gray-700",
                                    div { class: "flex gap-0.5",
                                        i { class: "fa-solid fa-star text-yellow-400 text-xs" }
                                        i { class: "fa-solid fa-star text-yellow-400 text-xs" }
                                        i { class: "fa-solid fa-star text-yellow-400 text-xs" }
                                        i { class: "fa-solid fa-star text-yellow-400 text-xs" }
                                        i { class: "fa-regular fa-star text-yellow-400 text-xs" }
                                    }
                                    span { "4.3" }
                                                                // TODO(db): Replace with vendor.avg_rating
                                }

                                div {
                                    class: "flex items-center gap-2 text-gray-600",
                                    i { class: "fa-solid fa-box" }
                                    span { "1240 sålda ordrar" }
                                                                // TODO(db): Replace with vendor.total_sales
                                }
                                div {
                                    class: "flex items-center gap-2 text-gray-600",
                                    i { class: "fa-solid fa-tag" }
                                    span { "18 produkter" }
                                                                // TODO(db): Replace with vendor.product_count
                                }
                            }

                            // who are they
                            div { class: "mt-6 pt-6 border-t",
                                p { class: "text-sm text-gray-600",
                                    "Vi odlar och säljer ekologiska grönsaker direkt från vår gård!"
                                                                // TODO(db): Replace with vendor.description
                                }
                            }
                        }
                    }

                    // products
                    div { class: "flex-1",
                        div { class: "bg-white rounded-2xl shadow-sm p-6",
                            h2 { class: "text-xl font-black text-gray-900 mb-4",
                                i { class: "fa-solid fa-tag text-green-700 mr-2" }
                                "Produkter"
                            }

                            // TODO(db): get products by this vendor
                            div { class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4",
                                for i in 0..3_u8 {
                                    div { class: "border rounded-xl p-4 hover:shadow-md transition",
                                        div { class: "bg-gray-100 rounded-lg h-32 mb-3 flex items-center justify-center",
                                            i { class: "fa-solid fa-image text-3xl text-gray-300" }
                                        }
                                        p { class: "font-bold text-sm text-gray-900",
                                            "Exempelprodukt {i+1}"
                                        }
                                        p { class: "text-green-700 font-black text-sm mt-1",
                                            "29,90 kr"
                                        }
                                    }
                                }
                            }
                            p { class: "text-gray-400 text-xs mt-4",
                                "// TODO(db): Ersätt med riktiga produkter"
                            }
                        }
                    }
                }
            }
        }
    }
}
