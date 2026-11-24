//! Wall of quotes, dealt into columns (3 on wide walls, 2 on tablets, 1 on phones, measured on
//! the wall itself). Each column scrolls endlessly, alternating up and down at slightly
//! different speeds. Hovering or keyboard-focusing a column eases it to a stop; dragging it
//! scrubs it with inertia. Reduced motion shows a static masonry.
//!
//! Port losses: the motion loop, drag, press-and-hold and keyboard nudging run in one
//! `document::eval` script. It measures the column count and copy counts and sends them back.
//! Full text on truncated quotes is set as a `title` from the same script on pointer-over.

use dioxus::prelude::*;

use crate::internal::next_id;

const CSS: Asset = asset!("/assets/testimonial-wall.css");

/// Motion engine. `__ID__` is the wall's DOM id. Reads `data-speed`, `data-pause`,
/// `data-max` and `data-items` from the DOM, and posts `[reduced, fit, copies]` on change.
const SCRIPT: &str = r#"
const root = document.getElementById("__ID__");
if (!root) return;
const mq = window.matchMedia("(prefers-reduced-motion: reduce)");
const PACE = [1, -0.84, 1.12];
const START = [0, 0.42, 0.18];
let fit = 0;
let counts = [];
const send = () => dioxus.send([mq.matches, fit, counts]);
const cols = () => Array.from(root.querySelectorAll(".ui-tw__col"));
const st = [];
const hoverC = [];
const holdC = [];
const focusC = [];
const mod = (n, m) => (m > 0 ? ((n % m) + m) % m : 0);
const clamp = (n, a, b) => Math.min(b, Math.max(a, n));
let raf = 0;
let last = 0;
let visible = false;
let frames = 0;

