//! Endless "Trusted by" strip. Each row repeats its logos as often as the width needs and
//! slides by one measured set, so the wrap has no gap and no jump. Hovering or focusing the
//! strip eases it to a stop; dragging or swiping scrubs it with inertia. Reduced motion shows a
//! static, wrapping row.
//!
//! Port losses: the motion loop, pointer drag and keyboard nudging run in one
//! `document::eval` script that reads speed and direction from the DOM. Copy counts are measured
//! there and sent back, so the number of rendered sets follows the real width.

use dioxus::prelude::*;

use crate::internal::next_id;

const CSS: Asset = asset!("/assets/logo-marquee.css");

/// Motion engine. `__ID__` is the strip's DOM id. Reads `data-speed`, `data-pause` and
/// `data-dir` from the DOM each frame, and posts `[reduced, copies]` when either changes.
const SCRIPT: &str = r#"
const root = document.getElementById("__ID__");
if (!root) return;
const mq = window.matchMedia("(prefers-reduced-motion: reduce)");
let counts = [];
const send = () => dioxus.send([mq.matches, counts]);
const rows = () => Array.from(root.querySelectorAll(".ui-lm__row"));
const st = [];
const mod = (n, m) => (m > 0 ? ((n % m) + m) % m : 0);
const clamp = (n, a, b) => Math.min(b, Math.max(a, n));
let hover = false;
let focus = false;
let raf = 0;
let last = 0;
let visible = false;
let frames = 0;

const measure = () => {
  const next = rows().map((row, i) => {
    const set = row.querySelector(".ui-lm__set");
    const w = set ? set.getBoundingClientRect().width : 0;
    const s = (st[i] ??= { x: i * w * 0.5, v: 0, w, drag: null, seek: 0 });
    if (s.w > 0 && w > 0 && s.w !== w) s.x = (s.x / s.w) * w;
    s.w = w;
    return w > 0 ? Math.ceil(row.clientWidth / w) + 1 : 2;
  });
  if (next.length !== counts.length || next.some((c, i) => c !== counts[i])) {
    counts = next;
    send();
  }
};

const frame = (now) => {
  if (mq.matches) { raf = 0; return; }
  const dt = last ? Math.min(0.05, (now - last) / 1000) : 0;
  last = now;
  if (++frames % 30 === 0) measure();
  const speed = Number(root.dataset.speed) || 0;
  const stopped = root.dataset.pause === "true" && (hover || focus);
  rows().forEach((row, i) => {
    const s = (st[i] ??= { x: 0, v: 0, w: 0, drag: null, seek: 0 });
    const track = row.querySelector(".ui-lm__track");
    if (!track || s.w <= 0) return;
    const dir = Number(row.dataset.dir) || 1;
    if (!s.drag) {
      const target = stopped ? 0 : speed * dir;
      const fling = Math.abs(s.v - target) > Math.max(120, speed * 2);
      const tau = fling ? 0.55 : stopped ? 0.32 : 0.7;
      s.v += (target - s.v) * (1 - Math.exp(-dt / tau));
      s.x += s.v * dt;
      if (s.seek) {
        const step = s.seek * (1 - Math.exp(-dt / 0.12));
        s.x += step;
        s.seek = Math.abs(s.seek - step) < 0.5 ? 0 : s.seek - step;
      }
    }
    s.x = mod(s.x, s.w);
    track.style.transform = `translate3d(${-s.x}px, 0, 0)`;
  });
  raf = visible ? requestAnimationFrame(frame) : 0;
};

const rowOf = (target) => (target && target.closest ? target.closest(".ui-lm__row") : null);
const stateOf = (row) => st[rows().indexOf(row)];

new IntersectionObserver(([e]) => {
  visible = e.isIntersecting;
  if (visible && !raf) { last = 0; raf = requestAnimationFrame(frame); }
}).observe(root);

