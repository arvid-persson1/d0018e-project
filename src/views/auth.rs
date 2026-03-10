#![allow(non_snake_case)]
use crate::Route;
use crate::database::{NewUserData, create_user, log_in, login_info, Id, RawId, User, Username, Email};
use crate::state::GlobalState;
use dioxus::prelude::*;

/// Kundinloggning
#[component]
pub fn Login() -> Element {
    let global_state = use_context::<Signal<GlobalState>>();
    let nav = use_navigator();

    let mut username_val = use_signal(String::new);
    let mut password_val = use_signal(String::new);
    let mut error_msg = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);

    rsx! {
        div { class: "min-h-screen bg-gray-50 flex items-center justify-center p-4",
            div { class: "bg-white rounded-2xl shadow-sm p-8 w-full max-w-md",
                div { class: "text-center mb-8",
                    div { class: "text-4xl font-black italic text-green-700 mb-2", "boop" }
                    h1 { class: "text-2xl font-black text-gray-900", "Logga in som privatperson" }
                }
                div { class: "space-y-4",
                    input {
                        r#type: "text",
                        placeholder: "Användarnamn",
                        class: "w-full border-2 border-gray-200 rounded-xl px-4 py-3",
                        oninput: move |e| username_val.set(e.value()),
                    }
                    input {
                        r#type: "password",
                        placeholder: "••••••••",
                        class: "w-full border-2 border-gray-200 rounded-xl px-4 py-3",
                        oninput: move |e| password_val.set(e.value()),
                    }
                    if let Some(err) = error_msg() {
                        p { class: "text-red-500 text-sm", "{err}" }
                    }
                    button {
                        class: "w-full bg-green-700 text-white font-black py-3 rounded-full disabled:opacity-50",
                        disabled: loading(),
                        onclick: move |_| {
                            let uname = username_val();
                            let pwd = password_val();
                            let mut gs = global_state;
                            let nav = nav.clone();
                            let _task = spawn(async move {
                                loading.set(true);
                                error_msg.set(None);
                                let Some(username) = Username::new(uname.into()) else {
                                    error_msg.set(Some("Ogiltigt användarnamn.".into()));
                                    loading.set(false);
                                    return;
                                };
                                match log_in(username, pwd.into()).await {
                                    Ok(_) => {
                                        #[cfg(feature = "web")]
                                        {
                                            use web_sys::{HtmlDocument, wasm_bindgen::JsCast as _, window};
                                            if let Some(window) = window()
                                                && let Some(document) = window.document()
                                                && let Ok(html) = document.dyn_into::<HtmlDocument>()
                                                && let Ok(cookies) = html.cookie()
                                                && let Some(value) = cookies
                                                    .split(';')
                                                    .filter_map(|pair| pair.split_once('='))
                                                    .find(|(key, _)| key.trim() == "user_id")
                                                    .map(|(_, value)| value.trim().to_string())
                                                && let Ok(id) = value.parse::<RawId>()
                                            {
                                                if let Ok(info) = login_info(Id::<User>::from(id)).await {
                                                    gs.write().login = Some(info);
                                                }
                                            }
                                        }
                                        let _unused = nav.push(Route::Home {});
                                    }
                                    Err(_) => {
                                        error_msg.set(Some("Fel användarnamn eller lösenord.".into()));
                                    }
                                }
                                loading.set(false);
                            });
                        },
                        if loading() {
                            "Loggar in..."
                        } else {
                            "Logga in"
                        }
                    }
                }
                div { class: "text-center mt-6 text-sm text-gray-500",
                    "Har du inget konto? "
                    Link {
                        to: Route::Register {},
                        class: "text-green-700 font-bold hover:underline",
                        "Skapa konto här"
                    }
                }
                div { class: "text-center mt-2 text-sm text-gray-400",
                    "Är du företag? "
                    Link {
                        to: Route::VendorLogin {},
                        class: "text-green-700 font-bold hover:underline",
                        "Logga in som företag"
                    }
                }
            }
        }
    }
}