const measure = () => {
  const max = Number(root.dataset.max) || 3;
  const w = root.clientWidth;
  const nextFit = Math.min(max, w >= 740 ? 3 : w >= 480 ? 2 : 1);
  const next = cols().map((col, i) => {
    const set = col.querySelector("[data-real]");
    if (!set) return 2;
    const scale = col.getBoundingClientRect().height / (col.offsetHeight || 1) || 1;
    const h = set.getBoundingClientRect().height / scale;
    const s = (st[i] ??= { y: h * (START[i] ?? 0), v: 0, h, drag: null, seek: 0 });
    if (s.h > 0 && h > 0 && s.h !== h) s.y = (s.y / s.h) * h;
    s.h = h;
    return h > 0 ? Math.min(12, Math.ceil(col.clientHeight / h) + 1) : 2;
  });
  st.length = cols().length;
  if (nextFit !== fit || next.length !== counts.length || next.some((c, i) => c !== counts[i])) {
    fit = nextFit;
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
  const pauseOnHover = root.dataset.pause === "true";
  const list = cols();
  const count = list.length;
  for (let i = 0; i < count; i++) {
    const s = st[i];
    const track = list[i].querySelector(".ui-tw__track");
    if (!s || !track || s.h <= 0) continue;
    const pace = count === 1 ? 1 : PACE[i % 3];
    const stopped = holdC[i] || focusC[i] || (pauseOnHover && hoverC[i]);
    if (!s.drag) {
      const target = stopped ? 0 : speed * pace;
      const fling = Math.abs(s.v - target) > Math.max(120, speed * 2);
      const tau = fling ? 0.55 : stopped ? 0.32 : 0.7;
      s.v += (target - s.v) * (1 - Math.exp(-dt / tau));
      s.y += s.v * dt;
      if (s.seek) {
        const step = s.seek * (1 - Math.exp(-dt / 0.12));
        s.y += step;
        s.seek = Math.abs(s.seek - step) < 0.5 ? 0 : s.seek - step;
      }
    }
    s.y = focusC[i] ? clamp(s.y, -s.h, s.h * 2) : mod(s.y, s.h);
    track.style.transform = `translate3d(0, ${-(s.h + s.y)}px, 0)`;
  }
  raf = visible ? requestAnimationFrame(frame) : 0;
};

const colIndex = (target) => {
  const col = target && target.closest ? target.closest(".ui-tw__col") : null;
  return col ? cols().indexOf(col) : -1;
};

new IntersectionObserver(([e]) => {
  visible = e.isIntersecting;
  if (visible && !raf) { last = 0; raf = requestAnimationFrame(frame); }
}).observe(root);

root.addEventListener("pointerover", (e) => {
  const i = colIndex(e.target);
  if (i >= 0 && e.pointerType !== "touch") hoverC[i] = true;
  const text = e.target.closest && e.target.closest(".ui-tw__quote p, .ui-tw__name-text, .ui-tw__meta");
  if (text) {
    const cut = text.scrollWidth > text.clientWidth + 1 || text.scrollHeight > text.clientHeight + 1;
    text.title = cut ? text.textContent : "";
  }
});
root.addEventListener("pointerout", (e) => {
  const i = colIndex(e.target);
  const col = cols()[i];
  if (i >= 0 && col && !col.contains(e.relatedTarget)) hoverC[i] = false;
});
root.addEventListener("pointerdown", (e) => {
  const i = colIndex(e.target);
  const s = st[i];
  if (!s) return;
  if (e.pointerType === "touch") { holdC[i] = true; return; }
  if (e.button !== 0) return;
  s.drag = { id: e.pointerId, lastY: e.clientY, lastT: e.timeStamp, moved: 0, captured: false };
  s.v = 0;
});
root.addEventListener("pointermove", (e) => {
  const i = colIndex(e.target);
  const s = st[i];
  const d = s && s.drag;
  if (!s || !d || d.id !== e.pointerId) return;
  const col = cols()[i];
  const scale = col.getBoundingClientRect().height / (col.offsetHeight || 1) || 1;
  const dy = (e.clientY - d.lastY) / scale;
  const dt = Math.max(1, e.timeStamp - d.lastT) / 1000;
  d.moved += Math.abs(dy);
  if (!d.captured && d.moved > 4) {
    d.captured = true;
    col.setPointerCapture(e.pointerId);
    col.setAttribute("data-dragging", "");
  }
  s.y -= dy;
  s.v = s.v * 0.6 + (-dy / dt) * 0.4;
  d.lastY = e.clientY;
  d.lastT = e.timeStamp;
});
const release = (e) => {
  const i = colIndex(e.target);
  const s = st[i];
  const d = s && s.drag;
  holdC[i] = false;
  if (!s || !d || d.id !== e.pointerId) return;
  if (e.timeStamp - d.lastT > 80) s.v = 0;
  s.v = clamp(s.v, -2400, 2400);
  s.drag = null;
  const col = cols()[i];
  col.removeAttribute("data-dragging");
  if (d.captured) {
    const stop = (ev) => { ev.preventDefault(); ev.stopPropagation(); };
    col.addEventListener("click", stop, { capture: true, once: true });
    setTimeout(() => col.removeEventListener("click", stop, { capture: true }), 0);
  }
};
root.addEventListener("pointerup", release);
root.addEventListener("pointercancel", release);

root.addEventListener("focusin", (e) => {
  const i = colIndex(e.target);
  if (i < 0) return;
  focusC[i] = e.target.matches(":focus-visible");
  const card = e.target.closest(".ui-tw__card");
  const s = st[i];
  const col = cols()[i];
  if (!card || !s || !col || s.h <= 0 || !card.matches(":focus-visible") || mq.matches) return;
  const top = card.parentElement.offsetTop;
  const h = card.offsetHeight;
  const viewH = col.clientHeight;
  const pad = Math.min(56, viewH * 0.12);
  s.y = [s.y - s.h, s.y, s.y + s.h].reduce((best, y) =>
    Math.abs(top - y - viewH / 2) < Math.abs(top - best - viewH / 2) ? y : best,
  );
  const at = top - s.y;
  if (h > viewH - pad * 2 || at < pad) s.seek = at - pad;
  else if (at + h > viewH - pad) s.seek = at + h - (viewH - pad);
  else s.seek = 0;
});
root.addEventListener("focusout", (e) => {
  const i = colIndex(e.target);
  const col = cols()[i];
  if (i >= 0 && col && !col.contains(e.relatedTarget)) focusC[i] = false;
});

mq.addEventListener("change", send);
new ResizeObserver(() => measure()).observe(root);
measure();
send();
"#;

#[derive(Clone, PartialEq)]
pub struct Testimonial {
    /// The quote itself. Mix short and long ones: the masonry looks more natural.
    pub quote: String,
    pub name: String,
    /// e.g. "Design Engineer".
    pub role: Option<String>,
    /// e.g. "Fieldnote", shown after the role.
    pub company: Option<String>,
    /// Photo URL; without one the initials sit on a soft circle.
    pub avatar: Option<String>,
    /// 1 to 5, a small gray star row above the quote.
    pub rating: Option<u8>,
    /// A small gray check beside the name.
    pub verified: bool,
}

impl Testimonial {
    pub fn new(quote: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            quote: quote.into(),
            name: name.into(),
            role: None,
            company: None,
            avatar: None,
            rating: None,
            verified: false,
        }
    }
}

