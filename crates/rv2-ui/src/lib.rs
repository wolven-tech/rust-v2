//! The shared Dioxus component kit.
//!
//! **Layer 2, WASM-safe.** This is a **rewrite** of rust-v1's `packages/ui`
//! (shadcn/React), not a port — there is no mechanical path from JSX to RSX.
//!
//! The crate takes `dioxus` with `default-features = false, features = ["lib"]`
//! deliberately: a component library must not decide the renderer. `apps/app`
//! adds `web`, `apps/web` adds `fullstack`.
//!
//! Styling is Tailwind utility classes. `dx` has automatic Tailwind detection in
//! 0.7, so each app owns an `assets/tailwind.css` and `dx serve` runs the build.
//!
//! ## Scope
//!
//! Deliberately shadcn-shaped but **minimal**: the set was fixed by auditing a
//! real marketing page rather than by porting a component catalogue. Everything
//! here is used by `apps/web` or `apps/app`. There is no modal, tab, tooltip,
//! carousel or date picker, because no page needs one yet — an unused component
//! is bundle weight and maintenance cost with nothing on the other side of the
//! ledger.
//!
//! ## Dependencies
//!
//! `dioxus` only. In particular:
//!
//! - **No router.** Navigation components take plain `href` strings, so the kit
//!   works under any renderer and callers wrap with `Link` if they want
//!   client-side routing.
//! - **No domain types.** Components take primitives, so nothing in this crate
//!   changes when the API's shapes change. `PostCard` lived here once and forced
//!   a `rv2-api-types` dependency on every consumer of the kit; it now lives in
//!   `apps/app`, next to the only thing that renders it.
//! - **No JavaScript for disclosure.** [`site::Faq`] is built on native
//!   `<details>`, so it works in the SSG build before hydration.
//! - **No render loop.** [`motion`] is animated entirely in CSS, because a
//!   spring solver would mean `web-sys` and a per-frame callback in a crate
//!   whose selling point is having neither. Where that costs fidelity, the
//!   component says so.
//!
//! ## A deviation from "minimal", recorded rather than hidden
//!
//! The scope note above says every component here is used by an app. [`motion`]
//! stretches that: it is a set of *playful* components — a pokeable mascot, a
//! foil card, a pull-cord — reimplemented from [FeralUI](https://feralui.dev)'s
//! ideas, and they exist because a kit with no personality is a kit nobody
//! reaches for. They are exercised by `apps/web`'s `/motion` page, which is
//! their honest justification: a showcase, not a feature.
//!
//! If they stop earning their bundle weight, delete the module and the page
//! together.

#![forbid(unsafe_code)]
#![allow(non_snake_case)] // Dioxus components are PascalCase by convention.

pub mod feedback;
pub mod form;
pub mod layout;
pub mod motion;
pub mod primitives;
pub mod site;
pub mod typography;

pub use crate::feedback::{Card, EmptyState, ErrorBanner, PageHeader, Skeleton};
pub use crate::form::{TextArea, TextField};
pub use crate::layout::{Container, Divider, Grid, Row, Section, Space, Stack, Width};
pub use crate::motion::{
    Blob, Crumple, Foil, Fur, GradientBuilder, Hologram, Mood, PullCord, Vacuum,
};
pub use crate::primitives::{ArrowLink, Badge, Button, LinkButton, Size, StepMarker, Variant};
pub use crate::site::{
    Fact, FactList, Faq, FeatureCard, Footer, FooterColumn, Hero, NavBar, NavItem, PricingCard,
    QandA, Step, StepList,
};
pub use crate::typography::{Eyebrow, Heading, HeadingSize, Text, Tone};
