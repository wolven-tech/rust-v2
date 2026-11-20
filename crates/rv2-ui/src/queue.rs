//! A priority-ordered list of things that need someone: what, why, and either
//! where to go or what state it is in.
//!
//! Drawn from jbt-fit's coach review queue (`jbt-ui` `states.rs`), with its
//! fitness heading and empty copy moved out to the caller. The control room's
//! "Needs eyes" list is the same shape with a state instead of a link.

use dioxus::prelude::*;

use crate::feedback::{EmptyState, Verdict};

/// The end of a queue row.
#[derive(Debug, Clone, PartialEq)]
pub enum QueueTrail {
    /// The row is a link to where the work happens.
    Go { label: String, href: String },
    /// The row reports a state; nothing to follow.
    State { label: String, verdict: Verdict },
}

/// One row of an [`AttentionQueue`].
#[derive(Debug, Clone, PartialEq)]
pub struct QueueItem {
    pub id: String,
    pub title: String,
    pub reason: String,
    pub trail: QueueTrail,
}

impl QueueItem {
    pub fn go(
        id: impl Into<String>,
        title: impl Into<String>,
        reason: impl Into<String>,
        label: impl Into<String>,
        href: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            reason: reason.into(),
            trail: QueueTrail::Go {
                label: label.into(),
                href: href.into(),
            },
        }
    }

    pub fn state(
        id: impl Into<String>,
        title: impl Into<String>,
        reason: impl Into<String>,
        label: impl Into<String>,
        verdict: Verdict,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            reason: reason.into(),
            trail: QueueTrail::State {
                label: label.into(),
                verdict,
            },
        }
    }
}

/// How a row's reason reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReasonStyle {
    #[default]
    Prose,
    /// Measured evidence — "3 sessions missed" — in mono tabular digits.
    Evidence,
}

impl ReasonStyle {
    fn class(self) -> &'static str {
        match self {
            Self::Prose => "text-sm text-ink-muted",
            Self::Evidence => "font-mono text-xs tabular-nums text-ink-muted",
        }
    }
}

const ROW: &str = "grid min-h-11 grid-cols-[minmax(0,1fr)_auto] items-center gap-x-4 gap-y-0.5 \
                   rounded-lg border border-rule-soft bg-surface px-4 py-3";

/// The queue. The caller supplies the heading; the list is named by `label`.
///
/// Item ids key the rows and must be unique; a repeated id makes Dioxus reuse
/// the wrong row's DOM, so debug builds panic on one.
///
/// A `Go` row is one link with a real `href`. `on_open` lets an app route on
/// the client and still leaves the `href` for a new tab or no script.
#[component]
pub fn AttentionQueue(
    label: String,
    items: Vec<QueueItem>,
    /// Shown instead of the list when it is empty.
    empty: String,
    #[props(default)] reason: ReasonStyle,
    /// Prints 01, 02… beside each row for a queue read by position.
    #[props(default)]
    numbered: bool,
    #[props(default)] on_open: Option<EventHandler<(MouseEvent, String)>>,
) -> Element {
    if items.is_empty() {
        return rsx! { EmptyState { title: empty } };
    }
    debug_assert!(
        {
            let mut seen = std::collections::HashSet::new();
            items.iter().all(|item| seen.insert(item.id.as_str()))
        },
        "AttentionQueue item ids must be unique"
    );
    rsx! {
        ol { class: "space-y-2", aria_label: "{label}",
            for (i , item) in items.into_iter().enumerate() {
                li { key: "{item.id}", class: "flex items-center gap-3",
                    if numbered {
                        span { class: "w-6 shrink-0 font-mono text-xs tabular-nums text-ink-muted", aria_hidden: "true", "{i + 1:02}" }
                    }
                    {
                        let QueueItem { id, title, reason: why, trail } = item;
                        let body = rsx! {
                            span { class: "font-medium text-ink", "{title}" }
                            span { class: "col-start-1 {reason.class()}", "{why}" }
                        };
                        match trail {
                            QueueTrail::Go { label, href } => rsx! {
                                a {
                                    class: "{ROW} flex-1 hover:border-rule focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent",
                                    href,
                                    onclick: move |event: MouseEvent| {
                                        if let Some(open) = on_open {
                                            open.call((event, id.clone()));
                                        }
                                    },
                                    {body}
                                    span { class: "col-start-2 row-span-2 row-start-1 text-sm font-semibold text-ink",
                                        "{label}"
                                        span { aria_hidden: "true", " →" }
                                    }
                                }
                            },
                            QueueTrail::State { label, verdict } => rsx! {
                                div { class: "{ROW} flex-1",
                                    {body}
                                    span { class: "col-start-2 row-span-2 row-start-1 text-xs font-bold uppercase tracking-widest {verdict.ink()}",
                                        "{label}"
                                    }
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}
