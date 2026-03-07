//! Rudimentary authentication.

use crate::database::{
    Administrator, Customer, Email, Id, ProfilePicture, Role, Url, User, Username, Vendor,
};
use dioxus::prelude::*;
use dioxus_fullstack::response::Response;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use {
    crate::database::{QueryResultExt, build_pfp, connection},
    argon2::{
        Argon2, PasswordHasher as _, PasswordVerifier as _,
        password_hash::{
            PasswordHash, Salt, SaltString, errors::Error as PasswordHashError, rand_core::OsRng,
        },
    },
    dioxus_fullstack::response::IntoResponse as _,
    http::header::SET_COOKIE,
    sqlx::{query, query_as},
};

#[cfg(feature = "server")]
fn hash_password<'a>(
    password: &'a str,
    salt: Salt<'a>,
) -> Result<PasswordHash<'a>, PasswordHashError> {
    Argon2::default().hash_password(password.as_bytes(), salt)
}

#[cfg(feature = "server")]
fn verify_password(password: &str, hash: &PasswordHash) -> Result<bool, PasswordHashError> {
    // TODO: What errors are possible here?
    match Argon2::default().verify_password(password.as_bytes(), &hash) {
        Ok(()) => Ok(true),
        Err(PasswordHashError::Password) => Ok(false),
        Err(e) => Err(e),
    }
}

///
#[derive(Debug, Serialize, Deserialize)]
pub enum NewUserData {
    ///
    Customer {
        ///
        profile_picture: Url,
    },
    ///
    Vendor {
        ///
        profile_picture: Url,
        ///
        display_name: Box<str>,
        ///
        description: Box<str>,
    },
    ///
    Administrator,
}

/// Create a new user.
///
/// # Errors
///
/// Fails if:
/// - `username` or `email` is not unique.
/// - An error occurs during communication with the database.
#[server]
pub async fn create_user(
    username: Username,
    email: Email,
    password: Box<str>,
    data: NewUserData,
) -> Result<()> {
    let password_hash = hash_password(&password, (&SaltString::generate(OsRng)).into())
        .unwrap()
        .serialize();
    match data {
        NewUserData::Customer { profile_picture } => {
            query!(
                "
                WITH customer AS (
                    INSERT INTO customers (profile_picture)
                    VALUES ($4::TEXT)
                )
                INSERT INTO users (username, email, password_hash)
                VALUES ($1::TEXT, $2::TEXT, $3::TEXT)
                ",
                &*username,
                &*email,
                password_hash.as_ref(),
                &profile_picture,
            )
        },
        NewUserData::Vendor {
            profile_picture,
            display_name,
            description,
        } => {
            query!(
                "
                WITH vendor AS (
                    INSERT INTO vendors (profile_picture, display_name, description)
                    VALUES ($4::TEXT, $5, $6)
                )
                INSERT INTO users (username, email, password_hash)
                VALUES ($1::TEXT, $2::TEXT, $3::TEXT)
                ",
                &*username,
                &*email,
                password_hash.as_ref(),
                &profile_picture,
                &display_name,
                &description,
            )
        },
        NewUserData::Administrator => {
            query!(
                "
                INSERT INTO users (username, email, password_hash)
                VALUES ($1::TEXT, $2::TEXT, $3::TEXT)
                ",
                &*username,
                &*email,
                password_hash.as_ref(),
            )
        },
    }
    .execute(connection())
    .await
    .map(QueryResultExt::expect_one)
    .map_err(Into::into)
}

#[server]
pub async fn log_in(username: Username, password: Box<str>) -> Result<Response> {
    struct User {
        id: i32,
        password_hash: String,
    }
    let User { id, password_hash } = query_as!(
        User,
        "
        SELECT id, password_hash
        FROM users
        WHERE username = $1
        ",
        &*username,
    )
    .fetch_optional(connection())
    .await?
    .ok_or_else(|| todo!())?;

    verify_password(&password, &PasswordHash::new(&password_hash).unwrap())?
        .then(|| {
            (
                [(
                    SET_COOKIE,
                    format!("user_id={}; Path=/; HttpOnly=false", id),
                )],
                "",
            )
                .into_response()
        })
        .ok_or_else(|| todo!())
}

