use crate::Route;
use crate::components::product_card::{offer_label, ProductCard};
use crate::database::products::{product_info, products_by_category, set_favorite, set_rating};
use crate::database::cart::set_in_shopping_cart;
use crate::database::reviews::{
    create_comment, create_reply, create_review, delete_comment, delete_review,
    product_reviews, product_reviews_as, set_vote_comment, set_vote_review,
    CommentTree, OwnReview, ProductReview,
};
use crate::database::{Category, Customer, Id, Product as DbProduct, Rating, Review, Vote};
use crate::state::GlobalState;
use dioxus::prelude::*;
use rust_decimal::prelude::ToPrimitive;
 
// Breadcrumb
 
/// Breadcrumb-komponent på kategorivägen från produktdatan
/// Använder 'product.category'
#[component]
fn Breadcrumb(
    /// Kategorivägen; '[(id, name), ...]'
    category_path: Vec<(Id<Category>, Box<str>)>,
    /// Produktnamnet sist
    product_name: Box<str>,
) -> Element {
    rsx! {
        nav { class: "flex items-center flex-wrap gap-1 text-sm text-gray-500 mb-6",
            Link {
                to: Route::Home {},
                class: "hover:text-green-700 transition-colors font-medium",
                i { class: "fa-solid fa-house text-xs mr-1" }
                "Start"
            }
            for (cat_id , cat_name) in category_path.iter() {
                i { class: "fa-solid fa-chevron-right text-[10px] text-gray-300" }
                Link {
                    to: Route::Category { id: *cat_id },
                    class: "hover:text-green-700 transition-colors font-medium truncate max-w-[140px]",
                    "{cat_name}"
                }
            }
            i { class: "fa-solid fa-chevron-right text-[10px] text-gray-300" }
            span { class: "text-gray-900 font-semibold truncate max-w-[200px]", "{product_name}" }
        }
    }
}
 
// Vote buttons
 
#[component]
fn VoteButtons(
    sum_votes: i64,
    own_vote: Option<Vote>,
    on_like: EventHandler<()>,
    on_dislike: EventHandler<()>,
) -> Element {
    let liked    = own_vote == Some(Vote::Like);
    let disliked = own_vote == Some(Vote::Dislike);
    rsx! {
        div { class: "flex items-center gap-2",
            button {
                class: if liked { "text-green-700 font-bold text-xs transition" } else { "text-gray-400 hover:text-green-700 text-xs transition" },
                onclick: move |_| on_like.call(()),
                i { class: "fa-solid fa-thumbs-up" }
            }
            span { class: "text-xs font-bold text-gray-500", "{sum_votes}" }
            button {
                class: if disliked { "text-red-500 font-bold text-xs transition" } else { "text-gray-400 hover:text-red-500 text-xs transition" },
                onclick: move |_| on_dislike.call(()),
                i { class: "fa-solid fa-thumbs-down" }
            }
        }
    }
}
 
// Comment node
 
