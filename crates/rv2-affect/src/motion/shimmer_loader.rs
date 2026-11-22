//! Loading placeholders with a soft light sweep. `Skeleton` draws a block; the highlight is
//! pinned to the viewport, so every skeleton on the page sweeps together. `ShimmerText` runs a
//! highlight band through a label, like an AI "Thinking…" state.
//!
//! Port losses: none.

use dioxus::prelude::*;

use crate::internal::class_list;

const CSS: Asset = asset!("/assets/shimmer-loader.css");

/// A CSS length: a number is pixels, a string is any CSS length.
#[derive(Clone, Debug, PartialEq)]
pub enum Length {
    Px(f64),
    Css(String),
}

impl From<f64> for Length {
    fn from(px: f64) -> Self {
        Self::Px(px)
    }
}

impl From<&str> for Length {
    fn from(css: &str) -> Self {
        Self::Css(css.to_string())
    }
}

impl From<String> for Length {
    fn from(css: String) -> Self {
        Self::Css(css)
    }
}

pub fn length_css(length: &Length) -> String {
    match length {
        Length::Px(px) => format!("{px}px"),
        Length::Css(css) => css.clone(),
    }
}

/// Inline style for a skeleton. A circle takes its width for both sides; a fixed pixel width
/// stops the block from shrinking inside a flex row.
pub fn skeleton_style(
    width: Option<&Length>,
    height: Option<&Length>,
    radius: Option<&Length>,
    circle: bool,
    extra: &str,
) -> String {
    let mut parts = Vec::new();
    if circle || matches!(width, Some(Length::Px(_))) {
        parts.push("flex-shrink: 0".to_string());
    }
    let width_css = width.map(length_css);
    if let Some(w) = &width_css {
        parts.push(format!("width: {w}"));
    }
    let height_css = if circle {
        width_css.or_else(|| height.map(length_css))
    } else {
        height.map(length_css)
    };
    if let Some(h) = height_css {
        parts.push(format!("height: {h}"));
    }
    let radius_css = if circle {
        Some("50%".to_string())
    } else {
        radius.map(length_css)
    };
    if let Some(r) = radius_css {
        parts.push(format!("border-radius: {r}"));
    }
    if !extra.is_empty() {
        parts.push(extra.trim_end_matches(';').to_string());
    }
    parts.join("; ")
}

#[component]
pub fn Skeleton(
    #[props(into)] width: Option<Length>,
    #[props(into)] height: Option<Length>,
    #[props(into)] radius: Option<Length>,
    #[props(default)] circle: bool,
    #[props(default)] class: String,
    #[props(default, into)] style: String,
) -> Element {
    let style = skeleton_style(
        width.as_ref(),
        height.as_ref(),
        radius.as_ref(),
        circle,
        &style,
    );
    let class = class_list("ui-shm", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        span { class, "aria-hidden": "true", style }
    }
}

#[component]
pub fn ShimmerText(
    children: Element,
    #[props(default = 2.0)] duration: f64,
    #[props(default)] class: String,
) -> Element {
    let class = class_list("ui-shm-text", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        span { class, style: "--ui-shm-text-dur: {duration}s", {children} }
    }
}
