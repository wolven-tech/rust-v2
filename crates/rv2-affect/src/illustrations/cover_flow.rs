//! Browse covers in 3D. The cover in front faces you; the others turn away and stack up on both
//! sides, with a soft reflection on the floor. Drag, flick, scroll sideways, use the arrow keys
//! or click a cover: a spring carries the row along and settles on the nearest cover. The
//! caption rolls to the new title.
//!
//! Port losses: `renderCaption` is not supported; the caption shows `title` and `subtitle`. The
//! spring, drag and paint run in one `document::eval` script, which reports the front cover
//! back to Rust. A cover whose image fails to load keeps its alt text hidden only by the
//! browser's default, not by the original's visibility rule.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};

const CSS: Asset = asset!("/assets/cover-flow.css");

/// Motion engine. `__ID__` is the root's DOM id. The root carries `data-item-width`,
/// `data-angle`, `data-start`, `data-sound` and `data-reflection`. Each change of the front
/// cover posts `[false, index, forward]`; a select posts `[true, index, false]`.
const SCRIPT: &str = r#"
const root = document.getElementById("__ID__");
if (!root) return;
const viewport = root.querySelector(".ui-cf__viewport");
const stage = root.querySelector(".ui-cf__stage");
const cards = () => Array.from(root.querySelectorAll(".ui-cf__item"));
const mq = window.matchMedia("(prefers-reduced-motion: reduce)");
const VISIBLE = 7;
const DRAG_PX = 110;
const last = () => Math.max(cards().length - 1, 0);
const itemWidth = () => Number(root.dataset.itemWidth) || 280;
const itemHeight = () => Number(root.dataset.itemHeight) || 280;
const angle = () => Number(root.dataset.angle) || 50;
let scale = 1;
let audio = null;
const start = Math.min(Math.max(Number(root.dataset.start) || 0, 0), last());
const m = { pos: start, target: start, vel: 0, raf: 0, last: 0, active: start };

const wake = () => {
  if (root.dataset.sound !== "true") return;
  const Ctx = window.AudioContext || window.webkitAudioContext;
  if (!audio && Ctx) audio = new Ctx();
  if (audio && audio.state === "suspended") audio.resume();
};
const tick = () => {
  if (!audio || audio.state !== "running") return;
  const t = audio.currentTime;
  const osc = audio.createOscillator();
  const gain = audio.createGain();
  osc.type = "triangle";
  osc.frequency.setValueAtTime(2200, t);
  osc.frequency.exponentialRampToValueAtTime(900, t + 0.03);
  gain.gain.setValueAtTime(0.0001, t);
  gain.gain.exponentialRampToValueAtTime(0.05, t + 0.004);
  gain.gain.exponentialRampToValueAtTime(0.0001, t + 0.045);
  osc.connect(gain).connect(audio.destination);
  osc.start(t);
  osc.stop(t + 0.05);
};

const paint = () => {
  const pos = m.pos;
  const a = angle();
  const w = itemWidth() * scale;
  const gap = w * 0.78;
  const step = itemWidth() * 0.27 * scale;
  const depth = itemWidth() * 0.7 * scale;
  cards().forEach((el, i) => {
    const d = i - pos;
    const ad = Math.abs(d);
    if (ad > VISIBLE + 1) {
      el.style.visibility = "hidden";
      return;
    }
    const t = Math.min(ad, 1);
    const dir = Math.sign(d);
    const x = dir * (t * gap + Math.max(ad - 1, 0) * step);
    el.style.visibility = "";
    const k = 1 - t * 0.08 - Math.min(Math.max(ad - 1, 0), 4) * 0.015;
    el.style.transform = `translate3d(${x}px, 0, ${-t * depth}px) rotateY(${-dir * t * a}deg) scale(${k.toFixed(4)})`;
    el.style.zIndex = String(1000 - Math.round(ad * 10));
    el.style.setProperty("--ui-cf-t", t.toFixed(3));
    el.style.setProperty("--ui-cf-d", Math.min(Math.max(d, -1.5), 1.5).toFixed(3));
    el.dataset.side = d < 0 ? "left" : "right";
  });
  const now = Math.round(Math.min(Math.max(pos, 0), last()));
  if (now !== m.active) {
    const forward = now > m.active;
    m.active = now;
    root.dataset.dir = forward ? "next" : "prev";
    cards().forEach((el, i) => {
      el.toggleAttribute("data-active", i === now);
      el.setAttribute("aria-hidden", i === now ? "false" : "true");
    });
    dioxus.send([false, now, forward]);
    tick();
  }
};

