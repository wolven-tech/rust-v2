//! Rating: stars or thumbs in a radio group with one tab stop. Hover previews,
//! a press sets it, pressing the chosen one again clears it, and a choice pops.
//!
//! Port losses: `getValueLabel` is not ported, so labels read "3 out of 5". The
//! React `Rating` dispatches on `variant`; here `Rating` (stars) and
//! `ThumbsRating` are separate components. A controlled value cannot be null,
//! so "no rating" is `0.0` for stars and `ThumbRating::Unrated` for thumbs.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};

const CSS: Asset = asset!("/assets/rating.css");

/// Hugeicons Free (MIT), drawn solid. Filled as a second copy clipped on top.
const STAR: &str = "M13.7276 3.44418L15.4874 6.99288C15.7274 7.48687 16.3673 7.9607 16.9073 8.05143L20.0969 8.58575C22.1367 8.92853 22.6167 10.4206 21.1468 11.8925L18.6671 14.3927C18.2471 14.8161 18.0172 15.6327 18.1471 16.2175L18.8571 19.3125C19.417 21.7623 18.1271 22.71 15.9774 21.4296L12.9877 19.6452C12.4478 19.3226 11.5579 19.3226 11.0079 19.6452L8.01827 21.4296C5.8785 22.71 4.57865 21.7522 5.13859 19.3125L5.84851 16.2175C5.97849 15.6327 5.74852 14.8161 5.32856 14.3927L2.84884 11.8925C1.389 10.4206 1.85895 8.92853 3.89872 8.58575L7.08837 8.05143C7.61831 7.9607 8.25824 7.48687 8.49821 6.99288L10.258 3.44418C11.2179 1.51861 12.7777 1.51861 13.7276 3.44418Z";

const THUMB_UP: [&str; 2] = [
    "M2 12.5C2 11.3954 2.89543 10.5 4 10.5C5.65685 10.5 7 11.8431 7 13.5V17.5C7 19.1569 5.65685 20.5 4 20.5C2.89543 20.5 2 19.6046 2 18.5V12.5Z",
    "M15.4787 7.80626L15.2124 8.66634C14.9942 9.37111 14.8851 9.72349 14.969 10.0018C15.0369 10.2269 15.1859 10.421 15.389 10.5487C15.64 10.7065 16.0197 10.7065 16.7791 10.7065H17.1831C19.7532 10.7065 21.0382 10.7065 21.6452 11.4673C21.7145 11.5542 21.7762 11.6467 21.8296 11.7437C22.2965 12.5921 21.7657 13.7351 20.704 16.0211C19.7297 18.1189 19.2425 19.1678 18.338 19.7852C18.2505 19.8449 18.1605 19.9013 18.0683 19.9541C17.116 20.5 15.9362 20.5 13.5764 20.5H13.0646C10.2057 20.5 8.77628 20.5 7.88814 19.6395C7 18.7789 7 17.3939 7 14.6239V13.6503C7 12.1946 7 11.4668 7.25834 10.8006C7.51668 10.1344 8.01135 9.58664 9.00069 8.49112L13.0921 3.96056C13.1947 3.84694 13.246 3.79012 13.2913 3.75075C13.7135 3.38328 14.3652 3.42464 14.7344 3.84235C14.774 3.8871 14.8172 3.94991 14.9036 4.07554C15.0388 4.27205 15.1064 4.37031 15.1654 4.46765C15.6928 5.33913 15.8524 6.37436 15.6108 7.35715C15.5838 7.46692 15.5488 7.5801 15.4787 7.80626Z",
];
const THUMB_DOWN: [&str; 2] = [
    "M2 11.5C2 12.6046 2.89543 13.5 4 13.5C5.65685 13.5 7 12.1569 7 10.5V6.5C7 4.84315 5.65685 3.5 4 3.5C2.89543 3.5 2 4.39543 2 5.5V11.5Z",
    "M15.4787 16.1937L15.2124 15.3337C14.9942 14.6289 14.8851 14.2765 14.969 13.9982C15.0369 13.7731 15.1859 13.579 15.389 13.4513C15.64 13.2935 16.0197 13.2935 16.7791 13.2935H17.1831C19.7532 13.2935 21.0382 13.2935 21.6452 12.5327C21.7145 12.4458 21.7762 12.3533 21.8296 12.2563C22.2965 11.4079 21.7657 10.2649 20.704 7.9789C19.7297 5.88111 19.2425 4.83222 18.338 4.21485C18.2505 4.15508 18.1605 4.0987 18.0683 4.04586C17.116 3.5 15.9362 3.5 13.5764 3.5H13.0646C10.2057 3.5 8.77628 3.5 7.88814 4.36053C7 5.22106 7 6.60607 7 9.37607V10.3497C7 11.8054 7 12.5332 7.25834 13.1994C7.51668 13.8656 8.01135 14.4134 9.00069 15.5089L13.0921 20.0394C13.1947 20.1531 13.246 20.2099 13.2913 20.2493C13.7135 20.6167 14.3652 20.5754 14.7344 20.1577C14.774 20.1129 14.8172 20.0501 14.9036 19.9245C15.0388 19.728 15.1064 19.6297 15.1654 19.5323C15.6928 18.6609 15.8524 17.6256 15.6108 16.6429C15.5838 16.5331 15.5488 16.4199 15.4787 16.1937Z",
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RatingSize {
    /// Stars 16px, thumbs 28px.
    Sm,
    /// Stars 20px, thumbs 32px.
    #[default]
    Md,
    /// Stars 26px, thumbs 36px.
    Lg,
}

impl RatingSize {
    fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }
}