#[server]
pub async fn log_out() -> Result<Response> {
    Ok(([(SET_COOKIE, "user_id=; Path=/; Max-Age=0")], "").into_response())
}

#[cfg(feature = "server")]
struct LoginRepr {
    username: String,
    customer_pfp: Option<String>,
    vendor_pfp: Option<String>,
}

#[cfg(feature = "server")]
impl Login {
    fn from_repr(
        id: Id<User>,
        LoginRepr {
            username,
            customer_pfp,
            vendor_pfp,
        }: LoginRepr,
    ) -> Self {
        let (role, profile_picture) = build_pfp(customer_pfp, vendor_pfp);
        Self {
            id: LoginId::classify(id, role),
            username: Username::new(username.into()).expect("Invalid username."),
            profile_picture,
        }
    }
}

#[server]
pub async fn login_info(user: Id<User>) -> Result<Login> {
    query_as!(
        LoginRepr,
        "
        SELECT username, c.profile_picture AS customer_pfp, v.profile_picture AS vendor_pfp
        FROM users
        LEFT JOIN customers c ON c.id = $1
        LEFT JOIN vendors v ON v.id = $1
        ",
        user.get(),
    )
    .fetch_one(connection())
    .await
    .map(|repr| Login::from_repr(user, repr))
    .map_err(|_| todo!())
}

/// Information about a login session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Login {
    /// The ID of the logged in user.
    pub id: LoginId,
    /// The username of the logged in user.
    pub username: Username,
    /// The profile picture of the logged in user.
    pub profile_picture: ProfilePicture,
}

/// A user's role and their ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoginId {
    /// The user is a customer.
    Customer(Id<Customer>),
    /// The user is a vendor.
    Vendor(Id<Vendor>),
    /// The user is an administrator.
    Administrator(Id<Administrator>),
}

impl PartialEq<Id<Customer>> for LoginId {
    fn eq(&self, other: &Id<Customer>) -> bool {
        if let Self::Customer(id) = self {
            id == other
        } else {
            false
        }
    }
}

impl PartialEq<Id<Vendor>> for LoginId {
    fn eq(&self, other: &Id<Vendor>) -> bool {
        if let Self::Vendor(id) = self {
            id == other
        } else {
            false
        }
    }
}

impl PartialEq<Id<Administrator>> for LoginId {
    fn eq(&self, other: &Id<Administrator>) -> bool {
        if let Self::Administrator(id) = self {
            id == other
        } else {
            false
        }
    }
}

impl LoginId {
    /// Construct a `LoginId` from a generic user ID and a role.
    pub fn classify(id: Id<User>, role: Role) -> Self {
        let id = id.get();
        match role {
            Role::Customer => Self::Customer(id.into()),
            Role::Vendor => Self::Vendor(id.into()),
            Role::Administrator => Self::Administrator(id.into()),
        }
    }
}

// TODO: Remove.
mod usage {
    use super::login_info;
    use crate::database::{Id, RawId, User};
    use dioxus::prelude::*;

    #[component]
    fn Example() -> Element {
        let mut user_id = use_signal(|| None);
        _ = use_effect(move || {
            use web_sys::{HtmlDocument, wasm_bindgen::JsCast as _, window};
            if let Some(window) = window()
                && let Some(document) = window.document()
                && let Ok(html) = document.dyn_into::<HtmlDocument>()
                && let Ok(cookies) = html.cookie()
                && let Some(value) = cookies
                    .split(';')
                    .filter_map(|pair| pair.split_once('='))
                    .find(|(key, _)| key.trim() == "user_id")
                    .map(|(_, value)| value)
                && let Ok(id) = value.parse::<RawId>()
            {
                user_id.set(Some(Id::<User>::from(id)))
            }
        });
        let login_info = use_server_future(move || async move {
            if let Some(user_id) = user_id() {
                Some(login_info(user_id).await.unwrap())
            } else {
                None
            }
        })?;

        rsx! {
            match login_info() {
                Some(Some(login)) => rsx! { "Logged in as {login.username}" },
                Some(None) => rsx! { "Not logged in." },
                None => rsx! { "Loading..." },
            }
        }
    }
}
