//! A trend line beside a figure.
//!
//! Drawn from jbt-fit's metric cards (`jbt-ui` `metrics.rs`), whose point maths
//! is ported in [`spark_points`]. A sparkline is the shape of a series, never
//! its value: the figure it belongs to is text beside it, as with
//! [`crate::Bar`].

use dioxus::prelude::*;

const WIDTH: f64 = 240.0;
const HEIGHT: f64 = 48.0;

/// SVG polyline points for `values` across `width` × `height`.
///
/// Empty when any value is not finite or the box is empty, so a broken series
/// draws nothing rather than a line through invented data. One value, or a
/// flat series, is a line across the middle.
pub fn spark_points(values: &[f64], width: f64, height: f64) -> String {
    if values.is_empty()
        || values.iter().any(|v| !v.is_finite())
        || !width.is_finite()
        || !height.is_finite()
        || width <= 0.0
        || height <= 0.0
    {
        return String::new();
    }
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    // Normalised first so a span of two huge finite values cannot overflow,
    // and by the series' own magnitude so a series of tiny values is not
    // mistaken for a flat one.
    let scale = min.abs().max(max.abs());
    if values.len() == 1 || scale == 0.0 {
        return format!("0,{} {},{}", height / 2.0, width, height / 2.0);
    }
    let span = max / scale - min / scale;
    if span <= f64::EPSILON {
        return format!("0,{} {},{}", height / 2.0, width, height / 2.0);
    }
    let last = (values.len() - 1) as f64;
    values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            format!(
                "{:.1},{:.1}",
                i as f64 / last * width,
                height - ((v / scale - min / scale) / span) * height
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The trend of `values`, stretched to its container's width.
///
/// Decorative unless `label` is given, in which case it is an image with that
/// name — "Resting heart rate, falling over 6 weeks". A named series that
/// cannot be drawn shows its name as text, so the trend is not lost with the
/// line. The stroke colour is the `--rv2-spark-stroke` hook.
#[component]
pub fn Sparkline(values: Vec<f64>, #[props(default)] label: Option<String>) -> Element {
    let points = spark_points(&values, WIDTH, HEIGHT);
    if points.is_empty() {
        return match label {
            Some(label) => rsx! { span { class: "text-xs text-ink-muted", "{label}" } },
            None => rsx! {},
        };
    }
    let (role, hidden) = match label {
        Some(_) => (Some("img"), None),
        None => (None, Some("true")),
    };
    rsx! {
        svg {
            class: "rv2-sparkline block h-12 w-full",
            view_box: "0 0 {WIDTH} {HEIGHT}",
            preserve_aspect_ratio: "none",
            role,
            "aria-label": label,
            "aria-hidden": hidden,
            polyline {
                points,
                fill: "none",
                vector_effect: "non-scaling-stroke",
                style: "stroke:var(--rv2-spark-stroke,var(--color-ink-muted,#475569));stroke-width:2;stroke-linecap:round;stroke-linejoin:round",
            }
        }
    }
}
