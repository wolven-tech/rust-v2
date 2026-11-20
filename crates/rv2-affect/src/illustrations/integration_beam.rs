//! Your product in the middle, tools on each side joined by curved beams. In synced cycles,
//! light pulses travel from the tools on the left into the product and on out to the tools on
//! the right; every arrival makes its target glow. Hover a tool to bring its beam forward, click
//! it to pause that beam. The beams are measured from the real tile positions, so the scene
//! reflows with its container: on narrow widths the tools stack and the curves morph into place.
//!
//! Port losses: the measurement, morph and pulse loop runs in one `document::eval` script that
//! reports whether a sync cycle is running. Focus shows the tooltip on any focus, not only
//! keyboard focus, and a touch that is held over a tool also shows it.

use std::collections::BTreeSet;

use dioxus::prelude::*;

use crate::internal::{class_list, next_id, sleep};

const CSS: Asset = asset!("/assets/integration-beam.css");
const SPRITE: &str = include_str!("../../assets/integration-beam.svg");
const PETAL: &str = "M24 24C17.6 19.6 16.4 9.4 24 3.5C31.6 9.4 30.4 19.6 24 24Z";
/// How long a tool's tooltip stays up after a tap, in ms.
const FLASH_MS: u32 = 1400;

/// Motion engine. `__ID__` is the root's DOM id. Reads `data-left` once and, on each frame,
/// each tool's `data-off`. Posts `true`/`false` whenever a sync cycle starts or ends.
const SCRIPT: &str = r#"
const root = document.getElementById("__ID__");
if (!root) return;
const scene = root.querySelector(".ui-ibm__scene");
const svg = root.querySelector(".ui-ibm__beams");
const hub = root.querySelector(".ui-ibm__center");
const chip = root.querySelector(".ui-ibm__chip");
const chipInner = root.querySelector(".ui-ibm__chip-inner");
const statusEl = root.querySelector(".ui-ibm__status");
const centerRing = root.querySelector(".ui-ibm__ring--center");
const reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
const TRAVEL = 1500;
const STAGGER = 280;
const REST = 1500;
const MORPH = 460;
const EASE = "cubic-bezier(0.22, 1, 0.36, 1)";
const easeOut = (t) => 1 - (1 - t) ** 5;
const easeInOut = (t) => -(Math.cos(Math.PI * t) - 1) / 2;
const tools = () => Array.from(root.querySelectorAll(".ui-ibm__tool"));
const lines = () => Array.from(root.querySelectorAll(".ui-ibm__line"));
const pulses = () => Array.from(root.querySelectorAll(".ui-ibm__pulse"));
const grads = () => Array.from(root.querySelectorAll("linearGradient"));
const rings = () => tools().map((t) => t.querySelector(".ui-ibm__ring"));
const nLeft = Number(root.dataset.left);
const offSet = () => new Set(tools().filter((t) => t.hasAttribute("data-off")).map((t) => Number(t.dataset.idx)));
const centerOf = (el) => ({ x: el.offsetLeft + el.offsetWidth / 2, y: el.offsetTop + el.offsetHeight / 2 });
const curve = (a, b, stack) => {
  if (stack) {
    const my = (a.y + b.y) / 2;
    return [a.x, a.y, a.x, my, b.x, my, b.x, b.y];
  }
  const mx = (a.x + b.x) / 2;
  return [a.x, a.y, mx, a.y, mx, b.y, b.x, b.y];
};

let shown = [];
let from = [];
let to = [];
let morphStart = 0;
let lastPos = null;
let lastStack = null;
const lengths = [];
const draw = (curves) => {
  shown = curves;
  curves.forEach((c, i) => {
    const d = `M${c[0]} ${c[1]}C${c[2]} ${c[3]} ${c[4]} ${c[5]} ${c[6]} ${c[7]}`;
    const line = lines()[i];
    if (!line) return;
    line.setAttribute("d", d);
    pulses()[i]?.querySelectorAll("path").forEach((p) => p.setAttribute("d", d));
    lengths[i] = line.getTotalLength();
  });
};

const measure = () => {
  svg.setAttribute("viewBox", `0 0 ${root.clientWidth} ${root.clientHeight}`);
  const stack = getComputedStyle(scene).getPropertyValue("--ui-ibm-stack").trim() === "1";
  const list = tools();
  const pos = list.map(centerOf);
  const c = centerOf(hub);
  const target = pos.map((p, i) => (i < nLeft ? curve(p, c, stack) : curve(c, p, stack)));
  if (lastPos && lastStack !== null && stack !== lastStack && !reduce) {
    const els = [...list, hub];
    const all = [...pos, c];
    const prior = [...lastPos];
    els.forEach((el, i) => {
      const dx = prior[i].x - all[i].x;
      const dy = prior[i].y - all[i].y;
      if (dx || dy) el.animate([{ transform: `translate(${dx}px, ${dy}px)` }, { transform: "none" }], { duration: MORPH, easing: EASE });
    });
    from = shown.length === list.length ? shown : target;
    to = target;
    morphStart = performance.now();
    kick();
  } else if (morphStart) {
    to = target;
  } else {
    draw(target);
  }
  lastPos = [...pos, c];
  lastStack = stack;
};

