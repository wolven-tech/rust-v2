//! Routing (§6.3).
//!
//! `AppShell` is the auth gate: a `use_resource` over `GET /auth/get-session`
//! that redirects to `/login` when it resolves to `None`. That is the direct
//! analogue of rust-v1's Next.js `proxy.ts` middleware redirect (§7 item 4),
//! moved client-side.

use dioxus::prelude::*;
use uuid::Uuid;

use crate::views::{Dashboard, Login, PostDetail, Posts, Shell};

#[derive(Clone, Debug, PartialEq, Routable)]
pub enum Route {
    #[layout(Shell)]
    #[route("/")]
    Dashboard {},
    #[route("/posts")]
    Posts {},
    #[route("/posts/:id")]
    PostDetail { id: Uuid },
    #[end_layout]
    #[route("/login")]
    Login {},
}