/// The thumbs state. `Unrated` is the React `null`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThumbRating {
    #[default]
    Unrated,
    Up,
    Down,
}

/// Stars the widget shows: 0 means 5, and the count is capped at 50.
pub fn star_count(max: u32) -> u32 {
    match max {
        0 => 5,
        n => n.min(50),
    }
}

/// `"4.5 out of 5"`: one decimal at most, no trailing `.0`.
pub fn value_label(value: f64, max: u32) -> String {
    let rounded = (value * 10.0).round() / 10.0;
    let number = if rounded.fract() == 0.0 {
        format!("{rounded:.0}")
    } else {
        format!("{rounded:.1}")
    };
    format!("{number} out of {max}")
}

/// How much of star `index` (1-based) is filled when `shown` stars are lit.
pub fn star_fill(shown: f64, index: u32) -> f64 {
    (shown - f64::from(index - 1)).clamp(0.0, 1.0)
}

/// Star ratings are shown to the nearest half.
pub fn half_step(value: f64) -> f64 {
    (value * 2.0).round() / 2.0
}

/// A radio group of stars. `value` 0 is no rating; `value` is `Some` to control it.
#[component]
pub fn Rating(
    #[props(default)] size: RatingSize,
    #[props(default)] read_only: bool,
    #[props(default)] disabled: bool,
    #[props(default = true)] allow_clear: bool,
    #[props(default = 5)] max: u32,
    #[props(default)] value: Option<f64>,
    #[props(default)] default_value: f64,
    #[props(default, into)] name: String,
    #[props(default)] class: String,
    #[props(default)] on_value_change: Option<EventHandler<f64>>,
    #[props(extends = GlobalAttributes, extends = div)] attributes: Vec<Attribute>,
) -> Element {
    let id = use_hook(|| format!("ui-rtg-{}", next_id()));
    let mut own = use_signal(|| default_value);
    let mut hover = use_signal(|| 0u32);
    let mut burst = use_signal(|| (0u32, 0u32));

    let max = star_count(max);
    let size_str = size.as_str();
    let class = class_list("ui-rtg", &class);
    let current = value.unwrap_or(own());

    if read_only {
        let v = if current.is_finite() {
            current.clamp(0.0, f64::from(max))
        } else {
            0.0
        };
        let shown = half_step(v);
        let label = if v > 0.0 {
            value_label(v, max)
        } else {
            "No rating".to_string()
        };
        return rsx! {
            document::Link { rel: "stylesheet", href: CSS }
            div {
                role: "img",
                aria_label: "{label}",
                class,
                "data-size": size_str,
                "data-readonly": "true",
                ..attributes,
                for i in 1..=max {
                    span { key: "{i}", class: "ui-rtg__star",
                        StarGlyph { fill: star_fill(shown, i), pop: false, delay_ms: 0, preview: false }
                    }
                }
            }
        };
    }

    let chosen = if current > 0.0 {
        Some(current.round().clamp(1.0, f64::from(max)) as u32)
    } else {
        None
    };
    let shown = if hover() > 0 {
        hover()
    } else {
        chosen.unwrap_or(0)
    };

    let mut choose = move |next: f64| {
        if value.is_none() {
            own.set(next);
        }
        if next > 0.0 {
            burst.set((next as u32, burst().1 + 1));
        }
        if let Some(handler) = &on_value_change {
            handler.call(next);
        }
    };

    let group_id = id.clone();
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            id: "{id}",
            role: "radiogroup",
            aria_label: "Rating",
            aria_disabled: disabled.then_some("true"),
            class,
            "data-size": size_str,
            "data-disabled": disabled.then_some("true"),
            onpointerleave: move |_| hover.set(0),
            ..attributes,
            for i in 1..=max {
                {
                    let pop = burst().1 > 0 && i <= burst().0 && i <= chosen.unwrap_or(0);
                    let pop_key = if pop { format!("pop-{}", burst().1) } else { "rest".to_string() };
                    let focus_group = group_id.clone();
                    rsx! {
                        button {
                            key: "{i}",
                            r#type: "button",
                            role: "radio",
                            class: "ui-rtg__star",
                            aria_checked: if chosen == Some(i) { "true" } else { "false" },
                            aria_label: "{value_label(f64::from(i), max)}",
                            tabindex: if i == chosen.unwrap_or(1) { "0" } else { "-1" },
                            disabled,
                            "data-hover": (hover() == i).then_some("true"),
                            onpointerenter: move |event| {
                                if event.data().pointer_type() == "mouse" {
                                    hover.set(i);
                                }
                            },
                            onclick: move |_| {
                                choose(if chosen == Some(i) && allow_clear { 0.0 } else { f64::from(i) });
                            },
                            onkeydown: move |event: KeyboardEvent| {
                                let key = event.key();
                                if matches!(key, Key::Delete | Key::Backspace) && allow_clear {
                                    event.prevent_default();
                                    choose(0.0);
                                    return;
                                }
                                if !matches!(key, Key::Home | Key::End | Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft | Key::ArrowRight) {
                                    return;
                                }
                                event.prevent_default();
                                let now = i64::from(chosen.unwrap_or(0));
                                let max_i = i64::from(max);
                                let group = focus_group.clone();
                                spawn(async move {
                                    let rtl = document::eval(&format!(
                                        "return getComputedStyle(document.getElementById({group:?})).direction;"
                                    ))
                                    .await
                                    .ok()
                                    .and_then(|v| v.as_str().map(|s| s == "rtl"))
                                    .unwrap_or(false);
                                    let to = match key {
                                        Key::Home => 1,
                                        Key::End => max_i,
                                        Key::ArrowUp => now + 1,
                                        Key::ArrowDown => now - 1,
                                        Key::ArrowRight => if rtl { now - 1 } else { now + 1 },
                                        _ => if rtl { now + 1 } else { now - 1 },
                                    };
                                    let next = to.clamp(1, max_i) as u32;
                                    hover.set(0);
                                    choose(f64::from(next));
                                    document::eval(&format!(
                                        "document.getElementById({group:?})?.children[{}]?.focus();",
                                        next - 1
                                    ));
                                });
                            },
                            StarGlyph {
                                key: "{pop_key}",
                                fill: if i <= shown { 1.0 } else { 0.0 },
                                pop,
                                delay_ms: (i - 1) * 40,
                                preview: hover() > 0,
                            }
                        }
                    }
                }
            }
            if !name.is_empty() {
                input { r#type: "hidden", name: "{name}", value: chosen.map(|c| c.to_string()).unwrap_or_default() }
            }
        }
    }
}