let queue = [];
let nextCycle = 0;
let syncFrom = 0;
let syncTo = 0;
let wasSyncing = false;
let visible = false;
let raf = 0;
const pulseStart = [];

const hide = (i) => {
  pulseStart[i] = null;
  const g = pulses()[i];
  if (g) g.style.opacity = "0";
};
const ring = (el) =>
  el && el.animate(
    [
      { opacity: 0, transform: "scale(0.96)" },
      { opacity: 1, transform: "scale(1)", offset: 0.3 },
      { opacity: 0, transform: "scale(1.14)" },
    ],
    { duration: 760, easing: "ease-out" },
  );
const planCycle = (now) => {
  const off = offSet();
  const total = tools().length;
  const ins = [...Array(nLeft).keys()].filter((i) => !off.has(i));
  const outs = [...Array(total - nLeft).keys()].map((k) => nLeft + k).filter((i) => !off.has(i));
  if (!ins.length && !outs.length) {
    nextCycle = now + REST;
    return;
  }
  const t0 = now;
  const tOut = ins.length ? t0 + TRAVEL : t0;
  queue = [...ins.map((beam, k) => ({ beam, at: t0 + k * STAGGER })), ...outs.map((beam, k) => ({ beam, at: tOut + k * STAGGER }))];
  const end = Math.max(...queue.map((q) => q.at)) + TRAVEL;
  syncFrom = ins.length ? tOut : t0;
  syncTo = end;
  nextCycle = end + REST;
};
const paint = (i, t) => {
  const line = lines()[i];
  const g = pulses()[i];
  const grad = grads()[i];
  if (!line || !g || !grad) return;
  const L = lengths[i] || 0;
  const seg = Math.min(84, L * 0.42);
  const head = easeInOut(t) * (L + seg);
  g.querySelectorAll("path").forEach((p) => {
    p.style.strokeDasharray = `${seg} ${L + seg}`;
    p.style.strokeDashoffset = `${seg - head}`;
  });
  const a = line.getPointAtLength(Math.max(0, Math.min(L, head - seg)));
  const b = line.getPointAtLength(Math.max(0, Math.min(L, head)));
  grad.setAttribute("x1", a.x);
  grad.setAttribute("y1", a.y);
  grad.setAttribute("x2", b.x + (b.x === a.x && b.y === a.y ? 0.01 : 0));
  grad.setAttribute("y2", b.y);
  g.style.opacity = "1";
};

const frame = (now) => {
  raf = 0;
  if (!root.isConnected) return;
  if (morphStart) {
    const t = Math.min(1, (now - morphStart) / MORPH);
    const e = easeOut(t);
    draw(to.map((c, i) => c.map((v, k) => (from[i]?.[k] ?? v) + (v - (from[i]?.[k] ?? v)) * e)));
    if (t >= 1) morphStart = 0;
  }
  if (visible && !reduce) {
    if (now >= nextCycle) planCycle(now);
    queue = queue.filter((q) => {
      if (q.at > now) return true;
      if (!offSet().has(q.beam) && pulseStart[q.beam] == null) pulseStart[q.beam] = q.at;
      return false;
    });
    const total = tools().length;
    for (let i = 0; i < total; i++) {
      const start = pulseStart[i];
      if (start == null) continue;
      if (offSet().has(i)) {
        hide(i);
        continue;
      }
      const t = (now - start) / TRAVEL;
      if (t >= 1) {
        hide(i);
        ring(i < nLeft ? centerRing : rings()[i]);
        continue;
      }
      paint(i, t);
    }
    const isSyncing = now >= syncFrom && now < syncTo;
    if (isSyncing !== wasSyncing) {
      wasSyncing = isSyncing;
      dioxus.send(isSyncing);
    }
  }
  if (morphStart || (visible && !reduce)) raf = requestAnimationFrame(frame);
};
const kick = () => {
  if (!raf) raf = requestAnimationFrame(frame);
};

new ResizeObserver(measure).observe(root);
new IntersectionObserver(([e]) => {
  visible = e.isIntersecting;
  if (visible) {
    nextCycle = performance.now() + 400;
    kick();
    return;
  }
  queue = [];
  for (let i = 0; i < tools().length; i++) hide(i);
  syncTo = 0;
  if (wasSyncing) {
    wasSyncing = false;
    dioxus.send(false);
  }
}).observe(root);