/// Suffixes that are not a family name: "Christopher Montgomery III" reads "CM", not "CI".
fn is_suffix(word: &str) -> bool {
    matches!(
        word.trim_end_matches([',', '.']).to_lowercase().as_str(),
        "jr" | "sr" | "i" | "ii" | "iii" | "iv" | "v" | "vi" | "phd" | "md" | "esq"
    )
}

/// First and last word, so "María José de la Cruz y Fernández" reads "MF". Words that start
/// with a letter win, so "🦊 Fox" reads "F".
pub fn initials(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();
    let named: Vec<&str> = words
        .iter()
        .copied()
        .filter(|w| w.chars().next().is_some_and(char::is_alphabetic))
        .collect();
    let list = if named.is_empty() { words } else { named };
    let core: Vec<&str> = if list.len() > 1 {
        list.iter()
            .enumerate()
            .filter(|(i, w)| *i == 0 || !is_suffix(w))
            .map(|(_, w)| *w)
            .collect()
    } else {
        list
    };
    let picked: Vec<&str> = if core.len() > 1 {
        vec![core[0], core[core.len() - 1]]
    } else {
        core
    };
    let out: String = picked
        .iter()
        .filter_map(|word| word.chars().next())
        .flat_map(char::to_uppercase)
        .collect();
    if out.is_empty() { "?".to_string() } else { out }
}

/// Container width to column count, capped at `max`; matches the engine's breakpoints.
pub fn fit_columns(width: f64, max: u8) -> u8 {
    let fit: u8 = if width >= 740.0 {
        3
    } else if width >= 480.0 {
        2
    } else {
        1
    };
    fit.min(max).max(1)
}

/// Quote indices dealt into each column: 1, 2, 3, 1, 2, 3, so short and long ones mix.
pub fn deal(len: usize, columns: usize) -> Vec<Vec<usize>> {
    (0..columns)
        .map(|c| (0..len).filter(|i| i % columns == c).collect())
        .collect()
}

#[component]
fn Avatar(src: Option<String>, name: String) -> Element {
    let mut broken = use_signal(|| false);
    match src {
        Some(src) if !broken() => rsx! {
            img {
                class: "ui-tw__avatar",
                src,
                alt: "",
                width: "36",
                height: "36",
                "loading": "lazy",
                "decoding": "async",
                draggable: "false",
                onerror: move |_| broken.set(true),
            }
        },
        _ => rsx! {
            span { class: "ui-tw__avatar ui-tw__avatar--initials", "aria-hidden": "true", {initials(&name)} }
        },
    }
}

