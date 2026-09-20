//! The UK tech hiring register: what a company and a role are, how a role is
//! classified, and what the filters on the page mean.
//!
//! **Layer 1, WASM-safe.** No I/O and no clock. Every "today" is an argument.
//! The job-board readers and the careers-link checker live on the server side
//! and never appear here, so this crate cross-compiles to
//! `wasm32-unknown-unknown` and the Dioxus page can run the same filters the
//! command line does.
//!
//! ## Why the classifiers are code rather than notes on a record
//!
//! Three questions decide whether a role is worth an application, and all three
//! are answered by reading a job description rather than by reading its title:
//!
//! - [`work_pattern`] — how much of the week is spent in an office.
//! - [`engagement`] — permanent employment, or a B2B engagement the reader can
//!   take through their own company.
//! - [`mandate`] — whether the role lets the holder build a function and teach,
//!   or hands them a squad inside someone else's.
//!
//! Each classifier keeps the words that decided it. A score with no quote is
//! not reviewable, and every one of these judgements is one a reader should be
//! able to overrule.

#![forbid(unsafe_code)]

pub mod breakdown;
pub mod csv;
pub mod dashboard;
pub mod engagement;
pub mod links;
pub mod litmus;
pub mod mandate;
pub mod model;
pub mod text;
pub mod view;
pub mod work_pattern;

/// The register, compiled in.
///
/// File-backed rather than fetched: the page has no API behind it, so shipping
/// the data inside the binary is one download instead of two and removes the
/// loading state entirely. The cost is that refreshing the register means
/// rebuilding the page, which is the right trade for a file that changes when
/// somebody runs `hiring-register check`, not when somebody opens a tab.
pub const REGISTER_JSON: &str = include_str!("../data/companies.json");

/// Parse the compiled-in register and fill any role that has no stored verdict.
///
/// # Panics
///
/// If the compiled-in register does not parse. That is a build-time fact about
/// a file in this crate, not a runtime condition: there is no state in which
/// the page could usefully carry on without it, and a silent empty register
/// would read as "nobody is hiring".
#[must_use]
pub fn embedded_register() -> Register {
    serde_json::from_str::<Register>(REGISTER_JSON)
        .expect("the compiled-in register must parse")
        .classified()
}

pub use engagement::Engagement;
pub use links::safe_url;
pub use mandate::{LevelBand, Mandate, MandateSignal};
pub use model::{Company, Openings, Register, Role};
pub use view::{Filters, Layout, Sort};
pub use work_pattern::WorkPattern;
