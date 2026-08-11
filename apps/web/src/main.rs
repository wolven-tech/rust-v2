//! The public marketing site.
//!
//! **D8: Dioxus fullstack in SSG mode**, built with `dx bundle --web --ssg` and
//! deployed as static files. It is different from `apps/app` because it is
//! public and needs crawlable HTML — a CSR SPA hands a search engine an empty
//! `<div>`.
//!
//! ## Why this page looks like a product site
//!
//! The home page is a deliberate **coverage fixture** for `rv2-ui`. The kit's
//! scope was fixed by auditing a real marketing page
//! (chargewindow-web.fly.dev), and this page renders every element that audit
//! found: nav with a call-to-action, hero with dual actions, numbered feature
//! grid, how-it-works steps, an itemised cost breakdown, a pricing block, an
//! FAQ, and a multi-column footer.
//!
//! It is a fixture, not decoration. If a component regresses or a prop is
//! removed, this page stops compiling — which is the cheapest possible test for
//! a component library whose output is otherwise only checkable by eye.
//!
//! ## Honest status (R1, OQ-10)
//!
//! The doc flags Dioxus as the least-charted decision and records OQ-10: the
//! exact `dx bundle --web --ssg` invocation **with `--package`** in a
//! multi-app workspace was never verified. This crate is therefore built to the
//! point where `cargo build` and the `wasm32` cross-compile both pass, with the
//! content real. The `IncrementalRendererConfig` + `static_routes`
//! server-function wiring that turns that into pre-rendered HTML is **not**
//! present — see the SSG SEAM below.

#![allow(non_snake_case)]

use dioxus::prelude::*;
use rv2_ui::{
    ArrowLink, Container, Divider, Eyebrow, Fact, FactList, Faq, FeatureCard, Footer, FooterColumn,
    Grid, Heading, HeadingSize, Hero, LinkButton, NavBar, NavItem, PricingCard, QandA, Section,
    Size, Space, Step, StepList, Text, Tone, Width,
};

const TAILWIND: Asset = asset!("/assets/tailwind.css");

#[derive(Clone, Debug, PartialEq, Routable)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/about")]
    About {},
}