/// Kundregistrering
#[component]
pub fn Register() -> Element {
    let nav = use_navigator();

    let mut username_val = use_signal(String::new);
    let mut email_val = use_signal(String::new);
    let mut password_val = use_signal(String::new);
    let mut error_msg = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);

    rsx! {
        div { class: "min-h-screen bg-gray-50 flex items-center justify-center p-4",
            div { class: "bg-white rounded-2xl shadow-sm p-8 w-full max-w-md",
                div { class: "text-center mb-8",
                    div { class: "text-4xl font-black italic text-green-700 mb-2", "boop" }
                    h1 { class: "text-2xl font-black text-gray-900", "Skapa konto som privatperson" }
                }
                div { class: "space-y-4",
                    input {
                        r#type: "text",
                        placeholder: "Användarnamn",
                        class: "w-full border-2 border-gray-200 rounded-xl px-4 py-3",
                        oninput: move |e| username_val.set(e.value()),
                    }
                    input {
                        r#type: "email",
                        placeholder: "E-post",
                        class: "w-full border-2 border-gray-200 rounded-xl px-4 py-3",
                        oninput: move |e| email_val.set(e.value()),
                    }
                    input {
                        r#type: "password",
                        placeholder: "Lösenord",
                        class: "w-full border-2 border-gray-200 rounded-xl px-4 py-3",
                        oninput: move |e| password_val.set(e.value()),
                    }
                    if let Some(err) = error_msg() {
                        p { class: "text-red-500 text-sm", "{err}" }
                    }
                    button {
                        class: "w-full bg-green-700 text-white font-black py-3 rounded-full disabled:opacity-50",
                        disabled: loading(),
                        onclick: move |_| {
                            let uname = username_val();
                            let mail = email_val();
                            let pwd = password_val();
                            let nav = nav.clone();
                            let _task = spawn(async move {
                                loading.set(true);
                                error_msg.set(None);
                                let Some(username) = Username::new(uname.into()) else {
                                    error_msg.set(Some("Ogiltigt användarnamn.".into()));
                                    loading.set(false);
                                    return;
                                };
                                let Some(email) = Email::new(mail.into()) else {
                                    error_msg.set(Some("Ogiltig e-postadress.".into()));
                                    loading.set(false);
                                    return;
                                };
                                // TODO(auth): Profilbild — använd en default-URL tills vidare
                                match create_user(
                                        username,
                                        email,
                                        pwd.into(),
                                        NewUserData::Customer {
                                            profile_picture: String::new().into(),
                                        },
                                    )
                                    .await
                                {
                                    Ok(_) => {
                                        let _unused = nav.push(Route::Login {});
                                    }
                                    Err(e) => {
                                        error_msg.set(Some(format!("Fel: {e}")));
                                    }
                                }
                                loading.set(false);
                            });
                        },
                        if loading() {
                            "Skapar konto..."
                        } else {
                            "Skapa konto"
                        }
                    }
                }
                div { class: "text-center mt-6 text-sm text-gray-500",
                    "Har du redan ett konto? "
                    Link {
                        to: Route::Login {},
                        class: "text-green-700 font-bold hover:underline",
                        "Logga in här"
                    }
                }
                div { class: "text-center mt-2 text-sm text-gray-400",
                    "Är du företag? "
                    Link {
                        to: Route::VendorRegister {},
                        class: "text-green-700 font-bold hover:underline",
                        "Skapa företagskonto"
                    }
                }
            }
        }
    }
}

