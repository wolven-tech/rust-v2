//! Server-side cross-cutting types.
//!
//! **Layer 2, SERVER-ONLY.** Nothing in the WASM half may depend on this crate.

#![forbid(unsafe_code)]

pub mod auth;
pub mod config;

pub use crate::{
    auth::{AuthUser, Role},
    config::{ConfigError, ServerConfig},
};
