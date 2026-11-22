//! Agent and product components for rv2, built clean-room.
//!
//! **Layer 2, WASM-safe, opt-in.** Each item is implemented from a functional
//! spec in `docs/specs/rv2-kit/`, written without access to any third-party
//! source (decision AF7 in `docs/architecture/005-rv2-affect.md`). The look is
//! rv2's own: every colour is an rv2 register through `assets/kit.css`.
//!
//! Interactive behaviour reuses `rv2-affect` primitives (dialog, popover,
//! select, tooltip …) rather than restating them.

#![forbid(unsafe_code)]
#![allow(non_snake_case)] // Dioxus components are PascalCase by convention.
#![allow(unpredictable_function_pointer_comparisons)]
// Props PartialEq on fn pointers only decides re-render.
// Dioxus rsx handlers clone Copy callbacks and closures per handler by design.
#![allow(
    clippy::clone_on_copy,
    clippy::redundant_closure,
    clippy::collapsible_if,
    clippy::collapsible_else_if,
    clippy::collapsible_match,
    clippy::type_complexity
)]

mod internal;

pub mod agents;
pub mod blocks;
pub mod components;
pub mod illustrations;
pub mod motion;
pub mod screens;
