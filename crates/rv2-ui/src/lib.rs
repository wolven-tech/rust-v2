//! The shared Dioxus component kit.
//!
//! **Layer 2, WASM-safe.** This is a **rewrite** of rust-v1's `packages/ui`
//! (shadcn/React), not a port — there is no mechanical path from JSX to RSX.
//! getformlab's `crates/jbt-ui` is the shape and scope being matched.
//!
//! The crate takes `dioxus` with `default-features = false, features = ["lib"]`
//! deliberately: a component library must not decide the renderer. `apps/app`
//! adds `web`, `apps/web` adds `fullstack`.
//!
//! Styling is Tailwind utility classes. `dx` has automatic Tailwind detection in
//! 0.7, so each app owns an `assets/tailwind.css` and `dx serve` runs the build.

#![forbid(unsafe_code)]
#![allow(non_snake_case)] // Dioxus components are PascalCase by convention.

pub mod components;

pub use crate::components::{
    Button, Card, EmptyState, ErrorBanner, PageHeader, PostCard, Skeleton, TextArea, TextField,
    Variant,
};
