//! A little cloud with eyes on a coloured sky. A bright coloured body sits under a white glow
//! that reaches further in at the top and stays subtle at the bottom. It floats and breathes,
//! looks left and right, blinks now and then, follows the pointer with its eyes, and gives a
//! squishy bounce and a wink when clicked. Keeps still with reduced motion.
//!
//! Port losses: the animation loop (springs, blink, pointer) runs in one `document::eval`
//! script on requestAnimationFrame, and pauses off screen.

use dioxus::prelude::*;

use crate::illustrations::frost_cloud::CloudColor;
use crate::internal::{class_list, next_id};

const CSS: Asset = asset!("/assets/glow-cloud.css");

/// Motion engine, the same as the frost cloud's. `__ID__` is the root's DOM id and `__P__`
/// the class prefix. Reads `data-follow` from the root each event.
const SCRIPT: &str = r#"
const root = document.getElementById("__ID__");
if (!root) return;
const q = (name) => root.querySelector(".__P____" + name);
const floatEl = q("float");
const turnEl = q("turn");
const eyesEl = q("eyes");
const leftEl = q("left");
const rightEl = q("right");
const shineEl = q("shine");
const shadowEl = q("shadow");
const still = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
const CX = 120;
const CY = 120;
const BOTTOM = 180;
const EYES = { y: 106, w: 18, h: 31, left: 96.5, right: 127.5 };
const GLANCES = [
  { x: 0, y: 0 }, { x: -1, y: 0.05 }, { x: 1, y: -0.05 }, { x: 0, y: 0 }, { x: 0.65, y: -0.55 }, { x: -0.7, y: 0.35 },
];
const about = (x, y, t) => `translate(${x} ${y}) ${t} translate(${-x} ${-y})`;
const look = { x: 0, y: 0, vx: 0, vy: 0 };
const target = { x: 0, y: 0 };
const squish = { p: 0, v: 0 };
let pointer = false;
let nextGlance = 1.2;
let nextBlink = 1.8;
let blinkAt = -1;
let blinkTwice = false;
let winkAt = -1;
let frame = 0;
let last = 0;
let time = 0;
const blinkScale = (since, length = 0.16) => (since < 0 || since > length ? 1 : 1 - 0.9 * Math.sin((since / length) * Math.PI));

const draw = () => {
  const float = still ? 0 : Math.sin(time * 1.1) * 5;
  const breathe = still ? 0 : Math.sin(time * 1.1 + Math.PI / 2) * 0.012;
  const sy = 1 + breathe - squish.p * 0.12;
  const sx = 1 - breathe + squish.p * 0.1;
  floatEl.setAttribute("transform", `translate(0 ${float.toFixed(2)}) ${about(CX, BOTTOM, `scale(${sx.toFixed(4)} ${sy.toFixed(4)})`)}`);
  turnEl.setAttribute("transform", `translate(${(look.x * 4).toFixed(2)} ${(look.y * 2).toFixed(2)}) ${about(CX, CY, `rotate(${(look.x * 2.5).toFixed(2)}) scale(${(1 - Math.abs(look.x) * 0.025).toFixed(4)} 1)`)}`);
  eyesEl.setAttribute("transform", `translate(${(look.x * 12).toFixed(2)} ${(look.y * 7).toFixed(2)})`);
  shineEl.setAttribute("transform", `translate(${(-look.x * 7).toFixed(2)} ${(-look.y * 3).toFixed(2)})`);
  const both = blinkAt < 0 ? 1 : Math.min(blinkScale(time - blinkAt), blinkTwice ? blinkScale(time - blinkAt - 0.26) : 1);
  const wink = blinkScale(time - winkAt, 0.34);
  const eyeY = EYES.y + EYES.h / 2;
  leftEl.setAttribute("transform", about(EYES.left + EYES.w / 2, eyeY, `scale(1 ${both.toFixed(3)})`));
  rightEl.setAttribute("transform", about(EYES.right + EYES.w / 2, eyeY, `scale(1 ${Math.min(both, wink).toFixed(3)})`));
  const lift = (float + 5) / 10;
  shadowEl.setAttribute("transform", about(CX, 210, `scale(${(1 - lift * 0.12).toFixed(3)} 1)`));
  shadowEl.setAttribute("opacity", (0.34 - lift * 0.1).toFixed(3));
};

