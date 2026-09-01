//! The authenticated dashboard, as a library so it can be unit-tested.
//!
//! **CSR SPA (D7).** No `fullstack`, no SSR: this is an authenticated
//! dashboard, so SEO is irrelevant and keeping the crate 100% `wasm32` keeps
//! the WASM boundary trivially checkable.
//!
//! All data comes from `apps/api` over REST via `rv2-client` (D6). There are no
//! server functions and no fixtures — what renders is what the API served.

#![allow(non_snake_case)]

pub mod post_card;
pub mod routes;
pub mod views;

pub use crate::routes::Route;

pub const PUBLIC_SITE_URL: &str = match option_env!("PUBLIC_SITE_URL") {
    Some(url) => url,
    None => "http://localhost:4401",
};
pub const PUBLIC_PRODUCT_NAME: &str = match option_env!("PUBLIC_PRODUCT_NAME") {
    Some(name) => name,
    None => "rust-v2",
};