#[component]
fn Card(quote: Testimonial, interactive: bool) -> Element {
    let stars = quote.rating.map(|r| r.clamp(1, 5)).unwrap_or(0);
    let meta = [quote.role.as_deref(), quote.company.as_deref()]
        .into_iter()
        .flatten()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\u{a0}· ");
    let text = quote.quote.trim().to_string();
    rsx! {
        li { class: "ui-tw__item",
            figure {
                class: "ui-tw__card",
                tabindex: interactive.then_some("0"),
                if stars > 0 {
                    div { class: "ui-tw__stars", role: "img", "aria-label": "Rated {stars} out of 5",
                        for n in 0..5u8 {
                            svg { "viewBox": "0 0 16 16", "aria-hidden": "true", "data-on": (n < stars).then_some(""),
                                path { d: "M8 1.6l1.9 3.95 4.35.6-3.17 3.03.78 4.32L8 11.43 4.14 13.5l.78-4.32L1.75 6.15l4.35-.6L8 1.6z" }
                            }
                        }
                    }
                }
                if !text.is_empty() {
                    blockquote { class: "ui-tw__quote",
                        p { dir: "auto", "{text}" }
                    }
                }
                figcaption { class: "ui-tw__who",
                    Avatar { src: quote.avatar.clone(), name: quote.name.trim().to_string() }
                    span { class: "ui-tw__id",
                        span { class: "ui-tw__name",
                            span { class: "ui-tw__name-text", dir: "auto", "{quote.name.trim()}" }
                            if quote.verified {
                                svg { class: "ui-tw__verified", "viewBox": "0 0 16 16", role: "img", "aria-label": "Verified customer",
                                    circle { cx: "8", cy: "8", r: "7" }
                                    path {
                                        d: "M5.2 8.1l1.9 1.9 3.7-3.8",
                                        fill: "none",
                                        stroke: "var(--ui-tw-card)",
                                        stroke_width: "1.6",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                    }
                                }
                            }
                        }
                        if !meta.is_empty() {
                            span { class: "ui-tw__meta", dir: "auto", "{meta}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn TestimonialWall(
    items: Vec<Testimonial>,
    #[props(default = 3)] columns: u8,
    #[props(default = 28.0)] speed: f64,
    #[props(default = true)] pause_on_hover: bool,
    #[props(default = "What customers say".to_string(), into)] label: String,
    #[props(default)] class: String,
) -> Element {
    let id = use_hook(|| format!("ui-tw-{}", next_id()));
    let mut reduced = use_signal(|| false);
    let mut fit = use_signal(|| columns.max(1));
    let mut counts = use_signal(Vec::<usize>::new);

    let engine_id = id.clone();
    use_effect(move || {
        let mut channel = document::eval(&SCRIPT.replace("__ID__", &engine_id));
        spawn(async move {
            while let Ok((is_reduced, measured_fit, measured)) =
                channel.recv::<(bool, u8, Vec<usize>)>().await
            {
                reduced.set(is_reduced);
                fit.set(measured_fit);
                if !measured.is_empty() {
                    counts.set(measured);
                }
            }
        });
    });

    let count = (fit() as usize).min(items.len()).max(1);
    let dealt = deal(items.len(), count);
    let reduced_now = reduced();
    let class = if class.is_empty() {
        "ui-tw".to_string()
    } else {
        format!("ui-tw {class}")
    };
    let style = format!("--ui-tw-cols: {count}");
    let max = columns.max(1).to_string();
    let speed_attr = format!("{speed}");
    let pause_attr = if pause_on_hover { "true" } else { "false" };
    let items_attr = items.len().to_string();

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            id: "{id}",
            class,
            role: "region",
            "aria-label": "{label}",
            "data-cols": "{count}",
            "data-max": "{max}",
            "data-items": "{items_attr}",
            "data-speed": "{speed_attr}",
            "data-pause": "{pause_attr}",
            "data-static": reduced_now.then_some(""),
            "data-empty": items.is_empty().then_some(""),
            style,
            if items.is_empty() {
                p { class: "ui-tw__empty", "No quotes yet" }
            }
            for (i, list) in dealt.iter().enumerate() {
                if reduced_now {
                    div { key: "{i}", class: "ui-tw__col",
                        ul { class: "ui-tw__set",
                            for n in list.iter().copied() {
                                Card { key: "{n}", quote: items[n].clone(), interactive: true }
                            }
                        }
                    }
                } else {
                    div { key: "{i}", class: "ui-tw__col",
                        div { class: "ui-tw__track",
                            for c in 0..(1 + counts().get(i).copied().unwrap_or(2).max(2)) {
                                ul {
                                    key: "{c}",
                                    class: "ui-tw__set",
                                    "data-real": (c == 1).then_some(""),
                                    "aria-hidden": (c != 1).then_some("true"),
                                    "inert": (c != 1).then_some(""),
                                    for n in list.iter().copied() {
                                        Card { key: "{n}", quote: items[n].clone(), interactive: c == 1 }
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