root.addEventListener("pointerenter", (e) => { if (e.pointerType !== "touch") hover = true; });
root.addEventListener("pointerleave", () => { hover = false; });
root.addEventListener("pointerdown", (e) => {
  const row = rowOf(e.target);
  const s = row && stateOf(row);
  if (!s || (e.pointerType === "mouse" && e.button !== 0)) return;
  s.drag = { id: e.pointerId, lastX: e.clientX, lastT: e.timeStamp, moved: 0, captured: false };
  s.v = 0;
});
root.addEventListener("pointermove", (e) => {
  const row = rowOf(e.target);
  const s = row && stateOf(row);
  const d = s && s.drag;
  if (!s || !d || d.id !== e.pointerId) return;
  const dx = e.clientX - d.lastX;
  const dt = Math.max(1, e.timeStamp - d.lastT) / 1000;
  d.moved += Math.abs(dx);
  if (!d.captured && d.moved > 4) {
    d.captured = true;
    row.setPointerCapture(e.pointerId);
    row.setAttribute("data-dragging", "");
  }
  s.x -= dx;
  s.v = s.v * 0.6 + (-dx / dt) * 0.4;
  d.lastX = e.clientX;
  d.lastT = e.timeStamp;
});
const release = (e) => {
  const row = rowOf(e.target);
  const s = row && stateOf(row);
  const d = s && s.drag;
  if (!s || !d || d.id !== e.pointerId) return;
  if (e.timeStamp - d.lastT > 80) s.v = 0;
  s.v = clamp(s.v, -2400, 2400);
  s.drag = null;
  row.removeAttribute("data-dragging");
  if (d.captured) {
    const stop = (ev) => { ev.preventDefault(); ev.stopPropagation(); };
    row.addEventListener("click", stop, { capture: true, once: true });
    setTimeout(() => row.removeEventListener("click", stop, { capture: true }), 0);
  }
};
root.addEventListener("pointerup", release);
root.addEventListener("pointercancel", release);

root.addEventListener("keydown", (e) => {
  if (e.key !== "ArrowLeft" && e.key !== "ArrowRight") return;
  const back = e.key === "ArrowLeft";
  if (e.target instanceof HTMLAnchorElement) {
    const list = Array.from(e.target.closest("ul").querySelectorAll("a[href]"));
    const next = list[list.indexOf(e.target) + (back ? -1 : 1)];
    if (next) { e.preventDefault(); next.focus(); }
    return;
  }
  if (e.target !== root || mq.matches) return;
  e.preventDefault();
  for (const s of st) if (s) s.seek += back ? -160 : 160;
});
root.addEventListener("focusin", (e) => {
  focus = e.target.matches(":focus-visible");
  if (!(e.target instanceof HTMLAnchorElement) || mq.matches) return;
  const row = rowOf(e.target);
  const s = row && stateOf(row);
  if (!s) return;
  const v = row.getBoundingClientRect();
  const r = e.target.getBoundingClientRect();
  const pad = Math.min(96, v.width * 0.15);
  if (r.left < v.left + pad) s.seek = r.left - (v.left + pad);
  else if (r.right > v.right - pad) s.seek = r.right - (v.right - pad);
});
root.addEventListener("focusout", (e) => {
  if (!root.contains(e.relatedTarget)) focus = false;
});

mq.addEventListener("change", send);
new ResizeObserver(() => measure()).observe(root);
measure();
send();
"#;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MarqueeDirection {
    /// Content moves left.
    #[default]
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MarqueeLabels {
    #[default]
    None,
    Below,
    Beside,
}

impl MarqueeLabels {
    fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Below => "below",
            Self::Beside => "beside",
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct MarqueeLogo {
    /// Brand name: the accessible name, and the visible label when `labels` is on.
    pub name: String,
    /// The mark, ideally an `svg` with `fill="currentColor"` so it follows the muted colour.
    pub svg: Option<Element>,
    /// An image instead of `svg`; grayscale until hovered when `grayscale` is on.
    pub src: Option<String>,
    /// Brand colour the mark lifts to on hover. Leave out for black marks.
    pub color: Option<String>,
    /// Optical size correction; 1 by default.
    pub scale: Option<f64>,
    pub href: Option<String>,
}

impl MarqueeLogo {
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            svg: None,
            src: None,
            color: None,
            scale: None,
            href: None,
        }
    }
}

/// First letter of a name, uppercased, for the letter tile. `"?"` for an empty name.
pub fn initial(name: &str) -> String {
    name.trim()
        .chars()
        .next()
        .map(|c| c.to_uppercase().collect())
        .unwrap_or_else(|| "?".to_string())
}

/// Which logos go in which row, with each row's run direction (`1` left, `-1` right).
/// Two rows split the logos alternately, so each row shows different brands.
pub fn row_plan(len: usize, rows: u8, direction: MarqueeDirection) -> Vec<(Vec<usize>, i8)> {
    let first: i8 = if direction == MarqueeDirection::Left {
        1
    } else {
        -1
    };
    if rows == 2 && len > 1 {
        vec![
            ((0..len).filter(|i| i % 2 == 0).collect(), first),
            ((0..len).filter(|i| i % 2 == 1).collect(), -first),
        ]
    } else {
        vec![((0..len).collect(), first)]
    }
}

