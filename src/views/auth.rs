#![allow(non_snake_case)]
use crate::Route;
use dioxus::prelude::*;

/// Kundinloggning
/// TODO(auth): ERSÄTT fake-logiken med:
/// 1. POST /login med email + bcrypt-hashat lösenord
/// 2. Server sätter cookie med user_id
/// 3. global_state.write().auth hämtas från databasen
#[component]
pub fn Login() -> Element {
    rsx! {
        div { class: "min-h-screen bg-gray-50 flex items-center justify-center p-4",
            div { class: "bg-white rounded-2xl shadow-sm p-8 w-full max-w-md",
                div { class: "text-center mb-8",
                    div { class: "text-4xl font-black italic text-green-700 mb-2", "boop" }
                    h1 { class: "text-2xl font-black text-gray-900", "Logga in priatperson" }
                }
                div { class: "space-y-4",
                    input {
                        r#type: "email",
                        placeholder: "din@epost.se",
                        class: "w-full border-2 border-gray-200 rounded-xl px-4 py-3",
                    }
                    input {
                        r#type: "password",
                        placeholder: "••••••••",
                        class: "w-full border-2 border-gray-200 rounded-xl px-4 py-3",
                    }
                    button { class: "w-full bg-green-700 text-white font-black py-3 rounded-full",
                        "Logga in"
                    }
                }
            }
        }
    }
}

#[component]
pub fn Register() -> Element {
    rsx! {
        div { class: "min-h-screen bg-gray-50 flex items-center justify-center p-4",
            div { class: "bg-white rounded-2xl shadow-sm p-8 w-full max-w-md",
                div { class: "text-center mb-8",
                    div { class: "text-4xl font-black italic text-green-700 mb-2", "boop" }
                    h1 { class: "text-2xl font-black text-gray-900", "Skapa konto privatperson" }
                }
                div { class: "space-y-4",
                    input {
                        r#type: "text",
                        placeholder: "Namn",
                        class: "w-full border-2 border-gray-200 rounded-xl px-4 py-3",
                    }
                    input {
                        r#type: "email",
                        placeholder: "E-post",
                        class: "w-full border-2 border-gray-200 rounded-xl px-4 py-3",
                    }
                    button { class: "w-full bg-green-700 text-white font-black py-3 rounded-full",
                        "Skapa konto"
                    }
                }
            }
        }
    }
}

/// Företagsinloggning
/// TODO(auth): Samma som Login men verifiera att users.is_vendor = true
#[component]
pub fn VendorLogin() -> Element {
    rsx! {
        div { class: "min-h-screen bg-gray-50 flex items-center justify-center p-4",
            div { class: "bg-white rounded-2xl shadow-sm p-8 w-full max-w-md",
                div { class: "text-center mb-8",
                    div { class: "text-4xl font-black italic text-green-700 mb-2", "boop" }
                    h1 { class: "text-2xl font-black text-gray-900", "Logga in företag" }
                }
                div { class: "space-y-4",
                    input {
                        r#type: "email",
                        placeholder: "din@epost.se",
                        class: "w-full border-2 border-gray-200 rounded-xl px-4 py-3",
                    }
                    input {
                        r#type: "password",
                        placeholder: "••••••••",
                        class: "w-full border-2 border-gray-200 rounded-xl px-4 py-3",
                    }
                    button { class: "w-full bg-green-700 text-white font-black py-3 rounded-full",
                        "Logga in"
                    }
                }
            }
        }
    }
}

/// Företagsregistrering
/// TODO(auth): POST /register/vendor; skapa vendor + user i DB; sätt cookie
#[component]
pub fn VendorRegister() -> Element {
    rsx! {
        div { class: "min-h-screen bg-gray-50 flex items-center justify-center p-4",
            div { class: "bg-white rounded-2xl shadow-sm p-8 w-full max-w-md",
                div { class: "text-center mb-8",
                    div { class: "text-4xl font-black italic text-green-700 mb-2", "boop" }
                    h1 { class: "text-2xl font-black text-gray-900", "Skapa konto Företag" }
                }
                div { class: "space-y-4",
                    input {
                        r#type: "text",
                        placeholder: "Namn",
                        class: "w-full border-2 border-gray-200 rounded-xl px-4 py-3",
                    }
                    input {
                        r#type: "email",
                        placeholder: "E-post",
                        class: "w-full border-2 border-gray-200 rounded-xl px-4 py-3",
                    }
                    button { class: "w-full bg-green-700 text-white font-black py-3 rounded-full",
                        "Skapa Buisness-konto"
                    }
                }
            }
        }
    }
}