//! A2UI surfaces in the rv2 catalog.
//!
//! **Layer 1, WASM-safe.** `apps/api` builds messages with these types and
//! `apps/app` parses and holds them, so both sides of the WASM boundary agree
//! on one definition. Decisions and the catalog table:
//! `docs/architecture/004-a2ui-surface.md` (`A2UI1`–`A2UI9`).
//!
//! - [`wire`]: the v0.9.1 envelopes, and JSONL framing.
//! - [`pointer`]: JSON Pointer reads and writes on the data model.
//! - [`value`]: a prop that is a literal or a data-model binding, and actions.
//! - [`catalog`]: the components an agent may place, their props, and the
//!   published JSON Schema generated from the same table.
//! - [`surface`]: applying messages atomically, and the tree checks that make
//!   an agent's output safe to render.
//!
//! Rendering lives in `rv2-a2ui-dioxus`, so this crate needs no renderer.

#![forbid(unsafe_code)]

pub mod catalog;
pub mod pointer;
pub mod surface;
pub mod value;
pub mod wire;

pub use crate::catalog::{CATALOG_ID, CatalogError, Node, catalog_document};
pub use crate::surface::{MAX_COMPONENTS, MAX_DEPTH, Surface, SurfaceError, Surfaces};
pub use crate::value::{Action, ActionRequest, Binding, Dyn};
pub use crate::wire::{
    ActionMessage, ClientBody, ClientMessage, ComponentDef, ErrorMessage, ServerBody,
    ServerMessage, VERSION, WireError, parse_jsonl, to_jsonl,
};