const frame = (now) => {
  const dt = Math.min(now - m.last, 32) / 1000;
  m.last = now;
  const k = 190;
  const c = 2 * Math.sqrt(k) * 0.92;
  m.vel += (k * (m.target - m.pos) - c * m.vel) * dt;
  m.pos += m.vel * dt;
  if (Math.abs(m.target - m.pos) < 0.0006 && Math.abs(m.vel) < 0.002) {
    m.pos = m.target;
    m.vel = 0;
    m.raf = 0;
    paint();
    return;
  }
  paint();
  m.raf = requestAnimationFrame(frame);
};
const run = () => {
  if (m.raf) return;
  m.last = performance.now();
  m.raf = requestAnimationFrame(frame);
};
const goTo = (i) => {
  m.target = Math.min(Math.max(Math.round(i), 0), last());
  if (mq.matches) {
    m.pos = m.target;
    m.vel = 0;
    paint();
  } else {
    run();
  }
};
const select = (i) => dioxus.send([true, i, false]);
root.__goTo = goTo;

const fit = () => {
  scale = Math.min(1, Math.max(0.5, viewport.clientWidth / (itemWidth() * 3.2)));
  root.style.setProperty("--ui-cf-w", `${Math.round(itemWidth() * scale)}px`);
  root.style.setProperty("--ui-cf-h", `${Math.round(itemHeight() * scale)}px`);
  stage.style.perspective = `${Math.round(1000 * scale)}px`;
  paint();
};
new ResizeObserver(fit).observe(viewport);
fit();

let drag = null;
viewport.addEventListener("pointerdown", (e) => {
  if (e.button !== 0) return;
  wake();
  const hit = e.target.closest("[data-cf-index]");
  drag = {
    id: e.pointerId,
    x: e.clientX,
    from: m.pos,
    moved: false,
    hit: hit ? Number(hit.dataset.cfIndex) : null,
    samples: [{ x: e.clientX, t: performance.now() }],
  };
});
viewport.addEventListener("pointermove", (e) => {
  const d = drag;
  if (!d || d.id !== e.pointerId) return;
  const dx = e.clientX - d.x;
  if (!d.moved) {
    if (Math.abs(dx) < 6) return;
    d.moved = true;
    viewport.setPointerCapture(e.pointerId);
    viewport.dataset.dragging = "";
  }
  cancelAnimationFrame(m.raf);
  m.raf = 0;
  let p = d.from - dx / (DRAG_PX * scale);
  const max = last();
  if (p < 0) p = p * 0.3;
  if (p > max) p = max + (p - max) * 0.3;
  m.pos = m.target = p;
  m.vel = 0;
  d.samples.push({ x: e.clientX, t: performance.now() });
  if (d.samples.length > 6) d.samples.shift();
  paint();
});
const release = (e) => {
  const d = drag;
  if (!d || d.id !== e.pointerId) return;
  drag = null;
  delete viewport.dataset.dragging;
  if (!d.moved) {
    if (d.hit === null) return;
    if (d.hit === Math.round(m.target)) select(d.hit);
    else goTo(d.hit);
    return;
  }
  const first = d.samples[0];
  const end = d.samples[d.samples.length - 1];
  const dt = Math.max(end.t - first.t, 1);
  const v = (-((end.x - first.x) / dt) * 1000) / (DRAG_PX * scale);
  m.vel = mq.matches ? 0 : v;
  let to = Math.round(m.pos + v * 0.18);
  if (Math.abs(v) > 0.8 && to === Math.round(d.from)) to += Math.sign(v);
  goTo(to);
};
viewport.addEventListener("pointerup", release);
viewport.addEventListener("pointercancel", release);

