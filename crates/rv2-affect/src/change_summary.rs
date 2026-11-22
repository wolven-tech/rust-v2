//! Change summary: what moved since last time, one metric per row, in one white
//! card with hairline shadows. The heading is the card's first row.
//!
//! Port losses: none.

use dioxus::prelude::*;

use crate::internal::class_list;

const CSS: Asset = asset!("/assets/change-summary.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MetricTone {
    /// Green arrow.
    Good,
    /// Red arrow.
    Bad,
    /// Grey dash.
    #[default]
    Neutral,
}

impl MetricTone {
    fn as_str(self) -> &'static str {
        match self {
            Self::Good => "good",
            Self::Bad => "bad",
            Self::Neutral => "neutral",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetricDirection {
    Up,
    Down,
}

#[derive(Clone, PartialEq)]
pub struct MetricChange {
    pub label: String,
    /// What happened, in words, e.g. "Up 6.1% to 18.2K".
    pub change: String,
    pub tone: MetricTone,
    /// Arrow direction. Defaults to up for good and down for bad.
    pub direction: Option<MetricDirection>,
}

/// Whether the arrow points down. Neutral rows draw a dash, so their direction is unused.
pub fn points_down(tone: MetricTone, direction: Option<MetricDirection>) -> bool {
    match direction {
        Some(direction) => direction == MetricDirection::Down,
        None => tone == MetricTone::Bad,
    }
}

/// Start of a row's entrance, in ms. The stagger stops after twelve rows, so a
/// long list is all there at once.
pub fn row_delay_ms(index: usize) -> usize {
    120 + index.min(12) * 70
}

#[component]
pub fn ChangeSummary(
    #[props(default = "What's changed".to_string(), into)] title: String,
    /// Small text on the right of the heading, e.g. "This week".
    #[props(default, into)]
    period: Option<String>,
    #[props(default)] icon: Option<Element>,
    changes: Vec<MetricChange>,
    #[props(default)] class: String,
) -> Element {
    let class = class_list("ui-csc", &class);

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        section { class, aria_label: "{title}",
            header { class: "ui-csc__head",
                span { class: "ui-csc__title",
                    span { class: "ui-csc__icon",
                        if let Some(icon) = icon {
                            {icon}
                        } else {
                            ChangesIcon {}
                        }
                    }
                    span { class: "ui-csc__title-text", dir: "auto", title: "{title}", "{title}" }
                }
                if let Some(period) = period {
                    span { class: "ui-csc__period", dir: "auto", title: "{period}", "{period}" }
                }
            }
            if !changes.is_empty() {
                dl { class: "ui-csc__list",
                    for (i, change) in changes.into_iter().enumerate() {
                        {
                            let delay = row_delay_ms(i);
                            let tone = change.tone;
                            let down = points_down(tone, change.direction);
                            let label = change.label.clone();
                            let text = change.change.clone();
                            let key = format!("{i}-{}", change.label);
                            rsx! {
                                div { key: "{key}", class: "ui-csc__row", style: "animation-delay: {delay}ms",
                                    dt { class: "ui-csc__label", dir: "auto", title: "{label}", "{label}" }
                                    dd { class: "ui-csc__change",
                                        span { class: "ui-csc__trend", "data-tone": tone.as_str(), "aria-hidden": "true",
                                            if tone == MetricTone::Neutral {
                                                DashIcon {}
                                            } else {
                                                ArrowIcon { down }
                                            }
                                        }
                                        span { class: "ui-csc__text", dir: "auto", title: "{text}", "{text}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ChangesIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 16 16",
            width: "16",
            height: "16",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.1",
            stroke_linecap: "round",
            "aria-hidden": "true",
            path { d: "M2.5 4h4M2.5 8h8M6.5 12h7M9.5 4h1" }
        }
    }
}

#[component]
fn ArrowIcon(down: bool) -> Element {
    rsx! {
        svg {
            view_box: "0 0 12 12",
            width: "12",
            height: "12",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.25",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            if down {
                path { d: "M6 2.5v7M3 6.5l3 3 3-3" }
            } else {
                path { d: "M6 9.5v-7M3 5.5l3-3 3 3" }
            }
        }
    }
}

#[component]
fn DashIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 12 12",
            width: "12",
            height: "12",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.25",
            stroke_linecap: "round",
            "aria-hidden": "true",
            path { d: "M3 6h6" }
        }
    }
}
