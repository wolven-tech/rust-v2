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
    ArrowLink, Blob, Container, Crumple, Divider, Eyebrow, Fact, FactList, Faq, FeatureCard, Foil,
    Footer, FooterColumn, Fur, GradientBuilder, Grid, Heading, HeadingSize, Hero, Hologram,
    LinkButton, Mood, NavBar, NavItem, PricingCard, PullCord, QandA, Section, Size, Space, Step,
    StepList, Text, Tone, Vacuum, Width,
};

const TAILWIND: Asset = asset!("/assets/tailwind.css");

#[derive(Clone, Debug, PartialEq, Routable)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/about")]
    About {},
    #[route("/motion")]
    Motion {},
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
                    NavItem::new("Motion", "/motion"),
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

/// The `rv2_ui::motion` showcase, and the reason that module is allowed to
/// exist at all.
///
/// Same argument as `Home`: this is a **coverage fixture**, not decoration. Every
/// motion component is mounted here with real props, so removing one or changing
/// its signature stops this page compiling. For components whose whole output is
/// visual, a page that must keep compiling is the cheapest test there is — and
/// for the rest, look at it. Nothing in CI renders a page.
#[component]
fn Motion() -> Element {
    // Blob state: the form companion. Where the blob looks, and whether its eyes
    // are shut, follows the focused field.
    let mut mood = use_signal(|| Mood::Neutral);
    let mut gaze = use_signal(|| (0.0_f32, 0.0_f32));
    let mut fed_up = use_signal(|| false);

    // Dismissal demos.
    let mut crumpled = use_signal(|| false);
    let mut vacuumed = use_signal(|| false);

    // The cord toggles the foil card between two skins, so its effect is
    // visible on the page rather than being a console message.
    let mut lights_on = use_signal(|| true);

    rsx! {
        Shell {
            Section {
                Container { width: Width::Prose,
                    Eyebrow { "Component kit" }
                    Heading { level: 1, size: HeadingSize::Display, "Things that move" }
                    Text { class: "mt-4", tone: Tone::Muted,
                        "Reimplementations of ideas from FeralUI, in Dioxus. No React, no npm, \
                         and no render loop — every one of these is CSS animation driven by \
                         signals. Where that costs fidelity, the component's docs say so."
                    }
                }
            }

            Divider {}

            // ── Blob ─────────────────────────────────────────────────────────
            Section {
                Container {
                    Heading { level: 2, size: HeadingSize::Section, "Blob" }
                    Text { class: "mt-2", tone: Tone::Muted,
                        "Poke it. Five pokes and it has had enough. Focus a field and it \
                         reads along — then shuts its eyes for the password, which is the \
                         one mood here with a behavioural contract rather than a cosmetic one."
                    }
                    Grid { columns: 2, class: "mt-6 items-center gap-8",
                        div { class: "flex flex-col items-center gap-3",
                            Blob {
                                mood: mood(),
                                gaze: gaze(),
                                size: 160,
                                on_overpoke: move |()| fed_up.set(true),
                            }
                            if fed_up() {
                                Text { tone: Tone::Muted, "“That is enough, thank you.”" }
                            }
                        }
                        div { class: "space-y-3",
                            label { class: "block space-y-1",
                                span { class: "text-sm font-medium text-slate-700", "Email" }
                                input {
                                    r#type: "email",
                                    name: "demo_email",
                                    autocomplete: "email",
                                    class: "w-full rounded-md border border-slate-300 px-3 py-2 text-sm",
                                    onfocusin: move |_| {
                                        mood.set(Mood::Hmm);
                                        gaze.set((6.0, 3.0));
                                    },
                                }
                            }
                            label { class: "block space-y-1",
                                span { class: "text-sm font-medium text-slate-700", "Password" }
                                input {
                                    r#type: "password",
                                    name: "demo_password",
                                    autocomplete: "current-password",
                                    class: "w-full rounded-md border border-slate-300 px-3 py-2 text-sm",
                                    onfocusin: move |_| {
                                        mood.set(Mood::Password);
                                        gaze.set((0.0, 0.0));
                                    },
                                }
                            }
                            div { class: "flex flex-wrap gap-2 pt-2",
                                for (label , value) in [
                                    ("Neutral", Mood::Neutral),
                                    ("Happy", Mood::Happy),
                                    ("Sad", Mood::Sad),
                                    ("Angry", Mood::Angry),
                                    ("Hmm", Mood::Hmm),
                                    ("Side eye", Mood::SideEye),
                                ] {
                                    button {
                                        r#type: "button",
                                        class: "rounded-md border border-slate-300 px-3 py-1 text-xs \
                                                hover:bg-slate-50",
                                        onclick: move |_| mood.set(value),
                                        "{label}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            Divider {}

            // ── Hologram + PullCord ──────────────────────────────────────────
            Section {
                Container {
                    Heading { level: 2, size: HeadingSize::Section, "Hologram, and a pull-cord" }
                    Text { class: "mt-2", tone: Tone::Muted,
                        "Point at the card and it tilts, the sheen follows, the border glows \
                         where the light is. Leave it alone and the light keeps sweeping. \
                         Pull the cord to change the foil — it actuates mid-pull, at the \
                         detent, not on release."
                    }
                    Grid { columns: 2, class: "mt-6 items-start gap-8",
                        div { class: "max-w-[320px]",
                            // Plain elements with explicit light classes, not
                            // `Heading`/`Text`: those carry their own slate
                            // colours, which beat the card's inherited
                            // `text-slate-100` and render dark-on-dark. The card
                            // is a dark surface, so its contents have to say so.
                            Hologram {
                                foil: if lights_on() { Foil::Sunburst } else { Foil::Cosmos },
                                intensity: 0.45,
                                p {
                                    class: "text-xs font-semibold uppercase tracking-widest text-slate-300",
                                    "AllSource"
                                }
                                p { class: "mt-1 text-xl font-semibold text-white", "Event, holo rare" }
                                p { class: "mt-2 text-sm text-slate-200",
                                    "Append-only. Folds on read. Never overwrites."
                                }
                            }
                        }
                        div { class: "flex flex-col items-center",
                            PullCord {
                                label: "Change the foil",
                                on_pull: move |()| lights_on.toggle(),
                            }
                            Text { class: "mt-4", tone: Tone::Muted,
                                if lights_on() { "Sunburst foil." } else { "Cosmos foil." }
                            }
                        }
                    }
                }
            }

            Divider {}

            // ── Crumple, Vacuum, Fur ─────────────────────────────────────────
            Section {
                Container {
                    Heading { level: 2, size: HeadingSize::Section, "Getting rid of things" }
                    Text { class: "mt-2", tone: Tone::Muted,
                        "Two ways to dismiss something, and a surface with a coat on it."
                    }
                    Grid { columns: 3, class: "mt-6 items-start gap-6",
                        div { class: "space-y-3",
                            Crumple { crumpled: crumpled(),
                                div { class: "rounded-lg border border-slate-200 bg-white p-4 shadow-sm",
                                    Text { "Screw this one up and throw it away." }
                                }
                            }
                            button {
                                r#type: "button",
                                class: "rounded-md border border-slate-300 px-3 py-1 text-xs hover:bg-slate-50",
                                onclick: move |_| crumpled.toggle(),
                                if crumpled() { "Un-crumple" } else { "Crumple" }
                            }
                        }
                        div { class: "space-y-3",
                            Vacuum {
                                active: vacuumed(),
                                target: ("40px".to_string(), "180px".to_string()),
                                div { class: "rounded-lg border border-slate-200 bg-white p-4 shadow-sm",
                                    Text { "Sucked toward a point you choose." }
                                }
                            }
                            button {
                                r#type: "button",
                                class: "rounded-md border border-slate-300 px-3 py-1 text-xs hover:bg-slate-50",
                                onclick: move |_| vacuumed.toggle(),
                                if vacuumed() { "Put it back" } else { "Vacuum" }
                            }
                        }
                        // Light text on an OPAQUE chip, not directly on the coat.
                        //
                        // Same reason as the hologram's plate: text on a
                        // repeating-gradient has no computable contrast ratio, so
                        // a checker refuses to certify it — and here it was a
                        // real risk, because the fur's strand highlights swing
                        // several stops of lightness under the text as it
                        // ruffles. A caption whose legibility depends on which
                        // strand it lands on is not AA at any moment.
                        //
                        // `Text` is not used: its tone classes beat a passed
                        // `class`, which is what left this dark-on-brown before.
                        Fur { tint: "#a16207".to_string(),
                            p {
                                class: "inline-block rounded-md bg-stone-900 px-3 py-1.5 \
                                        text-sm font-medium text-amber-50",
                                "Hover to ruffle the coat."
                            }
                        }
                    }
                }
            }

            Divider {}

            // ── GradientBuilder ──────────────────────────────────────────────
            Section {
                Container { width: Width::Prose,
                    Heading { level: 2, size: HeadingSize::Section, "Gradient builder" }
                    Text { class: "mt-2 mb-6", tone: Tone::Muted,
                        "The one component reproduced in full, because it is pure state — \
                         there is no simulation to approximate, so nothing is lost."
                    }
                    GradientBuilder {}
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
