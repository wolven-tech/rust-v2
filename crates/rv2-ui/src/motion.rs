//! Playful, physics-flavoured components.
//!
//! ## Where these came from
//!
//! Reimplementations of ideas from **[FeralUI](https://feralui.dev)** by Sarthak
//! Navalekar — "components that move like real things". FeralUI is React and
//! ships on npm; this workspace has no JavaScript package manager and no React,
//! and `crates/rv2-ui` exists precisely so it does not need either. So these are
//! **not ports**: no FeralUI code is used or translated. They are independent
//! Dioxus implementations of the same behaviours, written from the public
//! descriptions of what each component does.
//!
//! Credit belongs upstream for the ideas. Anything clumsy here is ours.
//!
//! ## The constraint that shapes all of it
//!
//! `rv2-ui` depends on `dioxus` and nothing else, and that is a documented
//! property of the crate rather than an accident. So there is **no per-frame
//! loop** anywhere below: no `requestAnimationFrame`, no `web-sys`, no timer, no
//! spring solver. Motion is CSS; state is signals; pointer position arrives
//! through Dioxus events and leaves as CSS custom properties.
//!
//! That buys a real cost and it should be stated plainly rather than glossed:
//!
//! | Component | Upstream | Here |
//! |---|---|---|
//! | [`Blob`] | SVG mascot, moods, gaze, poke | Same, faithfully |
//! | [`Hologram`] | Bump-mapped `feDiffuseLighting` + canvas particles | Tilt, foil, sheen and border glow — no particles, no relief lighting |
//! | [`PullCord`] | A simulated rope | A drag plus an overshooting settle. Feels right; is not a solver |
//! | [`Crumple`] | Paper crumple | A keyframed collapse |
//! | [`Vacuum`] | Sucked away | A keyframed collapse toward a point |
//! | [`Fur`] | Fur texture | Layered repeating gradients that ruffle on hover |
//! | [`GradientBuilder`] | Gradient editor | Same, faithfully — it is pure state, so nothing is lost |
//!
//! Two of those are honest approximations. Where a thing simulates upstream and
//! eases here, the doc comment on the component says so.
//!
//! ## Accessibility
//!
//! Everything decorative collapses under `prefers-reduced-motion: reduce` (see
//! `assets/motion.css`), and every interactive element here is a real `button`
//! with a label. An idle wobble that cannot be turned off is exactly the kind of
//! motion that hurts.

use dioxus::prelude::*;

// ═════════════════════════════════════════════════════════════════════════════
// Blob
// ═════════════════════════════════════════════════════════════════════════════

/// What the blob is feeling. Drives the eyes and mouth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mood {
    #[default]
    Neutral,
    Happy,
    Sad,
    Angry,
    /// Thinking. One eye narrowed.
    Hmm,
    /// Looking away, unimpressed.
    SideEye,
    /// Eyes shut. For a password field — the blob is not looking.
    Password,
}

impl Mood {
    /// Eye shape, as an SVG path fragment per eye.
    const fn eyes_closed(self) -> bool {
        matches!(self, Mood::Password | Mood::Happy)
    }

    const fn brow_tilt(self) -> f32 {
        match self {
            Mood::Angry => -14.0,
            Mood::Sad => 12.0,
            Mood::Hmm => -6.0,
            _ => 0.0,
        }
    }

    /// The mouth, as a quadratic curve. Positive bows down (sad), negative bows
    /// up (happy).
    const fn mouth_curve(self) -> f32 {
        match self {
            Mood::Happy => -9.0,
            Mood::Sad => 8.0,
            Mood::Angry => 6.0,
            Mood::Hmm | Mood::SideEye => 1.0,
            _ => -2.0,
        }
    }
}

