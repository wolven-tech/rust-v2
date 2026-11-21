//! Animated number like an odometer: each digit is a column that rolls to its new value,
//! up when the number grows and down when it shrinks. The first view counts in from zero.
//!
//! Port losses: `Intl.NumberFormat` is not available here, so formatting is
//! `decimals`, `grouping`, `prefix` and `suffix`. Columns that appear after mount (9,999 to
//! 10,000) do not slide in. The roll itself runs in `document::eval` on requestAnimationFrame.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id, prefers_motion_script};

const CSS: Asset = asset!("/assets/number-ticker.css");

/// One piece of the formatted text. Digits roll; everything else is static.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Part {
    Digit { key: String, digit: u8 },
    Static { key: String, ch: char },
}

/// Formats like `Intl.NumberFormat` with fixed decimals: `1234.5` → `1,234.5`.
/// A non-finite value shows a dash instead of the raw word.
pub fn format_value(value: f64, decimals: usize, grouping: bool) -> String {
    if !value.is_finite() {
        return "–".to_string();
    }
    let fixed = format!("{:.*}", decimals, value.abs());
    let (int_part, frac_part) = match fixed.split_once('.') {
        Some((int, frac)) => (int, Some(frac)),
        None => (fixed.as_str(), None),
    };
    let mut out = String::new();
    if value < 0.0 && fixed.chars().any(|c| c.is_ascii_digit() && c != '0') {
        out.push('-');
    }
    let len = int_part.len();
    for (i, c) in int_part.chars().enumerate() {
        if grouping && i > 0 && (len - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if let Some(frac) = frac_part {
        out.push('.');
        out.push_str(frac);
    }
    out
}

/// Splits text into parts keyed from the right, so the ones column keeps its identity
/// when the number grows (`$48,240` → `$`, `4`, `8`, `,`, …).
pub fn split(text: &str) -> Vec<Part> {
    let mut parts = Vec::new();
    let (mut digits, mut statics) = (0usize, 0usize);
    for ch in text.chars().rev() {
        match ch.to_digit(10) {
            Some(digit) => {
                parts.push(Part::Digit {
                    key: format!("d{digits}"),
                    digit: digit as u8,
                });
                digits += 1;
            }
            None => {
                parts.push(Part::Static {
                    key: format!("s{statics}"),
                    ch,
                });
                statics += 1;
            }
        }
    }
    parts.reverse();
    parts
}

/// Position a column rolls to. Columns hold 0 through 9 plus a trailing 0, so 9 to 0
/// continues upwards without a jump.
pub fn roll_target(pos: i64, digit: u8, up: bool) -> i64 {
    let delta = if up {
        (i64::from(digit) - pos).rem_euclid(10)
    } else {
        -(pos - i64::from(digit)).rem_euclid(10)
    };
    pos + delta
}

/// Vertical offset of a column for a fractional position: `translateY` percent.
pub fn column_offset_percent(pos: f64) -> f64 {
    -(pos.rem_euclid(10.0)) / 11.0 * 100.0
}

#[component]
pub fn NumberTicker(
    value: f64,
    #[props(default = 0)] decimals: usize,
    #[props(default = true)] grouping: bool,
    #[props(default, into)] prefix: String,
    #[props(default, into)] suffix: String,
    #[props(default = 900)] duration: u32,
    #[props(default = 0)] delay: u32,
    #[props(default)] class: String,
) -> Element {
    let id = use_hook(|| format!("ui-nt-{}", next_id()));
    let mut started = use_signal(|| false);
    let mut prev = use_signal(|| 0.0_f64);
    let mut up = use_signal(|| true);

    let text = format!(
        "{prefix}{}{suffix}",
        format_value(value, decimals, grouping)
    );
    let parts = split(&text);
    let digits: Vec<(String, u8)> = parts
        .iter()
        .filter_map(|part| match part {
            Part::Digit { key, digit } => Some((key.clone(), *digit)),
            Part::Static { .. } => None,
        })
        .collect();

    let observe_id = id.clone();
    use_effect(move || {
        let script = format!(
            "const el = document.getElementById({observe_id:?});
             const go = () => setTimeout(() => dioxus.send(true), {delay});
             if (!el || typeof IntersectionObserver === 'undefined') {{ go(); return; }}
             const io = new IntersectionObserver(([e]) => {{
               if (!e.isIntersecting) return;
               io.disconnect();
               go();
             }}, {{ threshold: 0.3 }});
             io.observe(el);"
        );
        let mut channel = document::eval(&script);
        spawn(async move {
            if channel.recv::<bool>().await.is_ok() {
                started.set(true);
            }
        });
    });

    // Direction follows the sign of the change; the same value keeps the last direction.
    let roll_id = id.clone();
    use_effect(use_reactive(
        (&digits, &started(), &value),
        move |(digits, started, value)| {
            if value != prev() {
                up.set(value > prev());
                prev.set(value);
            }
            if !started {
                return;
            }
            let list = digits
                .iter()
                .map(|(key, digit)| format!("[{key:?},{digit}]"))
                .collect::<Vec<_>>()
                .join(",");
            let script = format!(
            "const root = document.getElementById({roll_id:?});
             if (!root) return;
             const token = String(performance.now());
             root.dataset.roll = token;
             const reduce = !({motion});
             const up = {up};
             const rolls = [{list}].map(([key, digit]) => {{
               const el = root.querySelector('[data-col=\"' + key + '\"]');
               const from = Number(el?.dataset.pos ?? 0);
               const delta = up ? ((digit - from) % 10 + 10) % 10 : -(((from - digit) % 10 + 10) % 10);
               return {{ el, from, to: from + delta }};
             }}).filter((r) => r.el);
             const place = (el, pos) => {{ el.style.transform = `translateY(${{(-((((pos % 10) + 10) % 10)) / 11) * 100}}%)`; }};
             if (reduce) {{
               rolls.forEach((r) => {{ r.el.dataset.pos = r.to; place(r.el, r.to); }});
               return;
             }}
             const t0 = performance.now();
             const frame = (now) => {{
               if (root.dataset.roll !== token) return;
               const t = Math.min(1, (now - t0) / {duration});
               const e = 1 - Math.pow(1 - t, 4);
               for (const r of rolls) {{
                 const p = t < 1 ? r.from + (r.to - r.from) * e : r.to;
                 r.el.dataset.pos = p;
                 place(r.el, p);
               }}
               if (t < 1) requestAnimationFrame(frame);
             }};
             requestAnimationFrame(frame);",
            motion = prefers_motion_script(),
            up = if up() { "true" } else { "false" },
        );
            document::eval(&script);
        },
    ));

    let class = class_list("ui-nt", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        span { id: "{id}", class, "aria-live": "polite",
            span { class: "ui-nt__sr", "{text}" }
            span { class: "ui-nt__track", "aria-hidden": "true",
                for part in parts {
                    match part {
                        Part::Digit { key, .. } => rsx! {
                            span { key: "{key}", class: "ui-nt__col", "data-col": "{key}",
                                span { class: "ui-nt__stack", "data-pos": "0",
                                    for n in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0] {
                                        span { class: "ui-nt__cell", "{n}" }
                                    }
                                }
                            }
                        },
                        Part::Static { key, ch } => rsx! {
                            span { key: "{key}", class: "ui-nt__static", "{ch}" }
                        },
                    }
                }
            }
        }
    }
}