viewport.addEventListener("keydown", (e) => {
  const cur = Math.round(m.target);
  const keys = { ArrowLeft: cur - 1, ArrowRight: cur + 1, Home: 0, End: last() };
  if (e.key in keys) {
    e.preventDefault();
    wake();
    goTo(keys[e.key]);
  } else if ((e.key === "Enter" || e.key === " ") && e.target === viewport) {
    e.preventDefault();
    select(cur);
  }
});

let snap = 0;
viewport.addEventListener("wheel", (e) => {
  const dx = e.shiftKey && !e.deltaX ? e.deltaY : e.deltaX;
  if (Math.abs(dx) <= Math.abs(e.deltaY) && !e.shiftKey) return;
  e.preventDefault();
  wake();
  const unit = DRAG_PX * scale * 1.4;
  m.target = Math.min(Math.max(m.target + dx / unit, -0.35), last() + 0.35);
  run();
  clearTimeout(snap);
  snap = setTimeout(() => goTo(m.target + Math.sign(dx) * 0.3), 110);
}, { passive: false });

while (true) {
  const v = await dioxus.recv();
  goTo(v);
}
"#;

#[derive(Clone, PartialEq)]
pub struct CoverFlowItem {
    pub id: String,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    /// Image URL. Use `cover` instead for any other content.
    pub image: Option<String>,
    pub alt: Option<String>,
    /// Any content instead of an image: artwork, a video, a product card.
    pub cover: Option<Element>,
}

impl CoverFlowItem {
    pub fn titled(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: Some(title.into()),
            subtitle: None,
            image: None,
            alt: None,
            cover: None,
        }
    }
}