const fitChip = () => {
  if (chip && chipInner) chip.style.width = `${chipInner.offsetWidth}px`;
};
new MutationObserver(fitChip).observe(statusEl, { childList: true, characterData: true, subtree: true });
fitChip();
measure();
"#;

#[derive(Clone, PartialEq)]
pub struct BeamTool {
    /// Shown in the tooltip and read to screen readers, for example "Slack".
    pub name: String,
    /// The logo, drawn into a 28 × 28 box.
    pub logo: Element,
}

#[derive(Clone, PartialEq)]
pub struct BeamCenter {
    /// The product's name, used in the accessible description.
    pub name: String,
    /// The mark, drawn into a 36 × 36 box. A neutral flower when left out.
    pub logo: Option<Element>,
}

/// The sprite's symbol with this id, as inner SVG markup.
fn symbol(id: &str) -> &'static str {
    let needle = format!("<symbol id=\"{id}\"");
    let Some(start) = SPRITE.find(&needle) else {
        return "";
    };
    let body = start + SPRITE[start..].find('>').unwrap_or(0) + 1;
    let end = SPRITE[body..]
        .find("</symbol>")
        .map_or(body, |offset| body + offset);
    &SPRITE[body..end]
}

fn brand(id: &str) -> Element {
    rsx! {
        svg { "viewBox": "0 0 24 24", "aria-hidden": "true", dangerous_inner_html: symbol(id) }
    }
}

fn default_left() -> Vec<BeamTool> {
    vec![
        BeamTool {
            name: "Slack".into(),
            logo: brand("slack"),
        },
        BeamTool {
            name: "Notion".into(),
            logo: brand("notion"),
        },
        BeamTool {
            name: "Gmail".into(),
            logo: brand("gmail"),
        },
    ]
}

fn default_right() -> Vec<BeamTool> {
    vec![
        BeamTool {
            name: "HubSpot".into(),
            logo: brand("hubspot"),
        },
        BeamTool {
            name: "Linear".into(),
            logo: brand("linear"),
        },
        BeamTool {
            name: "Google Drive".into(),
            logo: brand("google-drive"),
        },
    ]
}

fn product_mark() -> Element {
    rsx! {
        svg { "viewBox": "0 0 48 48", "aria-hidden": "true",
            g { opacity: "0.36",
                for r in [45, 135, 225, 315] {
                    path { d: PETAL, transform: "rotate({r} 24 24)" }
                }
            }
            for r in [0, 90, 180, 270] {
                path { d: PETAL, transform: "rotate({r} 24 24)" }
            }
        }
    }
}

/// Pauses or resumes one tool, announces it, and shows its tooltip for a moment.
fn toggle(
    index: usize,
    name: &str,
    total: usize,
    mut off: Signal<BTreeSet<usize>>,
    mut announce: Signal<String>,
    mut flash: Signal<Option<usize>>,
    mut flash_token: Signal<u64>,
) {
    let pausing = !off().contains(&index);
    let mut next = off();
    if pausing {
        next.insert(index);
    } else {
        next.remove(&index);
    }
    let connected = total - next.len();
    let state = if pausing { "paused" } else { "connected" };
    announce.set(format!("{name} {state}. {connected} of {total} connected."));
    off.set(next);
    flash.set(Some(index));
    let token = flash_token() + 1;
    flash_token.set(token);
    spawn(async move {
        sleep(FLASH_MS).await;
        if flash_token() == token {
            flash.set(None);
        }
    });
}