#[component]
fn CommentNode(
    comment: CommentTree,
    customer_id: Option<Id<Customer>>,
    /// Vendor-ID för produkten; bara vendorn får svara på kommentarer
    vendor_id: Id<crate::database::Vendor>,
    review_id: Id<Review>,
    depth: u8,
    on_refresh: EventHandler<()>,
) -> Element {
    let mut show_reply    = use_signal(|| false);
    let mut reply_text    = use_signal(String::new);
    let mut reply_loading = use_signal(|| false);
 
    let comment_id = comment.id;
    let sum_votes  = comment.sum_votes;
    let own_vote   = comment.own_vote;
 
    let gs = use_context::<Signal<GlobalState>>();
    let is_vendor = gs.read().login.as_ref().is_some_and(|l| {
        matches!(l.id, crate::database::LoginId::Vendor(vid) if vid == vendor_id)
    });
    let can_reply = is_vendor && depth < 5;
    // Kommentarens författare eller vendorn kan ta bort kommentar
    let is_own_comment = customer_id.is_some_and(|cid| cid.get() == comment.user_id.get());
    let can_delete = is_own_comment || is_vendor;
 
    rsx! {
        div { class: if depth == 0 { "border-l-2 border-gray-100 pl-4" } else { "border-l-2 border-gray-50 pl-3 mt-2" },
            div { class: "flex items-start gap-3 py-2",
                div { class: "w-7 h-7 rounded-full bg-gray-100 flex items-center justify-center shrink-0",
                    i { class: "fa-solid fa-user text-gray-400 text-xs" }
                }
                div { class: "flex-1 min-w-0",
                    div { class: "flex items-center gap-2 flex-wrap mb-0.5",
                        span { class: "font-bold text-xs text-gray-800", "{comment.username}" }
                        // Rollbadge: vendorn visas med grön badge
                        if comment.user_id.get() == vendor_id.get() {
                            span { class: "text-[10px] bg-green-100 text-green-700 px-1.5 py-0.5 rounded font-bold",
                                "Säljare"
                            }
                        }
                        span { class: "text-gray-300 text-xs", "{comment.created_at}" }
                    }
                    p { class: "text-sm text-gray-700 leading-relaxed", "{comment.content}" }
                    div { class: "flex items-center gap-4 mt-1",
                        VoteButtons {
                            sum_votes,
                            own_vote,
                            on_like: move |_| {
                                if let Some(cid) = customer_id {
                                    let new_vote = if own_vote == Some(Vote::Like) {
                                        None
                                    } else {
                                        Some(Vote::Like)
                                    };
                                    #[allow(unused_results)]
                                    spawn(async move {
                                        drop(set_vote_comment(cid, comment_id, new_vote).await);
                                    });
                                }
                            },
                            on_dislike: move |_| {
                                if let Some(cid) = customer_id {
                                    let new_vote = if own_vote == Some(Vote::Dislike) {
                                        None
                                    } else {
                                        Some(Vote::Dislike)
                                    };
                                    #[allow(unused_results)]
                                    spawn(async move {
                                        drop(set_vote_comment(cid, comment_id, new_vote).await);
                                    });
                                }
                            },
                        }
                        if can_reply {
                            button {
                                class: "text-xs text-gray-400 hover:text-green-700 transition font-semibold",
                                onclick: move |_| show_reply.toggle(),
                                i { class: "fa-solid fa-reply mr-1" }
                                "Svara"
                            }
                        }
                        if can_delete {
                            button {
                                class: "text-xs text-gray-400 hover:text-red-500 transition font-semibold",
                                onclick: move |_| {
                                    #[allow(unused_results)]
                                    spawn(async move {
                                        drop(delete_comment(comment_id).await);
                                        on_refresh.call(());
                                    });
                                },
                                i { class: "fa-solid fa-trash text-[10px] mr-1" }
                                "Ta bort"
                            }
                        }
                    }
                    if show_reply() && can_reply {
                        div { class: "mt-2",
                            textarea {
                                class: "w-full border border-gray-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-green-500 resize-none",
                                rows: 2,
                                placeholder: "Skriv ett svar...",
                                value: "{reply_text}",
                                oninput: move |e| reply_text.set(e.value()),
                            }
                            div { class: "flex gap-2 mt-1",
                                button {
                                    class: "text-xs text-gray-500 hover:text-gray-700",
                                    onclick: move |_| {
                                        show_reply.set(false);
                                        reply_text.set(String::new());
                                    },
                                    "Avbryt"
                                }
                                button {
                                    class: if reply_loading() { "text-xs bg-gray-300 text-gray-500 px-3 py-1 rounded-lg cursor-not-allowed" } else { "text-xs bg-green-700 text-white px-3 py-1 rounded-lg hover:bg-green-800 transition" },
                                    disabled: reply_loading(),
                                    onclick: move |_| {
                                        if let Some(cid) = customer_id {
                                            let text = reply_text().trim().to_string();
                                            if text.is_empty() {
                                                return;
                                            }
                                            reply_loading.set(true);
                                            let user_id: Id<crate::database::User> = cid.into();
                                            #[allow(unused_results)]
                                            spawn(async move {
                                                drop(create_reply(user_id, comment_id, text.into()).await);
                                                reply_loading.set(false);
                                                show_reply.set(false);
                                                reply_text.set(String::new());
                                                on_refresh.call(());
                                            });
                                        }
                                    },
                                    if reply_loading() {
                                        "Skickar..."
                                    } else {
                                        "Skicka"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            for reply in comment.replies.iter() {
                CommentNode {
                    comment: reply.clone(),
                    customer_id,
                    vendor_id,
                    review_id,
                    depth: depth + 1,
                    on_refresh,
                }
            }
        }
    }
}
 
// Review card
 
#[component]
fn ReviewCard(
    review: ProductReview,
    customer_id: Option<Id<Customer>>,
    vendor_id: Id<crate::database::Vendor>,
    /// Om inloggad kund har köpt produkten
    is_buyer: bool,
    on_refresh: EventHandler<()>,
) -> Element {
    let mut show_comment   = use_signal(|| false);
    let mut comment_text   = use_signal(String::new);
    let mut comment_loading = use_signal(|| false);
 
    let review_id  = review.id;
    let sum_votes  = review.sum_votes;
    let own_vote   = review.own_vote;
    let full_stars = review.rating.get().get() as usize;
 
    let gs = use_context::<Signal<GlobalState>>();
    let is_vendor = gs.read().login.as_ref().is_some_and(|l| {
        matches!(l.id, crate::database::LoginId::Vendor(vid) if vid == vendor_id)
    });
    // Köpare och vendorn kan kommentera
    let can_comment = (is_buyer || is_vendor) && customer_id.is_some();
 
    rsx! {
        div { class: "bg-white border border-gray-100 rounded-2xl p-5 shadow-sm",
            div { class: "flex items-start gap-3 mb-3",
                div { class: "w-10 h-10 rounded-full bg-green-100 flex items-center justify-center shrink-0",
                    i { class: "fa-solid fa-user text-green-700 text-sm" }
                }
                div { class: "flex-1",
                    div { class: "flex items-center gap-2 flex-wrap",
                        span { class: "font-bold text-gray-900", "{review.username}" }
                        span { class: "text-gray-300 text-xs", "{review.created_at}" }
                    }
                    div { class: "flex gap-0.5 mt-1",
                        for i in 0..5_usize {
                            i { class: if i < full_stars { "fa-solid fa-star text-yellow-400 text-sm" } else { "fa-regular fa-star text-yellow-400 text-sm" } }
                        }
                    }
                }
                VoteButtons {
                    sum_votes,
                    own_vote,
                    on_like: move |_| {
                        if let Some(cid) = customer_id {
                            let new_vote = if own_vote == Some(Vote::Like) {
                                None
                            } else {
                                Some(Vote::Like)
                            };
                            #[allow(unused_results)]
                            spawn(async move {
                                drop(set_vote_review(cid, review_id, new_vote).await);
                            });
                        }
                    },
                    on_dislike: move |_| {
                        if let Some(cid) = customer_id {
                            let new_vote = if own_vote == Some(Vote::Dislike) {
                                None
                            } else {
                                Some(Vote::Dislike)
                            };
                            #[allow(unused_results)]
                            spawn(async move {
                                drop(set_vote_review(cid, review_id, new_vote).await);
                            });
                        }
                    },
                }
            }
            p { class: "font-bold text-gray-900 mb-1", "{review.title}" }
            p { class: "text-gray-600 text-sm leading-relaxed mb-3", "{review.content}" }

            if !review.comments.is_empty() {
                div { class: "mt-3 space-y-1",
                    for comment in review.comments.iter() {
                        CommentNode {
                            comment: comment.clone(),
                            customer_id,
                            vendor_id,
                            review_id,
                            depth: 0,
                            on_refresh,
                        }
                    }
                }
            }

            if can_comment {
                div { class: "mt-3 pt-3 border-t border-gray-50",
                    if !show_comment() {
                        button {
                            class: "text-xs text-gray-400 hover:text-green-700 transition font-semibold",
                            onclick: move |_| show_comment.set(true),
                            i { class: "fa-solid fa-comment mr-1" }
                            "Kommentera"
                        }
                    } else {
                        div {
                            textarea {
                                class: "w-full border border-gray-200 rounded-xl px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-green-500 resize-none",
                                rows: 2,
                                placeholder: "Skriv en kommentar...",
                                value: "{comment_text}",
                                oninput: move |e| comment_text.set(e.value()),
                            }
                            div { class: "flex gap-2 mt-1",
                                button {
                                    class: "text-xs text-gray-500 hover:text-gray-700",
                                    onclick: move |_| {
                                        show_comment.set(false);
                                        comment_text.set(String::new());
                                    },
                                    "Avbryt"
                                }
                                button {
                                    class: if comment_loading() { "text-xs bg-gray-300 text-gray-500 px-3 py-1 rounded-lg cursor-not-allowed" } else { "text-xs bg-green-700 text-white px-3 py-1 rounded-lg hover:bg-green-800 transition" },
                                    disabled: comment_loading(),
                                    onclick: move |_| {
                                        if let Some(cid) = customer_id {
                                            let text = comment_text().trim().to_string();
                                            if text.is_empty() {
                                                return;
                                            }
                                            comment_loading.set(true);
                                            let user_id: Id<crate::database::User> = cid.into();
                                            #[allow(unused_results)]
                                            spawn(async move {
                                                drop(create_comment(user_id, review_id, text.into()).await);
                                                comment_loading.set(false);
                                                show_comment.set(false);
                                                comment_text.set(String::new());
                                                on_refresh.call(());
                                            });
                                        }
                                    },
                                    if comment_loading() {
                                        "Skickar..."
                                    } else {
                                        "Skicka"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
 
// Own review card
 
#[component]
fn OwnReviewCard(
    review: OwnReview,
    customer_id: Id<Customer>,
    vendor_id: Id<crate::database::Vendor>,
    on_refresh: EventHandler<()>,
) -> Element {
    let full_stars   = review.rating.get().get() as usize;
    let review_id    = review.id;
    let mut confirm_delete = use_signal(|| false);
    let mut deleting       = use_signal(|| false);
 
    rsx! {
        div { class: "bg-green-50 border border-green-200 rounded-2xl p-5 mb-2",
            div { class: "flex items-center gap-2 mb-3",
                span { class: "text-xs bg-green-700 text-white px-2 py-0.5 rounded-full font-bold",
                    "Din recension"
                }
                div { class: "flex gap-0.5",
                    for i in 0..5_usize {
                        i { class: if i < full_stars { "fa-solid fa-star text-yellow-400 text-sm" } else { "fa-regular fa-star text-yellow-400 text-sm" } }
                    }
                }
                span { class: "text-gray-400 text-xs ml-auto", "{review.created_at}" }
                // Ta bort-knapp
                if !confirm_delete() {
                    button {
                        class: "text-xs text-gray-400 hover:text-red-500 transition ml-2",
                        onclick: move |_| confirm_delete.set(true),
                        i { class: "fa-solid fa-trash text-[10px] mr-1" }
                        "Ta bort"
                    }
                }
            }

            // Bekräftelsepanel
            if confirm_delete() {
                div { class: "bg-red-50 border border-red-200 rounded-xl p-3 mb-3 flex items-center justify-between gap-3",
                    p { class: "text-sm text-red-700 font-semibold",
                        i { class: "fa-solid fa-triangle-exclamation mr-2" }
                        "Ta bort recensionen och alla kommentarer på den?"
                    }
                    div { class: "flex gap-2 shrink-0",
                        button {
                            class: "text-xs text-gray-500 hover:text-gray-700 font-bold px-3 py-1 border border-gray-200 rounded-lg",
                            onclick: move |_| confirm_delete.set(false),
                            "Avbryt"
                        }
                        button {
                            class: if deleting() { "text-xs bg-gray-300 text-gray-500 px-3 py-1 rounded-lg cursor-not-allowed font-bold" } else { "text-xs bg-red-600 text-white px-3 py-1 rounded-lg hover:bg-red-700 transition font-bold" },
                            disabled: deleting(),
                            onclick: move |_| {
                                deleting.set(true);
                                #[allow(unused_results)]
                                spawn(async move {
                                    drop(delete_review(review_id).await);
                                    on_refresh.call(());
                                });
                            },
                            if deleting() {
                                "Tar bort..."
                            } else {
                                "Ja, ta bort"
                            }
                        }
                    }
                }
            }

            p { class: "font-bold text-gray-900 mb-1", "{review.title}" }
            p { class: "text-gray-600 text-sm leading-relaxed mb-3", "{review.content}" }
            if !review.comments.is_empty() {
                div { class: "mt-3 space-y-1",
                    for comment in review.comments.iter() {
                        CommentNode {
                            comment: comment.clone(),
                            customer_id: Some(customer_id),
                            vendor_id,
                            review_id,
                            depth: 0,
                            on_refresh,
                        }
                    }
                }
            }
        }
    }
}
 
// Reviews section
 
#[component]
fn ReviewsSection(
    product_id: Id<DbProduct>,
    vendor_id: Id<crate::database::Vendor>,
    customer_id: Option<Id<Customer>>,
    has_purchased: bool,
    /// Kundens betyg om de har köpt produkten
    existing_rating: Option<Rating>,
) -> Element {
    let mut selected_rating = use_signal(|| existing_rating.map(|r| r.get().get()).unwrap_or(0));
    let mut rating_saved    = use_signal(|| existing_rating.is_some());
    let mut rating_loading  = use_signal(|| false);
 
    let mut review_title   = use_signal(String::new);
    let mut review_content = use_signal(String::new);
    let mut review_loading = use_signal(|| false);
    let mut review_error   = use_signal(|| None::<String>);
    let mut review_sent    = use_signal(|| false);
 
    let mut refresh = use_signal(|| 0_u32);
 
    let reviews_data = use_resource(move || {
        let _t = refresh();
        async move {
            match customer_id {
                Some(cid) => product_reviews_as(cid, product_id, 20, 0)
                    .await
                    .map(|(own, others)| (own, others.into_vec())),
                None => product_reviews(product_id, 20, 0)
                    .await
                    .map(|r| (None, r.into_vec())),
            }
        }
    });
 
    let max_chars = 500_usize;
 
    rsx! {
        div { class: "max-w-3xl",
            h2 { class: "text-2xl font-black mb-6 text-gray-900", "Recensioner" }

            // Betyg + recensionsformulär
            if has_purchased {
                if let Some(cid) = customer_id {
                    div { class: "bg-green-50 border border-green-100 rounded-2xl p-6 mb-8",
                        h3 { class: "font-black text-lg mb-4 text-green-900", "Ditt betyg" }
                        // Stjärnor
                        div { class: "flex items-center gap-2 mb-4",
                            for star in 1_u8..=5 {
                                button {
                                    class: if selected_rating() >= star { "text-yellow-400 text-3xl transition-transform hover:scale-110" } else { "text-gray-300 text-3xl transition-transform hover:scale-110 hover:text-yellow-300" },
                                    onclick: move |_| {
                                        selected_rating.set(star);
                                        rating_saved.set(false);
                                    },
                                    i { class: "fa-solid fa-star" }
                                }
                            }
                            if rating_loading() {
                                span { class: "text-xs text-gray-400 ml-2", "Sparar..." }
                            } else if rating_saved() {
                                span { class: "text-xs text-green-700 font-bold ml-2",
                                    i { class: "fa-solid fa-check mr-1" }
                                    "Sparat"
                                }
                            } else if selected_rating() > 0 {
                                button {
                                    class: "text-xs bg-green-700 text-white px-3 py-1 rounded-full ml-2 hover:bg-green-800 transition",
                                    onclick: move |_| {
                                        if let Some(r) = Rating::new(selected_rating()) {
                                            rating_loading.set(true);
                                            #[allow(unused_results)]
                                            spawn(async move {
                                                drop(set_rating(cid, product_id, r).await);
                                                rating_loading.set(false);
                                                rating_saved.set(true);
                                            });
                                        }
                                    },
                                    "Spara betyg"
                                }
                            }
                        }

                        // Recensionsformulär
                        if selected_rating() > 0 && !review_sent() {
                            div { class: "border-t border-green-200 pt-4",
                                p { class: "text-sm font-bold text-green-900 mb-3",
                                    "Skriv en recension (valfritt)"
                                }
                                input {
                                    r#type: "text",
                                    class: "w-full border border-green-200 rounded-xl px-3 py-2 mb-2 text-sm focus:outline-none focus:ring-2 focus:ring-green-500",
                                    placeholder: "Rubrik",
                                    maxlength: "100",
                                    value: "{review_title}",
                                    oninput: move |e| review_title.set(e.value()),
                                }
                                textarea {
                                    class: "w-full border border-green-200 rounded-xl px-3 py-2 mb-1 text-sm focus:outline-none focus:ring-2 focus:ring-green-500 resize-none",
                                    rows: 4,
                                    maxlength: "{max_chars}",
                                    placeholder: "Berätta mer om produkten...",
                                    value: "{review_content}",
                                    oninput: move |e| review_content.set(e.value()),
                                }
                                div { class: "flex justify-between items-center",
                                    span { class: "text-xs text-gray-400",
                                        "{review_content().len()} / {max_chars} tecken"
                                    }
                                    button {
                                        class: if review_loading() || review_title().trim().is_empty()
    || review_content().trim().is_empty() { "bg-gray-200 text-gray-400 px-5 py-2 rounded-full text-sm font-bold cursor-not-allowed" } else { "bg-green-700 text-white px-5 py-2 rounded-full text-sm font-bold hover:bg-green-800 transition" },
                                        disabled: review_loading() || review_title().trim().is_empty()
                                            || review_content().trim().is_empty(),
                                        onclick: move |_| {
                                            let title = review_title().trim().to_string();
                                            let content = review_content().trim().to_string();
                                            if title.is_empty() || content.is_empty() {
                                                return;
                                            }
                                            review_loading.set(true);
                                            review_error.set(None);
                                            #[allow(unused_results)]
                                            spawn(async move {
                                                match create_review(cid, product_id, title.into(), content.into()).await {
                                                    Ok(()) => {
                                                        review_sent.set(true);
                                                        review_loading.set(false);
                                                        refresh.set(refresh() + 1);
                                                    }
                                                    Err(e) => {
                                                        review_error.set(Some(e.to_string()));
                                                        review_loading.set(false);
                                                    }
                                                }
                                            });
                                        },
                                        if review_loading() {
                                            "Skickar..."
                                        } else {
                                            "Skicka recension"
                                        }
                                    }
                                }
                                if let Some(err) = review_error() {
                                    p { class: "text-red-500 text-xs mt-2",
                                        i { class: "fa-solid fa-triangle-exclamation mr-1" }
                                        "{err}"
                                    }
                                }
                            }
                        }
                        if review_sent() {
                            p { class: "text-green-700 text-sm font-bold mt-2",
                                i { class: "fa-solid fa-check mr-2" }
                                "Recension skickad!"
                            }
                        }
                    }
                }
            } else if customer_id.is_none() {
                div { class: "bg-gray-50 border border-gray-200 rounded-2xl p-4 mb-6 text-sm text-gray-500 text-center",
                    i { class: "fa-solid fa-lock mr-2" }
                    "Logga in för att sätta betyg och skriva recensioner."
                }
            } else {
                div { class: "bg-gray-50 border border-gray-200 rounded-2xl p-4 mb-6 text-sm text-gray-500 text-center",
                    i { class: "fa-solid fa-bag-shopping mr-2" }
                    "Du måste ha köpt produkten för att kunna sätta betyg och skriva recension."
                }
            }

            // Recensionslista
            {
                let rev_read = reviews_data.read();
                if rev_read.is_none() {
                    rsx! {
                        p { class: "text-gray-400 animate-pulse", "Laddar recensioner..." }
                    }
                } else if let Some(Err(e)) = rev_read.as_ref() {
                    rsx! {
                        p { class: "text-red-400 text-sm", "Fel: {e}" }
                    }
                } else if let Some(Ok((own_review, other_reviews))) = rev_read.as_ref() {
                    rsx! {
                        if let Some(own) = own_review {
                            OwnReviewCard {
                                review: own.clone(),
                                customer_id: customer_id.unwrap(),
                                vendor_id,
                                on_refresh: move |_| refresh.set(refresh() + 1),
                            }
                        }

                        if other_reviews.is_empty() && own_review.is_none() {
                            div { class: "text-center py-12 bg-gray-50 rounded-2xl border-2 border-dashed border-gray-200",
                                i { class: "fa-regular fa-comment text-4xl text-gray-200 mb-3" }
                                p { class: "text-gray-400 font-semibold", "Inga recensioner ännu." }
                                p { class: "text-gray-300 text-sm mt-1", "Bli den första att recensera produkten." }
                            }
                        } else {
                            div { class: "space-y-4",
                                for review in other_reviews.iter() {
                                    ReviewCard {
                                        review: review.clone(),
                                        customer_id,
                                        vendor_id,
                                        is_buyer: has_purchased,
                                        on_refresh: move |_| refresh.set(refresh() + 1),
                                    }
                                }
                            }
                        }
                    }
                } else {
                    rsx! {}
                }
            }
        }
    }
}
 
// Similar products
 
#[derive(Props, Clone, PartialEq)]
struct SimilarProductsProps {
    category_id: Id<Category>,
    exclude_id: Id<DbProduct>,
}
 
#[component]
fn SimilarProducts(props: SimilarProductsProps) -> Element {
    let category_id = props.category_id;
    let exclude_id  = props.exclude_id;
    let similar = use_resource(move || async move {
        products_by_category(None, category_id, Some(exclude_id), 4, 0).await
    });
    let sim_read = similar.read();
    if sim_read.is_none() {
        rsx! {
            p { class: "text-gray-400 animate-pulse", "Laddar liknande produkter..." }
        }
    } else if let Some(Err(_)) = sim_read.as_ref() {
        rsx! {
            p { class: "text-gray-400", "Kunde inte hämta liknande produkter." }
        }
    } else if let Some(Ok(items)) = sim_read.as_ref() {
        if items.is_empty() {
            rsx! {
                p { class: "text-gray-400", "Inga liknande produkter hittades." }
            }
        } else {
            rsx! {
                div { class: "grid grid-cols-2 md:grid-cols-4 gap-6",
                    for p in items.iter() {
                        ProductCard {
                            id: p.id.get(),
                            name: p.name.clone(),
                            price: p.price.to_f64().unwrap_or_default(),
                            comparison_price: format!("{:.2} kr / {}", p.price, p.amount_per_unit),
                            image_url: p.thumbnail.to_string(),
                            in_stock: p.in_stock.into(),
                            special_offer: offer_label(p.special_offer_deal, p.price),
                        }
                    }
                }
            }
        }
    } else {
        rsx! {}
    }
}
 
// Product page
 
/// Product page
/// # Arguments
/// * `id` - The product ID to display.
#[allow(clippy::same_name_method, non_snake_case)]
#[component]
pub fn Product(id: i32) -> Element {
    let mut global_state = use_context::<Signal<GlobalState>>();
    let db_id = Id::<DbProduct>::from(id);
 
    let login       = global_state.read().login.clone();
    let customer_id = login.as_ref().and_then(|l| {
        if let crate::database::LoginId::Customer(cid) = l.id { Some(cid) } else { None }
    });
    let is_favorite = global_state.read().favorites.contains(&id);
    let quantity    = global_state.read().cart.iter()
        .find(|i| i.product_id == id)
        .map(|i| i.quantity)
        .unwrap_or(0);
    let heart_class = if is_favorite { "text-red-500" } else { "text-gray-400 hover:text-red-500" };
 
    // Hämta produktinfo med customer_id så vi får has_purchased och own_rating
    let product_resource = use_resource(move || async move { product_info(customer_id, db_id).await });
 
    let prod_read = product_resource.read_unchecked();
    if prod_read.is_none() {
        return rsx! {
            div { class: "max-w-6xl mx-auto p-4 md:p-8 bg-white",
                div { class: "flex justify-center items-center py-20",
                    p { class: "text-xl font-bold text-gray-400 animate-pulse",
                        "Hämtar produkt från databasen..."
                    }
                }
            }
        };
    }
    if let Some(Err(e)) = prod_read.as_ref() {
        let e = e.to_string();
        return rsx! {
            div { class: "max-w-6xl mx-auto p-4 md:p-8 bg-white",
                div { class: "text-center p-20",
                    h2 { class: "text-red-500 text-2xl font-black", "Ett fel uppstod" }
                    p { class: "text-gray-500", "{e}" }
                }
            }
        };
    }
    let product = match prod_read.as_ref() {
        Some(Ok(p)) => p,
        _ => return rsx! {},
    };
    // Breadcrumb: category_path
    let mut crumbs: Vec<(Id<Category>, Box<str>)> = product.category.iter().cloned().collect();
    crumbs.reverse();

    let formatted_price  = format!("{:.2}", product.price).replace('.', ",");
    let avg_rating       = product.rating.rating().unwrap_or(0.0);
    let rating_count     = product.rating.count();
    let full_stars       = avg_rating.round() as usize;
    let pname  = product.name.to_string();
    let pprice = product.price.to_string().parse::<f64>().unwrap_or(0.0);
    let pimage = product.gallery.first().map(|u| u.to_string()).unwrap_or_default();
    let category_id  = product.category.last().map(|(cat_id, _)| *cat_id);
    let amount_str   = product.amount_per_unit.to_string();
    let vendor_id    = product.vendor_id;
    let has_purchased = product.has_purchased;
    // Befintligt betyg
    let existing_rating = has_purchased
        .then(|| Rating::new(product.own_rating.get().get()))
        .flatten();

    let offer_label: Option<String> = match product
        .special_offer_deal
        .and_then(|d| d.database_repr())
    {
        Some((None, Some(take), Some(pay_for))) =>
            Some(format!("Köp {take} st, betala för {pay_for} st")),
        Some((Some(pay), Some(take), None)) => {
            let pay_fmt = format!("{:.2}", pay).replace('.', ",");
            Some(format!("{take} för {pay_fmt} kr"))
        }
        Some((Some(new_price_val), None, None)) => {
            let new_fmt = format!("{:.2}", new_price_val).replace('.', ",");
            Some(format!("Rea: {new_fmt} kr (ord. {formatted_price} kr)"))
        }
        _ => None,
    };

    rsx! {
        // Breadcrumb
        Breadcrumb { category_path: crumbs, product_name: product.name.clone() }

        // Produkt-grid: bild + info
        div { class: "grid grid-cols-1 md:grid-cols-2 gap-12 mb-16",
            // Bildkolumn
            div { class: "flex flex-col items-center",
                div { class: "bg-gray-50 rounded-xl p-8 w-full flex justify-center",
                    if let Some(img) = product.gallery.first() {
                        img {
                            src: "{img}",
                            class: "max-h-[400px] object-contain shadow-sm",
                        }
                    }
                }
                div { class: "mt-6 flex flex-col items-center gap-2",
                    div { class: "flex text-yellow-400 text-xl",
                        for i in 0..5_usize {
                            if i < full_stars {
                                i { class: "fa-solid fa-star" }
                            } else {
                                i { class: "fa-regular fa-star" }
                            }
                        }
                    }
                    span { class: "text-gray-500 text-sm font-medium",
                        "{avg_rating:.1} av 5 ({rating_count} recensioner)"
                    }
                }
            }

            // Infokolumn
            div { class: "flex flex-col justify-start",
                h1 { class: "text-4xl font-black text-gray-900 mb-2", "{product.name}" }
                p { class: "text-gray-500 text-lg mb-4", "{product.description}" }

                div { class: "border-t border-b py-6 mb-6",
                    div { class: "text-red-600 font-black text-5xl mb-1", "{formatted_price} kr" }
                    div { class: "flex items-center gap-2 mt-2 mb-2",
                        span { class: "bg-gray-100 text-gray-700 text-sm font-semibold px-3 py-1 rounded-full",
                            "{amount_str} / förpackning"
                        }
                    }
                    if let Some(label) = offer_label {
                        div { class: "inline-flex items-center bg-green-100 text-green-800 font-bold text-sm px-4 py-2 rounded-full mt-1",
                            "{label}"
                        }
                    }
                    div { class: "text-gray-500 font-bold mt-3",
                        "Säljs av "
                        Link {
                            to: Route::Vendor { id: vendor_id },
                            class: "text-green-700 hover:underline",
                            "{product.vendor_name}"
                        }
                    }
                }

                // Varukorg + hjärta
                div { class: "flex gap-4 items-center h-16",
                    if quantity == 0 {
                        button {
                            class: "flex-grow h-full bg-green-700 text-white rounded-full font-black text-xl hover:bg-green-800 transition-colors shadow-md flex items-center justify-center gap-3",
                            onclick: move |_| {
                                global_state.write().add_to_cart(id, pname.clone(), pprice, pimage.clone());
                                if let Some(cid) = global_state.read().customer_id() {
                                    let pid = Id::<DbProduct>::from(id);
                                    #[allow(unused_results)]
                                    spawn(async move {
                                        drop(set_in_shopping_cart(cid, pid, 1).await);
                                    });
                                }
                            },
                            i { class: "fa-solid fa-cart-plus" }
                            "LÄGG I VARUKORG"
                        }
                    } else {
                        div { class: "flex-grow h-full flex items-center justify-between bg-green-100 rounded-full overflow-hidden border-2 border-green-700",
                            button {
                                class: "px-8 h-full bg-green-700 text-white font-bold text-2xl",
                                onclick: move |_| {
                                    let new_qty = quantity - 1;
                                    global_state.write().set_quantity(id, new_qty);
                                    if let Some(cid) = global_state.read().customer_id() {
                                        let pid = Id::<DbProduct>::from(id);
                                        #[allow(unused_results)]
                                        spawn(async move {
                                            drop(set_in_shopping_cart(cid, pid, new_qty).await);
                                        });
                                    }
                                },
                                i { class: "fas fa-minus" }
                            }
                            span { class: "font-black text-2xl text-green-900", "{quantity}" }
                            button {
                                class: "px-8 h-full bg-green-700 text-white font-bold text-2xl",
                                onclick: move |_| {
                                    let new_qty = quantity + 1;
                                    global_state.write().set_quantity(id, new_qty);
                                    if let Some(cid) = global_state.read().customer_id() {
                                        let pid = Id::<DbProduct>::from(id);
                                        #[allow(unused_results)]
                                        spawn(async move {
                                            drop(set_in_shopping_cart(cid, pid, new_qty).await);
                                        });
                                    }
                                },
                                i { class: "fas fa-plus" }
                            }
                        }
                    }
                    // Favorit-knapp
                    button {
                        class: "h-full px-6 border-2 border-gray-200 rounded-full transition-all {heart_class}",
                        onclick: move |_| {
                            let new_fav = !global_state.read().favorites.contains(&id);
                            {
                                let mut s = global_state.write();
                                if new_fav {
                                    if !s.favorites.contains(&id) {
                                        s.favorites.push(id);
                                    }
                                } else {
                                    s.favorites.retain(|&x| x != id);
                                }
                            }
                            if let Some(cid) = customer_id {
                                let pid = Id::<DbProduct>::from(id);
                                #[allow(unused_results)]
                                spawn(async move {
                                    drop(set_favorite(cid, pid, new_fav).await);
                                });
                            }
                        },
                        i { class: if is_favorite { "fa-solid fa-heart text-2xl" } else { "fa-regular fa-heart text-2xl" } }
                    }
                }
            }
        }

        // Liknande produkter
        div { class: "border-t pt-16 mb-16",
            h2 { class: "text-3xl font-black mb-8 text-gray-900", "Liknande produkter" }
            if let Some(cat_id) = category_id {
                SimilarProducts { category_id: cat_id, exclude_id: db_id }
            } else {
                p { class: "text-gray-400", "Inga liknande produkter." }
            }
        }

        // Recensioner
        div { class: "border-t pt-16",
            ReviewsSection {
                product_id: db_id,
                vendor_id,
                customer_id,
                has_purchased,
                existing_rating,
            }
        }
    }
}