/// Företagsinloggning
#[component]
pub fn VendorLogin() -> Element {
    let global_state = use_context::<Signal<GlobalState>>();
    let nav = use_navigator();

    let mut username_val = use_signal(String::new);
    let mut password_val = use_signal(String::new);
    let mut error_msg = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);

    rsx! {
        div { class: "min-h-screen bg-gray-50 flex items-center justify-center p-4",
            div { class: "bg-white rounded-2xl shadow-sm p-8 w-full max-w-md",
                div { class: "text-center mb-8",
                    div { class: "text-4xl font-black italic text-green-700 mb-2", "boop" }
                    h1 { class: "text-2xl font-black text-gray-900", "Logga in för företag" }
                }
                div { class: "space-y-4",
                    input {
                        r#type: "text",
                        placeholder: "Användarnamn",
                        class: "w-full border-2 border-gray-200 rounded-xl px-4 py-3",
                        oninput: move |e| username_val.set(e.value()),
                    }
                    input {
                        r#type: "password",
                        placeholder: "••••••••",
                        class: "w-full border-2 border-gray-200 rounded-xl px-4 py-3",
                        oninput: move |e| password_val.set(e.value()),
                    }
                    if let Some(err) = error_msg() {
                        p { class: "text-red-500 text-sm", "{err}" }
                    }
                    button {
                        class: "w-full bg-green-700 text-white font-black py-3 rounded-full disabled:opacity-50",
                        disabled: loading(),
                        onclick: move |_| {
                            let uname = username_val();
                            let pwd = password_val();
                            let mut gs = global_state;
                            let nav = nav.clone();
                            let _task = spawn(async move {
                                loading.set(true);
                                error_msg.set(None);
                                let Some(username) = Username::new(uname.into()) else {
                                    error_msg.set(Some("Ogiltigt användarnamn.".into()));
                                    loading.set(false);
                                    return;
                                };
                                match log_in(username, pwd.into()).await {
                                    Ok(_) => {
                                        #[cfg(feature = "web")]
                                        {
                                            use web_sys::{HtmlDocument, wasm_bindgen::JsCast as _, window};
                                            if let Some(window) = window()
                                                && let Some(document) = window.document()
                                                && let Ok(html) = document.dyn_into::<HtmlDocument>()
                                                && let Ok(cookies) = html.cookie()
                                                && let Some(value) = cookies
                                                    .split(';')
                                                    .filter_map(|pair| pair.split_once('='))
                                                    .find(|(key, _)| key.trim() == "user_id")
                                                    .map(|(_, value)| value.trim().to_string())
                                                && let Ok(id) = value.parse::<RawId>()
                                            {
                                                if let Ok(info) = login_info(Id::<User>::from(id)).await {
                                                    gs.write().login = Some(info);
                                                }
                                            }
                                        }
                                        let _unused = nav.push(Route::Home {});
                                    }
                                    Err(_) => {
                                        error_msg.set(Some("Fel användarnamn eller lösenord.".into()));
                                    }
                                }
                                loading.set(false);
                            });
                        },
                        if loading() {
                            "Loggar in..."
                        } else {
                            "Logga in"
                        }
                    }
                }
                div { class: "text-center mt-6 text-sm text-gray-500",
                    "Har du inget företagskonto? "
                    Link {
                        to: Route::VendorRegister {},
                        class: "text-green-700 font-bold hover:underline",
                        "Skapa företagskonto här"
                    }
                }
                div { class: "text-center mt-2 text-sm text-gray-400",
                    "Är du privatperson? "
                    Link {
                        to: Route::Login {},
                        class: "text-green-700 font-bold hover:underline",
                        "Logga in som privatperson"
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
    let nav = use_navigator();

    let mut username_val = use_signal(String::new);
    let mut email_val = use_signal(String::new);
    let mut password_val = use_signal(String::new);
    let mut display_name_val = use_signal(String::new);
    let mut error_msg = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);

    rsx! {
        div { class: "min-h-screen bg-gray-50 flex items-center justify-center p-4",
            div { class: "bg-white rounded-2xl shadow-sm p-8 w-full max-w-md",
                div { class: "text-center mb-8",
                    div { class: "text-4xl font-black italic text-green-700 mb-2", "boop" }
                    h1 { class: "text-2xl font-black text-gray-900", "Skapa konto för företag" }
                }
                div { class: "space-y-4",
                    input {
                        r#type: "text",
                        placeholder: "Användarnamn",
                        class: "w-full border-2 border-gray-200 rounded-xl px-4 py-3",
                        oninput: move |e| username_val.set(e.value()),
                    }
                    input {
                        r#type: "text",
                        placeholder: "Företagsnamn",
                        class: "w-full border-2 border-gray-200 rounded-xl px-4 py-3",
                        oninput: move |e| display_name_val.set(e.value()),
                    }
                    input {
                        r#type: "email",
                        placeholder: "E-post",
                        class: "w-full border-2 border-gray-200 rounded-xl px-4 py-3",
                        oninput: move |e| email_val.set(e.value()),
                    }
                    input {
                        r#type: "password",
                        placeholder: "Lösenord",
                        class: "w-full border-2 border-gray-200 rounded-xl px-4 py-3",
                        oninput: move |e| password_val.set(e.value()),
                    }
                    if let Some(err) = error_msg() {
                        p { class: "text-red-500 text-sm", "{err}" }
                    }
                    button {
                        class: "w-full bg-green-700 text-white font-black py-3 rounded-full disabled:opacity-50",
                        disabled: loading(),
                        onclick: move |_| {
                            let uname = username_val();
                            let mail = email_val();
                            let pwd = password_val();
                            let dname = display_name_val();
                            let nav = nav.clone();
                            let _task = spawn(async move {
                                loading.set(true);
                                error_msg.set(None);
                                let Some(username) = Username::new(uname.into()) else {
                                    error_msg.set(Some("Ogiltigt användarnamn.".into()));
                                    loading.set(false);
                                    return;
                                };
                                let Some(email) = Email::new(mail.into()) else {
                                    error_msg.set(Some("Ogiltig e-postadress.".into()));
                                    loading.set(false);
                                    return;
                                };
                                match create_user(
                                        username,
                                        email,
                                        pwd.into(),
                                        NewUserData::Vendor {
                                            profile_picture: String::new().into(),
                                            display_name: dname.into(),
                                            description: "".into(),
                                        },
                                    )
                                    .await
                                {
                                    Ok(_) => {
                                        let _unused = nav.push(Route::VendorLogin {});
                                    }
                                    Err(e) => {
                                        error_msg.set(Some(format!("Fel: {e}")));
                                    }
                                }
                                loading.set(false);
                            });
                        },
                        if loading() {
                            "Skapar konto..."
                        } else {
                            "Skapa Business-konto"
                        }
                    }
                }
                div { class: "text-center mt-6 text-sm text-gray-500",
                    "Har du redan ett företagskonto? "
                    Link {
                        to: Route::VendorLogin {},
                        class: "text-green-700 font-bold hover:underline",
                        "Logga in här"
                    }
                }
                div { class: "text-center mt-2 text-sm text-gray-400",
                    "Är du privatperson? "
                    Link {
                        to: Route::Register {},
                        class: "text-green-700 font-bold hover:underline",
                        "Skapa privatkonto"
                    }
                }
            }
        }
    }
}