const tick = (now) => {
  if (!root.isConnected) { stop(); return; }
  const dt = Math.min(0.05, (now - (last || now)) / 1000);
  last = now;
  time += dt;
  if (!pointer && time > nextGlance) {
    const g = GLANCES[Math.floor(Math.random() * GLANCES.length)];
    target.x = g.x;
    target.y = g.y;
    nextGlance = time + 1.4 + Math.random() * 2.2;
  }
  if (time > nextBlink) {
    blinkAt = time;
    blinkTwice = Math.random() < 0.25;
    nextBlink = time + 2.4 + Math.random() * 3;
  }
  look.vx += ((target.x - look.x) * 70 - look.vx * 13) * dt;
  look.vy += ((target.y - look.y) * 70 - look.vy * 13) * dt;
  look.x += look.vx * dt;
  look.y += look.vy * dt;
  squish.v += (-squish.p * 220 - squish.v * 9) * dt;
  squish.p += squish.v * dt;
  draw();
  frame = requestAnimationFrame(tick);
};
const start = () => {
  if (frame || still) return;
  last = 0;
  frame = requestAnimationFrame(tick);
};
const stop = () => {
  cancelAnimationFrame(frame);
  frame = 0;
};

new IntersectionObserver(([e]) => (e.isIntersecting ? start() : stop())).observe(root);
root.addEventListener("pointermove", (e) => {
  if (root.dataset.follow !== "true") return;
  const r = root.getBoundingClientRect();
  pointer = true;
  target.x = Math.max(-1, Math.min(1, ((e.clientX - r.left) / r.width - 0.5) * 2.4));
  target.y = Math.max(-1, Math.min(1, ((e.clientY - r.top) / r.height - 0.5) * 2.4));
});
root.addEventListener("pointerleave", () => {
  pointer = false;
  target.x = 0;
  target.y = 0;
  nextGlance = time + 1.2;
});
root.addEventListener("pointerdown", () => {
  squish.p = 0.9;
  squish.v = 0;
  winkAt = time + 0.12;
});
draw();
"#;

const PUFFS: [(f64, f64, f64); 5] = [
    (115.0, 93.0, 43.0),
    (163.0, 111.0, 39.0),
    (156.0, 155.0, 31.0),
    (107.0, 155.0, 35.0),
    (72.0, 126.0, 34.0),
];

const EYE_Y: f64 = 106.0;
const EYE_W: f64 = 18.0;
const EYE_H: f64 = 31.0;
const EYE_LEFT: f64 = 96.5;
const EYE_RIGHT: f64 = 127.5;

/// The five puffs and the core that the goo filter melts into one shape.
fn puffs() -> Element {
    rsx! {
        for (cx, cy, r) in PUFFS {
            circle { cx: "{cx}", cy: "{cy}", r: "{r}" }
        }
        rect { x: "80", y: "100", width: "80", height: "65" }
    }
}

/// One eye: the eye gradient inside the hole, and a light shade on its inner edge, deeper at the top.
fn eye(id: &str, x: f64, side: &str, class: &str) -> Element {
    let clip = format!("{id}-clip-{side}");
    let clip_url = format!("url(#{clip})");
    let edge = format!("url(#{id}-edge)");
    let fill = format!("url(#{id}-eye)");
    let (shade_y, rx) = (EYE_Y + 2.5, EYE_W / 2.0);
    rsx! {
        g { class,
            rect { x: "{x}", y: "{EYE_Y}", width: "{EYE_W}", height: "{EYE_H}", rx: "{rx}", fill: "{fill}" }
            clipPath { id: "{clip}",
                rect { x: "{x}", y: "{EYE_Y}", width: "{EYE_W}", height: "{EYE_H}", rx: "{rx}" }
            }
            rect { x: "{x}", y: "{shade_y}", width: "{EYE_W}", height: "{EYE_H}", rx: "{rx}", class: "ui-glw__eye-shade", fill: "none", "strokeWidth": "5", filter: "{edge}", "clipPath": "{clip_url}" }
        }
    }
}