#[component]
fn StarGlyph(fill: f64, pop: bool, delay_ms: u32, preview: bool) -> Element {
    rsx! {
        span {
            class: "ui-rtg__glyph",
            "data-pop": pop.then_some("true"),
            "data-preview": preview.then_some("true"),
            style: "--ui-rtg-f: {fill}; animation-delay: {delay_ms}ms;",
            "aria-hidden": "true",
            svg { view_box: "0 0 24 24", class: "ui-rtg__empty", "focusable": "false",
                path { d: STAR, fill: "currentColor", stroke: "currentColor", stroke_width: "1.5", stroke_linejoin: "round" }
            }
            if fill > 0.0 {
                svg { view_box: "0 0 24 24", class: "ui-rtg__fill", "focusable": "false",
                    path { d: STAR, fill: "currentColor", stroke: "currentColor", stroke_width: "1.5", stroke_linejoin: "round" }
                }
            }
        }
    }
}

/// Thumbs up and down. Arrow keys, Home and End move to the other choice.
#[component]
pub fn ThumbsRating(
    #[props(default)] size: RatingSize,
    #[props(default)] read_only: bool,
    #[props(default)] disabled: bool,
    #[props(default = true)] allow_clear: bool,
    #[props(default)] value: Option<ThumbRating>,
    #[props(default)] default_value: ThumbRating,
    #[props(default, into)] name: String,
    #[props(default = "Good response".to_string(), into)] up_label: String,
    #[props(default = "Bad response".to_string(), into)] down_label: String,
    #[props(default)] class: String,
    #[props(default)] on_value_change: Option<EventHandler<ThumbRating>>,
    #[props(extends = GlobalAttributes, extends = div)] attributes: Vec<Attribute>,
) -> Element {
    let id = use_hook(|| format!("ui-rtg-{}", next_id()));
    let mut own = use_signal(|| default_value);
    let mut burst = use_signal(|| 0u32);

    let size_str = size.as_str();
    let class = class_list("ui-rtg ui-rtg--thumbs", &class);
    let chosen = value.unwrap_or(own());

    if read_only {
        let label = match chosen {
            ThumbRating::Up => format!("Rated: {up_label}"),
            ThumbRating::Down => format!("Rated: {down_label}"),
            ThumbRating::Unrated => "Not rated".to_string(),
        };
        return rsx! {
            document::Link { rel: "stylesheet", href: CSS }
            div {
                role: "img",
                aria_label: "{label}",
                class,
                "data-size": size_str,
                "data-readonly": "true",
                ..attributes,
                for kind in [ThumbRating::Up, ThumbRating::Down] {
                    span { key: "{kind:?}", class: "ui-rtg__thumb", "data-on": (chosen == kind).then_some("true"),
                        ThumbGlyph { down: kind == ThumbRating::Down, solid: chosen == kind, pop: false }
                    }
                }
            }
        };
    }

    let mut choose = move |next: ThumbRating| {
        if value.is_none() {
            own.set(next);
        }
        if next != ThumbRating::Unrated {
            burst.set(burst() + 1);
        }
        if let Some(handler) = &on_value_change {
            handler.call(next);
        }
    };

    let tab_stop = if chosen == ThumbRating::Down {
        ThumbRating::Down
    } else {
        ThumbRating::Up
    };
    let group_id = id.clone();
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            id: "{id}",
            role: "radiogroup",
            aria_label: "Rate this response",
            aria_disabled: disabled.then_some("true"),
            class,
            "data-size": size_str,
            "data-disabled": disabled.then_some("true"),
            ..attributes,
            for (index, kind) in [ThumbRating::Up, ThumbRating::Down].into_iter().enumerate() {
                {
                    let label = if kind == ThumbRating::Up { up_label.clone() } else { down_label.clone() };
                    let is_on = chosen == kind;
                    let other = if kind == ThumbRating::Up { ThumbRating::Down } else { ThumbRating::Up };
                    let focus_group = group_id.clone();
                    let pop_key = if is_on { format!("pop-{}", burst()) } else { "rest".to_string() };
                    rsx! {
                        button {
                            key: "{index}",
                            r#type: "button",
                            role: "radio",
                            class: "ui-rtg__thumb",
                            aria_checked: if is_on { "true" } else { "false" },
                            aria_label: "{label}",
                            title: "{label}",
                            tabindex: if kind == tab_stop { "0" } else { "-1" },
                            disabled,
                            "data-on": is_on.then_some("true"),
                            onclick: move |_| {
                                choose(if is_on && allow_clear { ThumbRating::Unrated } else { kind });
                            },
                            onkeydown: move |event: KeyboardEvent| {
                                let key = event.key();
                                if matches!(key, Key::Delete | Key::Backspace) && allow_clear {
                                    event.prevent_default();
                                    choose(ThumbRating::Unrated);
                                    return;
                                }
                                if !matches!(key, Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown | Key::Home | Key::End) {
                                    return;
                                }
                                event.prevent_default();
                                let next = match key {
                                    Key::Home => ThumbRating::Up,
                                    Key::End => ThumbRating::Down,
                                    _ => other,
                                };
                                choose(next);
                                let slot = if next == ThumbRating::Up { 0 } else { 1 };
                                document::eval(&format!(
                                    "document.getElementById({:?})?.children[{slot}]?.focus();",
                                    focus_group
                                ));
                            },
                            ThumbGlyph {
                                key: "{pop_key}",
                                down: kind == ThumbRating::Down,
                                solid: is_on,
                                pop: is_on && burst() > 0,
                            }
                        }
                    }
                }
            }
            if !name.is_empty() {
                input { r#type: "hidden", name: "{name}", value: match chosen { ThumbRating::Up => "up", ThumbRating::Down => "down", ThumbRating::Unrated => "" } }
            }
        }
    }
}

#[component]
fn ThumbGlyph(down: bool, solid: bool, pop: bool) -> Element {
    let paths = if down { THUMB_DOWN } else { THUMB_UP };
    rsx! {
        svg {
            view_box: "0 0 24 24",
            class: "ui-rtg__icon",
            "data-down": down.then_some("true"),
            "data-pop": pop.then_some("true"),
            "focusable": "false",
            "aria-hidden": "true",
            for (index, d) in paths.into_iter().enumerate() {
                path {
                    key: "{index}",
                    d,
                    fill: if solid { "currentColor" } else { "none" },
                    stroke: "currentColor",
                    stroke_width: "1.5",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                }
            }
        }
    }
}