/// The mark: the svg, else the image, else (when no visible name sits next to it) a letter tile.
#[component]
fn Mark(logo: MarqueeLogo, named: bool) -> Element {
    let mut failed = use_signal(|| false);
    if let Some(svg) = logo.svg.clone() {
        return rsx! { span { class: "ui-lm__mark", "aria-hidden": "true", {svg} } };
    }
    match logo.src.clone() {
        Some(src) if !failed() => rsx! {
            span { class: "ui-lm__mark", "aria-hidden": "true",
                img { src, alt: "", draggable: "false", onerror: move |_| failed.set(true) }
            }
        },
        _ if named => rsx! {},
        _ => rsx! {
            span { class: "ui-lm__mark", "aria-hidden": "true",
                span { class: "ui-lm__tile", {initial(&logo.name)} }
            }
        },
    }
}

#[component]
fn Item(logo: MarqueeLogo, labels: MarqueeLabels, interactive: bool) -> Element {
    let named = labels != MarqueeLabels::None;
    let mut style = String::new();
    if let Some(color) = &logo.color {
        style.push_str(&format!("--ui-lm-brand: {color}; "));
    }
    if let Some(scale) = logo.scale {
        style.push_str(&format!("--ui-lm-scale: {scale};"));
    }
    let body = rsx! {
        Mark { logo: logo.clone(), named }
        if named {
            span { class: "ui-lm__name", title: "{logo.name}", "{logo.name}" }
        } else {
            span { class: "ui-lm__sr", "{logo.name}" }
        }
    };
    rsx! {
        li { class: "ui-lm__item", style,
            if let Some(href) = logo.href.clone() {
                a { class: "ui-lm__link", href, draggable: "false", tabindex: (!interactive).then_some("-1"), {body} }
            } else {
                span { class: "ui-lm__link", {body} }
            }
        }
    }
}

#[component]
pub fn LogoMarquee(
    logos: Vec<MarqueeLogo>,
    #[props(default = 40.0)] speed: f64,
    #[props(default)] direction: MarqueeDirection,
    #[props(default = 1)] rows: u8,
    #[props(default = 56)] gap: u32,
    #[props(default = true)] pause_on_hover: bool,
    #[props(default = true)] grayscale: bool,
    #[props(default)] labels: MarqueeLabels,
    #[props(default = "Logos of companies using the product".to_string(), into)] label: String,
    #[props(default)] class: String,
) -> Element {
    let id = use_hook(|| format!("ui-lm-{}", next_id()));
    let mut reduced = use_signal(|| false);
    let mut counts = use_signal(Vec::<usize>::new);

    let engine_id = id.clone();
    use_effect(move || {
        let mut channel = document::eval(&SCRIPT.replace("__ID__", &engine_id));
        spawn(async move {
            while let Ok((is_reduced, measured)) = channel.recv::<(bool, Vec<usize>)>().await {
                reduced.set(is_reduced);
                if !measured.is_empty() {
                    counts.set(measured);
                }
            }
        });
    });

    if logos.is_empty() {
        return rsx! {};
    }

    let has_links = logos.iter().any(|logo| logo.href.is_some());
    let plan = row_plan(logos.len(), rows, direction);
    let reduced_now = reduced();
    let style = format!("--ui-lm-gap: {gap}px");
    let class = if class.is_empty() {
        "ui-lm".to_string()
    } else {
        format!("ui-lm {class}")
    };
    let tab_stop = (!has_links && !reduced_now).then_some("0");
    let speed_attr = format!("{speed}");
    let pause_attr = if pause_on_hover { "true" } else { "false" };

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            id: "{id}",
            class,
            style,
            role: "region",
            "aria-label": "{label}",
            tabindex: tab_stop,
            "data-labels": labels.as_str(),
            "data-grayscale": grayscale.then_some(""),
            "data-static": reduced_now.then_some(""),
            "data-speed": "{speed_attr}",
            "data-pause": "{pause_attr}",
            if reduced_now {
                ul { class: "ui-lm__wrap",
                    for (k, logo) in logos.iter().enumerate() {
                        Item { key: "{k}", logo: logo.clone(), labels, interactive: true }
                    }
                }
            } else {
                for (row, (members, dir)) in plan.iter().enumerate() {
                    div { key: "{row}", class: "ui-lm__row", "data-dir": "{dir}",
                        div { class: "ui-lm__track",
                            for c in 0..counts().get(row).copied().unwrap_or(2).max(2) {
                                ul { key: "{c}", class: "ui-lm__set",
                                    "aria-hidden": (c > 0).then_some("true"),
                                    "inert": (c > 0).then_some(""),
                                    for k in members.iter().copied() {
                                        Item { key: "{k}", logo: logos[k].clone(), labels, interactive: c == 0 }
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