/// A jelly mascot, in SVG.
///
/// Poke it. It squashes, rebounds, and after enough pokes it has had enough —
/// `on_overpoke` fires, which is the hook for the joke.
///
/// Every colour is a CSS custom property (`--jelly-body`, `--jelly-cheek`,
/// `--jelly-eye`), so one component reskins per context by setting them on any
/// ancestor. That is upstream's design and it is the right one: a `color` prop
/// would need a variant per palette.
///
/// `gaze` nudges where it looks, in viewBox units. Pair it with a focused input
/// and the blob reads along as you type.
#[component]
pub fn Blob(
    #[props(default)] mood: Mood,
    /// Where it looks, in viewBox units, clamped to a small range so the pupils
    /// stay inside the eyes.
    #[props(default = (0.0, 0.0))]
    gaze: (f32, f32),
    #[props(default = 128)] size: u32,
    /// How many pokes it tolerates before `on_overpoke`.
    #[props(default = 5)]
    patience: u32,
    #[props(default)] on_overpoke: Option<EventHandler<()>>,
    #[props(default)] label: Option<String>,
) -> Element {
    let mut pokes = use_signal(|| 0_u32);
    // Toggled to restart the poke animation: re-applying the same class does
    // not replay a CSS animation, so the class has to actually change.
    let mut poking = use_signal(|| false);

    let (gx, gy) = (gaze.0.clamp(-10.0, 10.0), gaze.1.clamp(-6.0, 6.0));
    let closed = mood.eyes_closed();
    let brow = mood.brow_tilt();
    let curve = mood.mouth_curve();

    // SideEye looks away regardless of the gaze prop — the whole point of the
    // mood is that it is not looking at you.
    let (px, py) = if mood == Mood::SideEye {
        (7.5, -2.0)
    } else {
        (gx, gy)
    };

    let poke = move |_| {
        let n = pokes() + 1;
        pokes.set(n);
        poking.toggle();
        if n >= patience
            && let Some(handler) = &on_overpoke
        {
            handler.call(());
            pokes.set(0);
        }
    };

    let anim = if poking() {
        "animate-blob-poke"
    } else {
        "animate-blob-idle"
    };

    rsx! {
        button {
            r#type: "button",
            class: "relative inline-block cursor-pointer border-0 bg-transparent p-0 \
                    focus:outline-none focus-visible:ring-2 focus-visible:ring-slate-900 \
                    focus-visible:ring-offset-2 rounded-full",
            style: "width: {size}px; height: {size}px;",
            aria_label: label.unwrap_or_else(|| "Poke the blob".to_string()),
            onclick: poke,

            svg {
                view_box: "0 0 100 100",
                width: "100%",
                height: "100%",
                class: "{anim} origin-bottom",

                // NOTE: the palette custom properties are deliberately NOT
                // redeclared here. Writing `--jelly-body: var(--jelly-body, …)`
                // on this element looks like "default it if unset" and is
                // actually a **self-reference**: a custom property whose value
                // mentions itself is a cycle, invalid at computed-value time, so
                // every `fill` below resolved to nothing and painted black. The
                // blob rendered as a featureless dark circle while the DOM,
                // classes and animations all looked perfectly correct.
                //
                // Defaults belong at the use site, where `var(x, fallback)` does
                // what it appears to. An ancestor setting `--jelly-*` still wins,
                // which is the whole point of the palette being properties.

                // Body. A blobby path rather than a circle, so the idle wobble
                // has something asymmetric to deform.
                path {
                    d: "M50 8 C74 8 90 26 90 50 C90 76 74 92 50 92 C26 92 10 76 10 50 C10 26 26 8 50 8 Z",
                    fill: "var(--jelly-body, #a78bfa)",
                }

                // Cheeks.
                ellipse { cx: "27", cy: "62", rx: "8", ry: "5", fill: "var(--jelly-cheek, #f0abfc)", opacity: "0.55" }
                ellipse { cx: "73", cy: "62", rx: "8", ry: "5", fill: "var(--jelly-cheek, #f0abfc)", opacity: "0.55" }

                // Brows. Mirrored tilt, so angry converges and sad diverges.
                g {
                    stroke: "var(--jelly-eye, #1e1b4b)",
                    stroke_width: "2.5",
                    stroke_linecap: "round",
                    opacity: if brow == 0.0 { "0" } else { "0.9" },
                    line {
                        x1: "30", y1: "32", x2: "42", y2: "32",
                        transform: "rotate({brow} 36 32)",
                    }
                    line {
                        x1: "58", y1: "32", x2: "70", y2: "32",
                        transform: "rotate({-brow} 64 32)",
                    }
                }

                // Eyes. Closed moods draw arcs instead of pupils.
                if closed {
                    g {
                        stroke: "var(--jelly-eye, #1e1b4b)",
                        stroke_width: "3",
                        stroke_linecap: "round",
                        fill: "none",
                        path { d: "M28 45 Q36 39 44 45" }
                        path { d: "M56 45 Q64 39 72 45" }
                    }
                } else {
                    g {
                        circle { cx: "36", cy: "44", r: "7", fill: "white" }
                        circle { cx: "64", cy: "44", r: "7", fill: "white" }
                        circle {
                            cx: "{36.0 + px}",
                            cy: "{44.0 + py}",
                            r: if mood == Mood::Hmm { "2.4" } else { "3.4" },
                            fill: "var(--jelly-eye, #1e1b4b)",
                        }
                        circle {
                            cx: "{64.0 + px}",
                            cy: "{44.0 + py}",
                            r: "3.4",
                            fill: "var(--jelly-eye, #1e1b4b)",
                        }
                    }
                }

                // Mouth.
                path {
                    d: "M40 62 Q50 {62.0 + curve} 60 62",
                    stroke: "var(--jelly-eye, #1e1b4b)",
                    stroke_width: "3",
                    stroke_linecap: "round",
                    fill: "none",
                }
            }
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Hologram
// ═════════════════════════════════════════════════════════════════════════════

/// The foil pattern under the light.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Foil {
    /// Rays from a point. The classic.
    #[default]
    Sunburst,
    /// A swirl, for the cosmic ones.
    Cosmos,
    /// Vintage vertical streaks.
    Linear,
    /// Full spectrum.
    Rainbow,
}

impl Foil {
    fn background(self, angle: f32) -> String {
        match self {
            Foil::Sunburst => format!(
                "repeating-conic-gradient(from {angle}deg at 50% 50%, \
                 rgba(255,255,255,0.55) 0deg 4deg, transparent 4deg 12deg)"
            ),
            Foil::Cosmos => format!(
                "conic-gradient(from {angle}deg at 40% 40%, #f0abfc, #67e8f9, #fde68a, #f0abfc)"
            ),
            Foil::Linear => format!(
                "repeating-linear-gradient({angle}deg, rgba(255,255,255,0.5) 0 2px, \
                 transparent 2px 9px)"
            ),
            Foil::Rainbow => {
                format!("linear-gradient({angle}deg, #f87171, #fbbf24, #34d399, #60a5fa, #c084fc)")
            }
        }
    }
}

/// A foil card that catches the light.
///
/// Tilts toward the pointer, the sheen tracks it, and the border glows where the
/// light is. With no pointer it keeps sweeping on its own, because a foil card
/// that is inert until touched looks like a bug.
///
/// **Honest about the gap:** upstream drives an SVG `feDiffuseLighting` bump map
/// off the card art and re-lights it per frame, plus a canvas particle system.
/// This is gradients and transforms — convincing, and not the same thing. The
/// relief and the particles are absent, not approximated.
#[component]
pub fn Hologram(
    children: Element,
    #[props(default)] foil: Foil,
    /// 0.0–1.0. How strongly the foil reads.
    #[props(default = 0.35)]
    intensity: f32,
    /// Maximum tilt in degrees at the edges.
    #[props(default = 10.0)]
    tilt: f32,
) -> Element {
    // Pointer position as a fraction of the card, centre-relative: -0.5 to 0.5.
    let mut point = use_signal(|| (0.0_f32, 0.0_f32));
    let mut active = use_signal(|| false);

    let (nx, ny) = point();
    let rot_y = nx * tilt;
    let rot_x = -ny * tilt;
    // Light sits where the pointer is, in percent.
    let (lx, ly) = ((nx + 0.5) * 100.0, (ny + 0.5) * 100.0);
    let alpha = intensity.clamp(0.0, 1.0);
    let angle = 120.0 + nx * 90.0;

    let on_move = move |event: PointerEvent| {
        let coords = event.data().element_coordinates();
        // Dioxus gives element-relative pixels; the element is not measured
        // here, so normalise against a nominal box and clamp. Good enough for a
        // decorative tilt, and it avoids reaching for `web-sys` to measure.
        let x = ((coords.x as f32 / 320.0) - 0.5).clamp(-0.5, 0.5);
        let y = ((coords.y as f32 / 440.0) - 0.5).clamp(-0.5, 0.5);
        point.set((x, y));
        active.set(true);
    };

    let sheen_anim = if active() { "" } else { "animate-sheen" };

    rsx! {
        div {
            class: "relative isolate select-none rounded-2xl p-[2px] transition-transform \
                    duration-200 ease-out motion-tilt will-change-transform",
            style: "transform: perspective(900px) rotateX({rot_x}deg) rotateY({rot_y}deg) \
                    translateZ(0); \
                    background: radial-gradient(60% 60% at {lx}% {ly}%, \
                    rgba(255,255,255,0.95), rgba(148,163,184,0.25));",
            onpointermove: on_move,
            onpointerleave: move |_| {
                active.set(false);
                point.set((0.0, 0.0));
            },

            // The card face.
            //
            // ORDER MATTERS, and getting it wrong is not subtle. The light
            // layers are painted FIRST and the content LAST.
            //
            // `mix-blend-color-dodge` blends with its backdrop — everything
            // already painted beneath it. With the content painted first, that
            // backdrop included the text, and dodge brightens: near-white body
            // copy on a dark card washed out to unreadable at exactly the
            // intensities that make the foil look good. `z-10` on the content
            // did not save it, because the problem was never paint order, it was
            // what the blend had to chew on.
            //
            // Painting the light first means it only ever blends the card
            // surface, and the text sits cleanly on top of the result.
            div { class: "relative isolate overflow-hidden rounded-[14px] bg-slate-900 p-5 text-slate-100",

                // Foil, gated by a radial so it is brightest under the light.
                div {
                    class: "pointer-events-none absolute inset-0 mix-blend-color-dodge",
                    style: "background: {foil.background(angle)}; opacity: {alpha}; \
                            mask-image: radial-gradient(55% 55% at {lx}% {ly}%, \
                            black, transparent);",
                }

                // Glare: a moving band. Animates by itself when nothing is
                // pointing at it.
                div {
                    class: "pointer-events-none absolute inset-0 {sheen_anim}",
                    style: "background: linear-gradient(105deg, transparent 40%, \
                            rgba(255,255,255,0.35) 50%, transparent 60%); \
                            background-size: 220% 100%;",
                }

                // An OPAQUE plate under the content, and it is an accessibility
                // fix rather than a styling choice.
                //
                // Text sitting directly on the foil has no computable contrast
                // ratio — a checker walks up for a background colour, finds a
                // gradient overlapping the text, and correctly refuses to
                // certify it. "Unverifiable" is not "passing", and it was
                // genuinely marginal: `mix-blend-color-dodge` at the intensity
                // that makes foil look good will brighten a dark backdrop by an
                // unpredictable amount.
                //
                // Solid `bg-slate-900` makes the ratio computable AND fixed, so
                // the same card passes at any foil intensity. It is also how a
                // real foil card works: the art shimmers, the text box does not.
                div { class: "relative z-10 rounded-lg bg-slate-900 p-3", {children} }
            }
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// PullCord
// ═════════════════════════════════════════════════════════════════════════════

/// A ceiling pull-cord.
///
/// Drag the knob down. It actuates **mid-pull**, the moment travel crosses
/// `actuation`, the way a real pull-chain detents — not on release. Let go and
/// it swings back and settles.
///
/// **Honest about the gap:** upstream simulates the rope, with gravity, damping
/// and stiffness you can tune, integrated per frame. This is a drag plus a
/// keyframed damped swing. It reads as a rope; it is not solving one. Doing it
/// properly needs a render loop, which this crate deliberately cannot have.
#[component]
pub fn PullCord(
    on_pull: EventHandler<()>,
    /// Travel in pixels at which it actuates.
    #[props(default = 26.0)]
    actuation: f32,
    /// How far it can be dragged before it stops following.
    #[props(default = 64.0)]
    travel: f32,
    #[props(default)] label: Option<String>,
) -> Element {
    let mut pull = use_signal(|| 0.0_f32);
    let mut dragging = use_signal(|| false);
    // Latched so one drag fires once, however far past the detent it goes.
    let mut fired = use_signal(|| false);
    let mut settling = use_signal(|| false);

    let start = move |_| {
        dragging.set(true);
        settling.set(false);
        fired.set(false);
    };

    let mut release = move || {
        if dragging() {
            dragging.set(false);
            pull.set(0.0);
            settling.set(true);
        }
    };

    let on_move = move |event: PointerEvent| {
        if !dragging() {
            return;
        }
        let y = (event.data().element_coordinates().y as f32).clamp(0.0, travel);
        pull.set(y);
        if y >= actuation && !fired() {
            fired.set(true);
            on_pull.call(());
        }
    };

    let offset = pull();
    // Swing is proportional to how hard it was pulled, so a gentle tug settles
    // gently.
    let swing = (offset / travel) * 12.0;
    let settle = if settling() {
        "animate-cord-settle"
    } else {
        ""
    };

    rsx! {
        div {
            class: "flex flex-col items-center",
            onpointermove: on_move,
            onpointerup: move |_| release(),
            onpointerleave: move |_| release(),

            div {
                class: "flex flex-col items-center origin-top {settle}",
                style: "--cord-pull: {offset}px; --cord-swing: {swing}deg; \
                        transform: translateY({offset}px);",

                // The cord.
                div { class: "w-[2px] h-24 bg-gradient-to-b from-slate-400 to-slate-600 rounded-full" }

                // The knob.
                button {
                    r#type: "button",
                    class: "h-6 w-6 -mt-[2px] cursor-grab rounded-full bg-slate-700 \
                            shadow-md active:cursor-grabbing touch-none \
                            focus:outline-none focus-visible:ring-2 focus-visible:ring-slate-900 \
                            focus-visible:ring-offset-2",
                    aria_label: label.unwrap_or_else(|| "Pull the cord".to_string()),
                    onpointerdown: start,
                    // Keyboard parity: the detent is the whole interaction, so
                    // it must be reachable without a pointer.
                    onclick: move |_| on_pull.call(()),
                }
            }
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Crumple / Vacuum — dismissal effects
// ═════════════════════════════════════════════════════════════════════════════

/// Wraps content that crumples up and disappears when `crumpled` flips.
///
/// The caller owns the flag, so the element can be removed from the tree after
/// the animation rather than lingering invisible — `on_done` is the hook.
#[component]
pub fn Crumple(
    children: Element,
    crumpled: bool,
    #[props(default)] on_done: Option<EventHandler<()>>,
) -> Element {
    let class = if crumpled {
        "animate-crumple pointer-events-none"
    } else {
        ""
    };

    rsx! {
        div {
            class: "{class}",
            onanimationend: move |_| {
                if crumpled && let Some(handler) = &on_done {
                    handler.call(());
                }
            },
            {children}
        }
    }
}

/// Wraps content that gets sucked toward a point and vanishes.
///
/// `target` is the offset to travel, in CSS length units — aim it at wherever
/// the nozzle, bin or tab actually is.
#[component]
pub fn Vacuum(
    children: Element,
    active: bool,
    #[props(default = ("0px".to_string(), "120px".to_string()))] target: (String, String),
    #[props(default)] on_done: Option<EventHandler<()>>,
) -> Element {
    let class = if active {
        "animate-vacuum pointer-events-none"
    } else {
        ""
    };
    let (tx, ty) = target;

    rsx! {
        div {
            class: "{class}",
            style: "--vacuum-x: {tx}; --vacuum-y: {ty};",
            onanimationend: move |_| {
                if active && let Some(handler) = &on_done {
                    handler.call(());
                }
            },
            {children}
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Fur
// ═════════════════════════════════════════════════════════════════════════════

/// A furry surface. Ruffles when hovered.
///
/// Layered `repeating-linear-gradient`s rather than an image: it costs no
/// request, scales to any size, and recolours from one custom property.
#[component]
pub fn Fur(
    children: Element,
    /// Base colour, any CSS colour.
    #[props(default = "#78716c".to_string())]
    tint: String,
    #[props(default = "rounded-xl".to_string())] rounding: String,
) -> Element {
    rsx! {
        div {
            class: "fur-surface {rounding} p-5 shadow-inner",
            style: "--fur-tint: {tint};",
            {children}
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// GradientBuilder
// ═════════════════════════════════════════════════════════════════════════════

/// An interactive gradient editor that emits the CSS.
///
/// This one is reproduced in full, because it is pure state — there is no
/// simulation to approximate, so nothing is lost in translation.
#[component]
pub fn GradientBuilder(
    #[props(default = "#a78bfa".to_string())] start: String,
    #[props(default = "#67e8f9".to_string())] end: String,
    #[props(default = 135)] angle: i32,
    /// Fires whenever the CSS changes, with the full `linear-gradient(...)`.
    #[props(default)]
    on_change: Option<EventHandler<String>>,
) -> Element {
    let mut from = use_signal(|| start);
    let mut to = use_signal(|| end);
    let mut deg = use_signal(|| angle);

    let css = format!("linear-gradient({}deg, {}, {})", deg(), from(), to());

    // Report on every render where the value changed, so a caller can bind it
    // to a preview without polling.
    use_effect({
        let css = css.clone();
        move || {
            if let Some(handler) = &on_change {
                handler.call(css.clone());
            }
        }
    });

    rsx! {
        div { class: "space-y-3",
            div {
                class: "h-32 w-full rounded-xl border border-slate-200",
                style: "background: {css};",
            }
            div { class: "flex flex-wrap items-center gap-4",
                label { class: "flex items-center gap-2 text-sm text-slate-700",
                    "From"
                    input {
                        r#type: "color",
                        class: "h-8 w-12 cursor-pointer rounded border border-slate-300",
                        value: "{from}",
                        oninput: move |event| from.set(event.value()),
                    }
                }
                label { class: "flex items-center gap-2 text-sm text-slate-700",
                    "To"
                    input {
                        r#type: "color",
                        class: "h-8 w-12 cursor-pointer rounded border border-slate-300",
                        value: "{to}",
                        oninput: move |event| to.set(event.value()),
                    }
                }
                label { class: "flex flex-1 items-center gap-2 text-sm text-slate-700",
                    "Angle"
                    input {
                        r#type: "range",
                        class: "flex-1",
                        min: "0",
                        max: "360",
                        value: "{deg}",
                        oninput: move |event| {
                            if let Ok(v) = event.value().parse::<i32>() {
                                deg.set(v);
                            }
                        },
                    }
                    span { class: "w-12 tabular-nums text-right", "{deg}°" }
                }
            }
            code {
                class: "block overflow-x-auto rounded-lg bg-slate-900 px-3 py-2 text-xs text-slate-100",
                "background: {css};"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The mood table is the whole expressive range of the blob; a mood that
    /// renders identically to another is a mood that does not exist.
    #[test]
    fn every_mood_is_visually_distinct() {
        let moods = [
            Mood::Neutral,
            Mood::Happy,
            Mood::Sad,
            Mood::Angry,
            Mood::Hmm,
            Mood::SideEye,
            Mood::Password,
        ];
        let mut seen = Vec::new();
        for mood in moods {
            let shape = (
                mood.eyes_closed(),
                mood.brow_tilt().to_bits(),
                mood.mouth_curve().to_bits(),
            );
            assert!(
                !seen.contains(&shape),
                "{mood:?} renders identically to an earlier mood"
            );
            seen.push(shape);
        }
    }

    /// A password field must not be readable over the blob's shoulder. This is
    /// the one mood with a behavioural contract rather than a cosmetic one.
    #[test]
    fn the_password_mood_shuts_its_eyes() {
        assert!(Mood::Password.eyes_closed());
    }

    /// Happy bows up, sad bows down. Getting the sign backwards is the easiest
    /// possible mistake here and the least visible in code review.
    #[test]
    fn the_mouth_curves_the_right_way() {
        assert!(Mood::Happy.mouth_curve() < 0.0, "happy should bow upward");
        assert!(Mood::Sad.mouth_curve() > 0.0, "sad should bow downward");
    }

    /// Angry converges the brows, sad diverges them — mirrored by construction
    /// in the SVG, so only the sign matters here.
    #[test]
    fn brows_tilt_opposite_ways_for_angry_and_sad() {
        assert!(Mood::Angry.brow_tilt() < 0.0);
        assert!(Mood::Sad.brow_tilt() > 0.0);
        assert_eq!(Mood::Neutral.brow_tilt(), 0.0, "neutral draws no brows");
    }

    /// Each foil has to produce a different background or the choice is
    /// decorative in the worst sense.
    #[test]
    fn every_foil_paints_differently() {
        let foils = [Foil::Sunburst, Foil::Cosmos, Foil::Linear, Foil::Rainbow];
        let mut seen = Vec::new();
        for foil in foils {
            let css = foil.background(120.0);
            assert!(!seen.contains(&css), "{foil:?} duplicates another foil");
            seen.push(css);
        }
    }

    /// The angle has to reach the CSS, or the card will not track the pointer.
    #[test]
    fn the_foil_angle_reaches_the_css() {
        assert!(Foil::Sunburst.background(42.0).contains("42"));
        assert!(Foil::Rainbow.background(42.0).contains("42"));
    }
}
