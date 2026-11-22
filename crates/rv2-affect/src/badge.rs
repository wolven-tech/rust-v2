//! Badge: a small status label, the one place a status gets its colour. A soft
//! tint with matching text, a leading dot (pulsing while `live`), a leading icon,
//! or a round count such as `3` or `99+`.
//!
//! Port losses: `count` is an `i64` and the count is grouped with commas
//! everywhere; the React original uses the reader's locale digits. Hugeicons
//! icon objects are not accepted: pass the inline svg as `icon`.

use dioxus::prelude::*;

use crate::internal::class_list;

const CSS: Asset = asset!("/assets/badge.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeTone {
    /// Grey.
    #[default]
    Neutral,
    /// Blue: running.
    Info,
    /// Green: done.
    Success,
    /// Orange: needs you.
    Warning,
    /// Red: failed.
    Danger,
    /// Purple: a kind, such as Beta.
    Purple,
}

impl BadgeTone {
    fn as_str(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Info => "info",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Danger => "danger",
            Self::Purple => "purple",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeSize {
    /// 18px high, 11px text.
    #[default]
    Sm,
    /// 20px high, 12px text.
    Md,
}

impl BadgeSize {
    fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
        }
    }
}

/// A status pill. With `count` it is a round number and `children` is ignored.
#[component]
pub fn Badge(
    #[props(default)] tone: BadgeTone,
    #[props(default)] size: BadgeSize,
    #[props(default)] dot: bool,
    #[props(default)] live: bool,
    #[props(default)] icon: Option<Element>,
    #[props(default)] count: Option<i64>,
    #[props(default = 99)] max: i64,
    #[props(default)] children: Option<Element>,
    #[props(default)] class: String,
    #[props(extends = GlobalAttributes, extends = span)] attributes: Vec<Attribute>,
) -> Element {
    let is_count = count.is_some();
    let lead = if is_count {
        None
    } else if live || dot {
        Some("dot")
    } else if icon.is_some() {
        Some("icon")
    } else {
        None
    };
    if !is_count && lead.is_none() && children.is_none() {
        return rsx! {};
    }
    let count_text = count.map(|value| format_count(value, max));
    let class = class_list("ui-bdgc", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        span {
            class,
            "data-tone": tone.as_str(),
            "data-size": size.as_str(),
            "data-count": is_count.then_some(""),
            "data-lead": lead,
            ..attributes,
            if lead == Some("dot") {
                span { class: "ui-bdgc__dot", "data-live": live.then_some(""), aria_hidden: "true" }
            }
            if lead == Some("icon") {
                span { class: "ui-bdgc__icon", aria_hidden: "true", {icon} }
            }
            if let Some(text) = count_text {
                "{text}"
            }
            if !is_count {
                span { class: "ui-bdgc__label", {children} }
            }
        }
    }
}

/// `7`, `1,234`, and `99+` once above `max`. Negative values read as zero.
pub fn format_count(count: i64, max: i64) -> String {
    let value = count.max(0);
    let top = max.max(0);
    if value > top {
        format!("{}+", group_thousands(top))
    } else {
        group_thousands(value)
    }
}

/// Groups a non-negative number's digits in threes: `1234567` → `1,234,567`.
pub fn group_thousands(value: i64) -> String {
    let digits = value.max(0).to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (idx, ch) in digits.chars().enumerate() {
        if idx > 0 && (digits.len() - idx).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}
