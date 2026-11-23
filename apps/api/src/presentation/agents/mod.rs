//! Agents that answer in A2UI (`docs/architecture/004-a2ui-surface.md`).
//!
//! Each is a pure function from an action to the messages that answer it: no
//! surface state is kept between requests (A2UI8), and nothing here touches
//! the store, so the handlers own every side effect.

pub mod runway;