#[component]
pub fn CoverFlow(
    items: Vec<CoverFlowItem>,
    /// Controlled front cover.
    #[props(default)]
    index: Option<usize>,
    /// Starting cover when uncontrolled. Defaults to the middle one.
    #[props(default)]
    default_index: Option<usize>,
    #[props(default)] on_index_change: Option<EventHandler<usize>>,
    #[props(default)] on_select: Option<EventHandler<usize>>,
    #[props(default = 280.0)] item_width: f64,
    #[props(default = 280.0)] item_height: f64,
    #[props(default = 50.0)] angle: f64,
    #[props(default = true)] reflection: bool,
    #[props(default = true)] caption: bool,
    #[props(default = true)] controls: bool,
    #[props(default = true)] ticks: bool,
    #[props(default = false)] sound: bool,
    #[props(default = "Cover flow".to_string(), into)] aria_label: String,
    #[props(default, into)] class: String,
) -> Element {
    let id = use_hook(|| format!("ui-cf-{}", next_id()));
    let count = items.len();
    let last = count.saturating_sub(1);
    let start = index.or(default_index).unwrap_or(last / 2).min(last);
    let mut active = use_signal(|| start);
    let mut forward = use_signal(|| true);

    let engine_id = id.clone();
    let select_handler = on_select;
    use_effect(move || {
        let script = SCRIPT.replace("__ID__", &engine_id);
        let mut channel = document::eval(&script);
        spawn(async move {
            while let Ok((is_select, position, moved_forward)) =
                channel.recv::<(bool, usize, bool)>().await
            {
                if is_select {
                    if let Some(handler) = select_handler {
                        handler.call(position);
                    }
                } else {
                    active.set(position);
                    forward.set(moved_forward);
                    if let Some(handler) = on_index_change {
                        handler.call(position);
                    }
                }
            }
        });
    });

    let goto_id = id.clone();
    use_effect(use_reactive((&index,), move |(index,)| {
        if let Some(target) = index {
            document::eval(&format!(
                "document.getElementById({goto_id:?})?.__goTo?.({target});"
            ));
        }
    }));

    let class = class_list("ui-cf", &class);
    let item_width_attr = item_width.to_string();
    let item_height_attr = item_height.to_string();
    let angle_attr = angle.to_string();
    let start_attr = start.to_string();
    let sound_attr = if sound { "true" } else { "false" };
    let current = active();
    let caption_dir = if forward() { "next" } else { "prev" };
    let viewport_id = format!("{id}-viewport");
    let prev_dom = id.clone();
    let next_dom = id.clone();
    let tick_dom = id.clone();
    let counter = format!("{} / {count}", current + 1);
    let tick_labels: Vec<String> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            item.title.clone().map_or_else(
                || format!("Show cover {}", i + 1),
                |title| format!("Show {title}"),
            )
        })
        .collect();

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            id: "{id}",
            class,
            "data-reflection": reflection.then_some(""),
            "data-item-width": "{item_width_attr}",
            "data-item-height": "{item_height_attr}",
            "data-angle": "{angle_attr}",
            "data-start": "{start_attr}",
            "data-sound": "{sound_attr}",
            div {
                id: "{viewport_id}",
                class: "ui-cf__viewport",
                role: "region",
                "aria-roledescription": "carousel",
                "aria-label": "{aria_label}",
                tabindex: "0",
                "data-selectable": on_select.is_some().then_some(""),
                div { class: "ui-cf__stage",
                    for (i, item, ordinal) in items.iter().cloned().enumerate().map(|(i, item)| (i, item, i + 1)) {
                        div {
                            key: "{item.id}",
                            class: "ui-cf__item",
                            "data-cf-index": "{i}",
                            role: "group",
                            "aria-roledescription": "slide",
                            "aria-label": "{ordinal} of {count}",
                            div { class: "ui-cf__ground", "aria-hidden": "true" }
                            div { class: "ui-cf__card",
                                {media(&item, false)}
                            }
                            if reflection {
                                div { class: "ui-cf__reflection", "aria-hidden": "true", "inert": "",
                                    div { class: "ui-cf__card",
                                        {media(&item, true)}
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if caption || controls || ticks {
                div { class: "ui-cf__footer",
                    div { class: "ui-cf__row",
                        if controls {
                            button { r#type: "button", class: "ui-cf__nav", "aria-label": "Previous", disabled: current == 0,
                                onclick: move |_| { document::eval(&goto_script(&prev_dom, current.saturating_sub(1))); },
                                svg { "viewBox": "0 0 16 16", "aria-hidden": "true", path { d: "M10 3.5 5.5 8l4.5 4.5" } }
                            }
                        }
                        if caption {
                            div { class: "ui-cf__caption", "aria-live": "polite",
                                if let Some(item) = items.get(current) {
                                    div { key: "{item.id}", class: "ui-cf__caption-in", "data-dir": caption_dir,
                                        if let Some(title) = item.title.clone() {
                                            div { class: "ui-cf__title", "{title}" }
                                        }
                                        if let Some(subtitle) = item.subtitle.clone() {
                                            div { class: "ui-cf__subtitle", "{subtitle}" }
                                        }
                                    }
                                }
                            }
                        }
                        if controls {
                            button { r#type: "button", class: "ui-cf__nav", "aria-label": "Next", disabled: current == last,
                                onclick: move |_| { document::eval(&goto_script(&next_dom, current + 1)); },
                                svg { "viewBox": "0 0 16 16", "aria-hidden": "true", path { d: "M6 3.5 10.5 8 6 12.5" } }
                            }
                        }
                    }
                    if ticks && count > 15 {
                        div { class: "ui-cf__counter", "aria-hidden": "true", "{counter}" }
                    }
                    if ticks && count > 1 && count <= 15 {
                        div { class: "ui-cf__ticks",
                            for (i, label) in tick_labels.iter().enumerate() {
                                button {
                                    key: "{i}",
                                    r#type: "button",
                                    tabindex: "-1",
                                    class: "ui-cf__tick",
                                    "aria-label": "{label}",
                                    "aria-current": (i == current).then_some("true"),
                                    onclick: {
                                        let dom = tick_dom.clone();
                                        move |_| { document::eval(&goto_script(&dom, i)); }
                                    },
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn goto_script(dom_id: &str, target: usize) -> String {
    format!("document.getElementById({dom_id:?})?.__goTo?.({target});")
}

fn media(item: &CoverFlowItem, decorative: bool) -> Element {
    let alt = if decorative {
        String::new()
    } else {
        item.alt
            .clone()
            .or_else(|| item.title.clone())
            .unwrap_or_default()
    };
    rsx! {
        div { class: "ui-cf__media",
            if let Some(cover) = item.cover.clone() {
                {cover}
            } else if let Some(src) = item.image.clone() {
                img { src, alt, draggable: "false", "loading": "lazy", "decoding": "async" }
            }
        }
    }
}
