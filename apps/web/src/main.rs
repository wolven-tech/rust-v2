//! The public marketing site.
//!
//! **D8: Dioxus fullstack in SSG mode**, built with `dx bundle --web --ssg` and
//! deployed as static files. It is different from `apps/app` because it is
//! public and needs crawlable HTML — a CSR SPA hands a search engine an empty
//! `<div>`.
//!
//! ## Honest status (R1, OQ-10)
//!
//! The doc flags Dioxus as the least-charted decision and records OQ-10: the
//! exact `dx bundle --web --ssg` invocation **with `--package`** in a
//! multi-app workspace was never verified, nor whether the SSG server function
//! coexists with an app that otherwise defines none.
//!
//! This crate is therefore built to the point where `cargo build` and the
//! `wasm32` cross-compile both pass, with the `server` feature declared and the
//! content real. The `IncrementalRendererConfig` + `static_routes`
//! server-function wiring that turns that into rendered HTML is **not** present
//! — see the SSG SEAM below. R1's mitigation is explicit that `apps/app` (plain
//! CSR, the best-trodden path) is proven first and `apps/web` follows, and that
//! degrading this app to CSR is a contained, reversible loss.

#![allow(non_snake_case)]

use dioxus::prelude::*;
use rv2_ui::{Card, PageHeader};

const TAILWIND: Asset = asset!("/assets/tailwind.css");

#[derive(Clone, Debug, PartialEq, Routable)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/about")]
    About {},
}

fn main() {
    dioxus::logger::initialize_default();
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND }
        Router::<Route> {}
    }
}

// ── SSG SEAM ─────────────────────────────────────────────────────────────────
// To finish D8, add a server function at endpoint "static_routes" returning
// `Route::static_routes()`, and configure
// `ServeConfig::builder().incremental(IncrementalRendererConfig::new()
//     .static_dir(...))`. Both are documented at
// dioxuslabs.com/learn/0.7/essentials/fullstack/static_site_generation, and
// both are gated behind the `server` feature so they never reach the wasm
// bundle. Left unimplemented rather than guessed at — see OQ-10.

#[component]
fn Shell(children: Element) -> Element {
    rsx! {
        div { class: "min-h-screen bg-white text-slate-900",
            nav { class: "border-b border-slate-200",
                div { class: "mx-auto flex max-w-3xl gap-6 px-6 py-4",
                    Link { class: "font-semibold", to: Route::Home {}, "rust-v2" }
                    Link { class: "text-sm text-slate-600", to: Route::About {}, "About" }
                }
            }
            main { class: "mx-auto max-w-3xl px-6 py-16", {children} }
        }
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        Shell {
            PageHeader {
                title: "All Rust. One datastore.",
                subtitle: "An event-sourced stack with AllSource as the only source of truth.",
            }
            Card {
                p { class: "text-sm text-slate-600",
                    "No Postgres, no Supabase, no TypeScript in the data path. \
                     Every fact is an event; every read model is a fold."
                }
            }
        }
    }
}

#[component]
fn About() -> Element {
    rsx! {
        Shell {
            PageHeader { title: "About" }
            Card {
                p { class: "text-sm text-slate-600",
                    "rust-v2 replaces a Next.js + Supabase monorepo with a Cargo workspace: \
                     an Axum API, two Dioxus frontends, and AllSource underneath."
                }
            }
        }
    }
}