#[component]
pub fn IntegrationBeam(
    #[props(default = default_left())] left: Vec<BeamTool>,
    #[props(default = default_right())] right: Vec<BeamTool>,
    #[props(default = BeamCenter { name: "Acme".into(), logo: None })] center: BeamCenter,
    #[props(default, into)] class: String,
    #[props(default)] label: Option<String>,
) -> Element {
    let (left, right) = if left.is_empty() && right.is_empty() {
        (default_left(), default_right())
    } else {
        (left, right)
    };
    let id = use_hook(|| format!("ui-ibm-{}", next_id()));
    let n_left = left.len();
    let total = n_left + right.len();

    let off = use_signal(BTreeSet::<usize>::new);
    let mut focus = use_signal(|| None::<usize>);
    let flash = use_signal(|| None::<usize>);
    let flash_token = use_signal(|| 0u64);
    let mut syncing = use_signal(|| false);
    let announce = use_signal(String::new);

    let engine_id = id.clone();
    use_effect(move || {
        let mut channel = document::eval(&SCRIPT.replace("__ID__", &engine_id));
        spawn(async move {
            while let Ok(running) = channel.recv::<bool>().await {
                syncing.set(running);
            }
        });
    });

    let mut tools: Vec<(usize, BeamTool)> = left.into_iter().enumerate().collect();
    tools.extend(
        right
            .into_iter()
            .enumerate()
            .map(|(i, tool)| (n_left + i, tool)),
    );
    let names: Vec<String> = tools.iter().map(|(_, tool)| tool.name.clone()).collect();

    let connected = total - off().len();
    let status = if connected == 0 {
        "All paused".to_string()
    } else if syncing() {
        "Syncing…".to_string()
    } else {
        format!("{connected} connected")
    };
    let label =
        label.unwrap_or_else(|| format!("{} connected to {}", center.name, names.join(", ")));
    let class = class_list("ui-ibm", &class);
    let focused = focus().is_some();
    let left_attr = n_left.to_string();
    let is_syncing = syncing() && connected > 0;

    let tile = move |index: usize, tool: BeamTool| {
        let paused = off().contains(&index);
        let tipped = focus() == Some(index) || flash() == Some(index);
        let name = tool.name.clone();
        let aria = format!(
            "{}, {}",
            tool.name,
            if paused { "paused" } else { "connected" }
        );
        let tool_name = tool.name.clone();
        rsx! {
            button {
                key: "{index}",
                r#type: "button",
                class: "ui-ibm__tool",
                "data-idx": "{index}",
                "aria-pressed": "{!paused}",
                "aria-label": "{aria}",
                "data-off": paused.then_some(""),
                "data-tip": tipped.then_some(""),
                onclick: move |_| toggle(index, &name, total, off, announce, flash, flash_token),
                onpointerenter: move |_| focus.set(Some(index)),
                onpointerleave: move |_| focus.set(None),
                onfocus: move |_| focus.set(Some(index)),
                onblur: move |_| focus.set(None),
                span { class: "ui-ibm__ring" }
                span { class: "ui-ibm__logo", "aria-hidden": "true", {tool.logo} }
                span { class: "ui-ibm__tip", "aria-hidden": "true",
                    "{tool_name}"
                    if paused {
                        span { class: "ui-ibm__tip-state", " · Paused" }
                    }
                }
            }
        }
    };

    let left_tools: Vec<Element> = tools[..n_left]
        .iter()
        .cloned()
        .map(|(i, t)| tile(i, t))
        .collect();
    let right_tools: Vec<Element> = tools[n_left..]
        .iter()
        .cloned()
        .map(|(i, t)| tile(i, t))
        .collect();
    let center_mark = center.logo.clone().unwrap_or_else(product_mark);

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            id: "{id}",
            class,
            role: "group",
            "aria-label": "{label}",
            "data-focus": focused.then_some(""),
            "data-left": "{left_attr}",
            svg { class: "ui-ibm__beams", "aria-hidden": "true",
                defs {
                    filter { id: "{id}-blur", "filterUnits": "userSpaceOnUse", x: "-100", y: "-100", width: "10000", height: "10000",
                        feGaussianBlur { "stdDeviation": "3" }
                    }
                    for i in 0..total {
                        linearGradient { key: "{i}", id: "{id}-g{i}", "gradientUnits": "userSpaceOnUse",
                            stop { offset: "0", class: "ui-ibm__stop-tail" }
                            stop { offset: "0.6", class: "ui-ibm__stop-mid" }
                            stop { offset: "1", class: "ui-ibm__stop-head" }
                        }
                    }
                }
                for i in 0..total {
                    path { key: "{i}", class: "ui-ibm__line", "data-off": offset_attr(off(), i), "data-focus": (focus() == Some(i)).then_some("") }
                }
                for i in 0..total {
                    g { key: "{i}", class: "ui-ibm__pulse", "data-focus": (focus() == Some(i)).then_some(""),
                        path { class: "ui-ibm__glow", stroke: "url(#{id}-g{i})", filter: "url(#{id}-blur)" }
                        path { class: "ui-ibm__core", stroke: "url(#{id}-g{i})" }
                    }
                }
            }

            div { class: "ui-ibm__scene",
                div { class: "ui-ibm__group ui-ibm__group--in", {left_tools.into_iter()} }
                div { class: "ui-ibm__center", "data-syncing": is_syncing.then_some(""),
                    span { class: "ui-ibm__ring ui-ibm__ring--center" }
                    span { class: "ui-ibm__mark", "aria-hidden": "true", {center_mark} }
                    span { class: "ui-ibm__chip",
                        span { class: "ui-ibm__chip-inner",
                            span { class: "ui-ibm__dot", "data-idle": (connected == 0).then_some("") }
                            span { class: "ui-ibm__status", "{status}" }
                        }
                    }
                }
                div { class: "ui-ibm__group ui-ibm__group--out", {right_tools.into_iter()} }
            }

            span { class: "ui-ibm__sr", "aria-live": "polite", "{announce}" }
        }
    }
}

fn offset_attr(off: BTreeSet<usize>, index: usize) -> Option<&'static str> {
    off.contains(&index).then_some("")
}