fn main() {
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
            NavBar {
                brand: "rust-v2",
                items: vec![
                    NavItem::new("How it works", "/#how-it-works"),
                    NavItem::new("Pricing", "/#pricing"),
                    NavItem::new("About", "/about"),
                ],
                action: rsx! {
                    LinkButton { href: "/#pricing", "Get started" }
                },
            }
            main { {children} }
            Footer {
                columns: vec![
                    FooterColumn::new(
                        "Product",
                        vec![
                            NavItem::new("How it works", "/#how-it-works"),
                            NavItem::new("Pricing", "/#pricing"),
                            NavItem::new("FAQ", "/#faq"),
                        ],
                    ),
                    FooterColumn::new(
                        "Legal",
                        vec![
                            NavItem::new("Privacy", "/privacy"),
                            NavItem::new("Terms", "/terms"),
                        ],
                    ),
                    FooterColumn::new(
                        "Elsewhere",
                        vec![NavItem::external("AllSource", "https://github.com/all-source-os/all-source")],
                    ),
                ],
                note: rsx! {
                    p { "Built on AllSource. One datastore, one language, no TypeScript in the data path." }
                },
            }
        }
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        Shell {
            Section { space: Space::Loose,
                Container {
                    Hero {
                        eyebrow: "Event-sourced by default",
                        headline: "See every fact before it reaches your read model",
                        subheadline: "One datastore, one language. Every write is an event; every \
                                      read is a fold you can replay.",
                        actions: rsx! {
                            LinkButton { href: "/#pricing", size: Size::Large, "Get started" }
                            ArrowLink { href: "/#how-it-works", "See how it works" }
                        },
                    }
                }
            }

            Divider {}

            Section { id: "how-it-works",
                Container {
                    Eyebrow { "How it works" }
                    Heading { "From request to read model" }
                    div { class: "mt-8",
                        StepList {
                            steps: vec![
                                Step::new("Append", "A command validates, then appends one immutable event to AllSource."),
                                Step::new("Fold", "A projection worker folds the stream into a read model, checkpointed as it goes."),
                                Step::new("Read", "The API serves the folded state. Rebuild it any time by replaying from zero."),
                            ],
                        }
                    }
                }
            }

            Section {
                Container {
                    Heading { "What you get" }
                    div { class: "mt-8",
                        Grid { columns: 3,
                            FeatureCard {
                                step: 1,
                                title: "Complete history",
                                body: "Nothing is overwritten. Every state the system has ever been in is still queryable.",
                                action: rsx! {
                                    ArrowLink { href: "/#faq", "Read more" }
                                },
                            }
                            FeatureCard {
                                step: 2,
                                title: "Rebuildable reads",
                                body: "A read model is a pure fold. Change its shape and replay — no migration, no downtime.",
                                action: rsx! {
                                    ArrowLink { href: "/#faq", "Read more" }
                                },
                            }
                            FeatureCard {
                                step: 3,
                                title: "One dependency",
                                body: "WAL, Parquet and an in-memory index. No external database in the event path at all.",
                                action: rsx! {
                                    ArrowLink { href: "/#faq", "Read more" }
                                },
                            }
                        }
                    }
                }
            }

            Section { class: "bg-slate-50",
                Container { width: Width::Prose,
                    Heading { "What a write actually costs" }
                    Text { tone: Tone::Muted, class: "mt-2",
                        "Measured on the reference deployment, single node."
                    }
                    div { class: "mt-6 rounded-lg border border-slate-200 bg-white p-6",
                        FactList {
                            facts: vec![
                                Fact::new("Append throughput", "469,000 events/sec"),
                                Fact::new("Direct query latency", "11.9 µs"),
                                Fact::new("Durability", "WAL, CRC32 + fsync"),
                                Fact::total("External databases", "0"),
                            ],
                        }
                    }
                }
            }

            Section { id: "pricing",
                Container { width: Width::Prose,
                    Heading { "Pricing" }
                    div { class: "mt-6",
                        PricingCard {
                            title: "Community",
                            price: "£0",
                            cadence: "forever",
                            features: vec![
                                "Full event store".to_string(),
                                "Projections and replay".to_string(),
                                "Vector and keyword search".to_string(),
                                "Apache 2.0".to_string(),
                            ],
                            action: rsx! {
                                LinkButton {
                                    href: "https://github.com/all-source-os/all-source",
                                    external: true,
                                    size: Size::Large,
                                    class: "w-full",
                                    "Read the source",
                                }
                            },
                            note: "Replication and multi-tenancy are BSL 1.1, converting to Apache 2.0 in 2029.",
                        }
                    }
                }
            }

            Section { id: "faq",
                Container { width: Width::Prose,
                    Heading { "Direct answers" }
                    div { class: "mt-6",
                        Faq {
                            items: vec![
                                QandA::new(
                                    "Is everything event-sourced?",
                                    "No. Rate-limit counters, session caches, blob bytes and search \
                                     indexes are not. AllSource has no unique index and no \
                                     cross-entity transaction, so uniqueness is enforced elsewhere.",
                                ),
                                QandA::new(
                                    "What happens when a read model changes shape?",
                                    "Rename the worker's durable consumer id and it replays from \
                                     zero. Run old and new in parallel, cut reads over, delete the old state.",
                                ),
                                QandA::new(
                                    "Can an old event stop deserializing?",
                                    "Not if evolution stays additive. Every post-v1 field carries a \
                                     serde default, and a golden-JSON corpus test proves it.",
                                ),
                            ],
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn About() -> Element {
    rsx! {
        Shell {
            Section {
                Container { width: Width::Prose,
                    Heading { level: 1, size: HeadingSize::Display, "About" }
                    Text { class: "mt-4",
                        "rust-v2 is a Cargo workspace: \
                         an Axum API, two Dioxus frontends, and AllSource underneath."
                    }
                }
            }
        }
    }
}
