//! The `rv2-ui` instruction set, readable and enforceable outside this
//! repository.
//!
//! Most products built on the kit are forks that carry their own copy of
//! `crates/rv2-ui`, and some depend on it by git revision. Neither sees this
//! repository's docs or runs its `cargo xtask ci`, so the agents working in
//! them guessed at the vocabulary and nothing caught a raw `button`. This crate
//! is the part that travels:
//!
//! - [`isa`] — UI7, the scan that fails an app writing a raw control, heading
//!   or link, and names the instruction to use instead.
//! - [`index`] — every instruction with its intent, props, an example and the
//!   tests that pin it, generated from source into
//!   `docs/design-system/kit-index.{md,json}`.
//!
//! `cargo xtask ci` runs both against this repository.

#![forbid(unsafe_code)]

pub mod index;
pub mod isa;
