//! Step rail: a horizontal row of steps, each a node on a line with its name
//! and a state word under it. Finished steps are filled and joined by a drawn
//! line, the current step breathes, a drifted step flashes once.
//!
//! An original component, not an affectUI port (AF9 in
//! `docs/architecture/005-rv2-affect.md`): the ritual step rail of the cos
//! desktop app, rebuilt on this crate's conventions.

use dioxus::prelude::*;

use crate::internal::class_list;

const CSS: Asset = asset!("/assets/step-rail.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RailState {
    Finished,
    Current,
    #[default]
    Queued,
    /// The step drifted: its input changed after it was settled.
    Errored,
    AwaitingApproval,
}

impl RailState {
    /// The `data-state` value the stylesheet keys on.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Finished => "finished",
            Self::Current => "current",
            Self::Queued => "queued",
            Self::Errored => "errored",
            Self::AwaitingApproval => "awaiting-approval",
        }
    }

    /// The word shown under the step's name, read by screen readers with it.
    pub fn word(self) -> &'static str {
        match self {
            Self::Finished => "finished",
            Self::Current => "in progress",
            Self::Queued => "queued",
            Self::Errored => "drifted",
            Self::AwaitingApproval => "awaiting your approval",
        }
    }

    /// Where the flow stands: in progress, waiting on someone, or drifted.
    pub fn is_live(self) -> bool {
        matches!(self, Self::Current | Self::AwaitingApproval | Self::Errored)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RailStep {
    pub key: String,
    pub label: String,
    pub state: RailState,
}

impl RailStep {
    pub fn new(key: impl Into<String>, label: impl Into<String>, state: RailState) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            state,
        }
    }
}

/// The index of the step that carries `aria-current="step"`: the first live
/// one. A list with several live steps still names one current step.
pub fn current_index(steps: &[RailStep]) -> Option<usize> {
    steps.iter().position(|s| s.state.is_live())
}

#[component]
pub fn StepRail(
    steps: Vec<RailStep>,
    /// Names the list for assistive tech.
    #[props(default, into)]
    aria_label: Option<String>,
    #[props(default)] class: String,
) -> Element {
    let current = current_index(&steps);
    let last = steps.len().saturating_sub(1);
    let label = aria_label.unwrap_or_else(|| "Steps".to_string());
    let class = class_list("ui-step-rail", &class);

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        ol { class, aria_label: "{label}",
            for (index, step) in steps.iter().enumerate() {
                {
                    let is_current = current == Some(index);
                    let drawn = step.state == RailState::Finished;
                    let line_class = if drawn {
                        "ui-step-rail__line ui-step-rail__line--drawn"
                    } else {
                        "ui-step-rail__line"
                    };
                    rsx! {
                        li {
                            key: "{step.key}",
                            class: "ui-step-rail__step",
                            "data-state": step.state.as_str(),
                            aria_current: is_current.then_some("step"),
                            style: "--ui-sr-i: {index}",
                            span { class: "ui-step-rail__node", aria_hidden: "true" }
                            if index < last {
                                svg {
                                    class: line_class,
                                    view_box: "0 0 1 1",
                                    preserve_aspect_ratio: "none",
                                    "aria-hidden": "true",
                                    "focusable": "false",
                                    path { d: "M0 0.5H1", path_length: "1" }
                                }
                            }
                            span { class: "ui-step-rail__name", dir: "auto", title: "{step.label}", "{step.label}" }
                            span { class: "ui-step-rail__state", "{step.state.word()}" }
                        }
                    }
                }
            }
        }
    }
}