#[component]
pub fn GlowCloud(
    #[props(default = true)] background: bool,
    #[props(default = CloudColor::Orange)] color: CloudColor,
    #[props(default = true)] follow_pointer: bool,
    #[props(default, into)] class: String,
    #[props(default = "A glowing cloud with eyes that floats, looks around and blinks".to_string(), into)]
    label: String,
) -> Element {
    let root_id = use_hook(|| format!("ui-glw-{}", next_id()));
    let effect_id = root_id.clone();
    use_effect(move || {
        let script = SCRIPT
            .replace("__ID__", &effect_id)
            .replace("__P__", "ui-glw");
        document::eval(&script);
    });

    let class = class_list("ui-glw", &class);
    let follow = follow_pointer.then_some("true");
    let goo = format!("url(#{root_id}-goo)");
    let soft = format!("url(#{root_id}-soft)");
    let halo = format!("url(#{root_id}-halo)");
    let edge = format!("url(#{root_id}-edge)");
    let shade = format!("url(#{root_id}-shade)");
    let mask = format!("url(#{root_id}-mask)");
    let rim = format!("url(#{root_id}-rim)");
    let body = format!("url(#{root_id}-body)");
    let shape = format!("#{root_id}-shape");
    let color_attr = color.as_str();
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            id: "{root_id}",
            class,
            "data-bg": background.then_some(""),
            "data-color": color_attr,
            "data-follow": follow,
            role: "img",
            "aria-label": "{label}",
            svg { "viewBox": "0 0 240 240", "aria-hidden": "true",
                defs {
                    filter { id: "{root_id}-goo", x: "0", y: "0", width: "240", height: "240", "filterUnits": "userSpaceOnUse",
                        feGaussianBlur { "stdDeviation": "3.5" }
                        feColorMatrix { values: "1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 28 -12" }
                    }
                    filter { id: "{root_id}-soft", x: "0", y: "0", width: "240", height: "240", "filterUnits": "userSpaceOnUse",
                        feGaussianBlur { "stdDeviation": "7" }
                    }
                    filter { id: "{root_id}-halo", x: "0", y: "0", width: "240", height: "240", "filterUnits": "userSpaceOnUse",
                        feGaussianBlur { "stdDeviation": "9" }
                    }
                    filter { id: "{root_id}-edge", x: "0", y: "0", width: "240", height: "240", "filterUnits": "userSpaceOnUse",
                        feGaussianBlur { "stdDeviation": "1.6" }
                    }
                    filter { id: "{root_id}-shade", x: "0", y: "0", width: "240", height: "240", "filterUnits": "userSpaceOnUse",
                        feGaussianBlur { "stdDeviation": "5" }
                    }
                    g { id: "{root_id}-shape", filter: "{goo}", {puffs()} }
                    mask { id: "{root_id}-mask", "maskUnits": "userSpaceOnUse", x: "0", y: "0", width: "240", height: "240",
                        r#use { href: "{shape}", fill: "#fff" }
                    }
                    linearGradient { id: "{root_id}-rim", x1: "0", y1: "56", x2: "0", y2: "182", "gradientUnits": "userSpaceOnUse",
                        stop { offset: "0", style: "stop-color: var(--ui-glw-rim)" }
                        stop { offset: "1", style: "stop-color: var(--ui-glw-white)" }
                    }
                    radialGradient { id: "{root_id}-body", cx: "120", cy: "108", r: "78", "gradientUnits": "userSpaceOnUse",
                        stop { offset: "0", style: "stop-color: var(--ui-glw-body-1)" }
                        stop { offset: "0.6", style: "stop-color: var(--ui-glw-body-2)" }
                        stop { offset: "1", style: "stop-color: var(--ui-glw-body-3)" }
                    }
                    linearGradient { id: "{root_id}-eye", x1: "0", y1: "0", x2: "0", y2: "1",
                        stop { offset: "0", style: "stop-color: var(--ui-glw-white)" }
                        stop { offset: "1", style: "stop-color: var(--ui-glw-eye)" }
                    }
                }

                ellipse { class: "ui-glw__shadow", cx: "120", cy: "210", rx: "54", ry: "7", filter: "{shade}" }

                g { class: "ui-glw__float",
                    g { class: "ui-glw__turn",
                        r#use { class: "ui-glw__halo", href: "{shape}", filter: "{halo}" }
                        r#use { class: "ui-glw__edge", href: "{shape}", filter: "{edge}" }
                        g { mask: "{mask}",
                            rect { width: "240", height: "240", fill: "{rim}" }
                            g { filter: "{soft}", fill: "{body}", transform: "translate(0 2) translate(120 120) scale(0.9 0.87) translate(-120 -120)",
                                {puffs()}
                            }
                            ellipse { class: "ui-glw__shine", cx: "104", cy: "80", rx: "30", ry: "12", fill: "#fff", opacity: "0.32", filter: "{shade}" }
                        }
                        g { class: "ui-glw__eyes",
                            {eye(&root_id, EYE_LEFT, "l", "ui-glw__left")}
                            {eye(&root_id, EYE_RIGHT, "r", "ui-glw__right")}
                        }
                    }
                }
            }
        }
    }
}
