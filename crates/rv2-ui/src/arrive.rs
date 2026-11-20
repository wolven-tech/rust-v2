//! Figures that arrive as they scroll into view.
//!
//! [`Tally`] counts up to its value; [`Arrive::Grow`] on [`crate::Bar`] and
//! [`crate::SplitBar`] grows a fill from its baseline. Both are CSS scroll
//! timelines in `assets/arrive.css`, with no script and no timer.
//!
//! UI9 keeps motion outside the ISA, and this module stays inside it only
//! because nothing here carries meaning by moving: the number is text in the
//! DOM and the end state is the static state. A browser without scroll
//! timelines, a printout and a reader with reduced motion all get the figure
//! as it is.

use dioxus::prelude::*;

/// How a magnitude mark appears.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Arrive {
    /// At its length from the first paint.
    #[default]
    Static,
    /// Grows from its baseline while it enters the viewport.
    Grow,
}

impl Arrive {
    pub(crate) fn class(self) -> &'static str {
        match self {
            Self::Static => "",
            Self::Grow => "rv2-grow",
        }
    }
}

/// A whole number that counts up to itself as it scrolls into view.
///
/// It takes the size and weight of whatever text it sits in. The number is
/// text, so nothing reads the figure out of a counter. `assets/arrive.css`
/// hides that text visually and draws a CSS counter, hidden from assistive
/// technology, in its place; a page without the stylesheet shows the text.
///
/// A CSS counter holds a signed 32-bit integer, so the visible digits stop at
/// `i32::MAX`; the text copy is always exact.
#[component]
pub fn Tally(value: u32) -> Element {
    let counted = value.min(i32::MAX as u32);
    rsx! {
        span { class: "relative",
            span { class: "rv2-tally-text", "{value}" }
            span { class: "rv2-tally", style: "--to:{counted}", aria_hidden: "true" }
        }
    }
}
