//! Dioxus port of affectUI's free (MIT) components.
//!
//! **Layer 2, WASM-safe**, and **opt-in**: nothing in `rv2-ui` depends on this
//! crate, and an app takes it only when it wants these components. `rv2-ui`
//! stays the minimal, Tailwind-styled instruction set of
//! `docs/architecture/003-ui-semantic-isa.md`; this crate is a catalogue, and
//! `docs/architecture/005-rv2-affect.md` records why it lives apart.
//!
//! Source: the 74 MIT items of <https://www.affectui.com> (51 components, 4
//! motion, 2 blocks, 17 illustrations). Licence notice: `LICENSE-affectui`.
//!
//! ## How a component is built
//!
//! - **CSS is the original's**, moved from the TSX template string into
//!   `assets/<slug>.css` and linked once per component with
//!   `document::Link`. Class names (`ui-*`), data attributes and the `--aui-*`
//!   theme variables are unchanged, so affectUI's theming docs apply as
//!   written. Dark mode follows `.dark` or `[data-theme="dark"]` on any
//!   ancestor.
//! - **Props mirror the React props**, as enums for the string unions and
//!   `Element` for `ReactNode` slots.
//! - **No `web-sys`, no JS package.** Measurement and focus go through
//!   `onmounted`; the few behaviours Dioxus has no API for (Web Animations
//!   width morphs, clipboard, `matchMedia`) go through `document::eval`. Where
//!   a port loses fidelity, the component's doc comment says what.
//!
//! ## Using it
//!
//! Components are addressed by module, `rv2_affect::button::Button`, because
//! several items define a same-named helper (`Kbd`, `Spinner`). Render
//! [`Rv2Theme`] once to take colours from the app's rv2 registers instead of
//! affectUI's defaults.

#![forbid(unsafe_code)]
#![allow(non_snake_case)] // Dioxus components are PascalCase by convention.

mod internal;

// Primitives
pub mod avatar;
pub mod badge;
pub mod button;
pub mod card;
pub mod checkbox;
pub mod empty_state;
pub mod form_field;
pub mod icon_button;
pub mod kbd;
pub mod separator;
pub mod skeleton;
pub mod spinner;
pub mod switch;

// Inputs
pub mod color_picker;
pub mod date_picker;
pub mod number_input;
pub mod otp_input;
pub mod rating;
pub mod segmented_control;
pub mod tag_input;
pub mod text_area;
pub mod text_field;
pub mod time_picker;
pub mod toggle_group;

// Overlays
pub mod context_menu;
pub mod dialog;
pub mod dropdown_menu;
pub mod hover_card;
pub mod popover;
pub mod select;
pub mod tooltip;

// Navigation and layout
pub mod breadcrumb;
pub mod filter_menu;
pub mod pagination;
pub mod profile_menu;
pub mod resizable;
pub mod scroll_area;
pub mod tree_view;

// AI and content
pub mod change_summary;
pub mod code_block;
pub mod comment_thread;
pub mod decision_inbox;
pub mod email_draft;
pub mod message_actions;
pub mod notification_stack;
pub mod progress_tracker;
pub mod reaction_bar;
pub mod smart_recommendation;
pub mod thinking_loader;
pub mod tips_panel;
pub mod tool_call_card;

pub mod blocks;
pub mod illustrations;
pub mod motion;
pub mod theme;

pub use crate::theme::Rv2Theme;
