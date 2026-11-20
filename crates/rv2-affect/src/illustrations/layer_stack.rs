//! An isometric stack of five layers next to a list of five entries. The chosen entry lifts its
//! layer out of the stack in blue, with a soft glow underneath, and the others settle back to
//! glassy grey. It steps through the entries on its own, pauses while hovered or focused, and
//! stops for good once the viewer picks an entry. Laid out on a 600 × 400 grid.
//!
//! Port losses: the auto-step timer is a `spawn` loop over `sleep`, so it stops for good once
//! the viewer picks an entry, and reduced motion is read once on mount through `document::eval`.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id, prefers_motion_script, sleep};

const CSS: Asset = asset!("/assets/layer-stack.css");

/// Layer geometry on the 600 × 400 grid.
const SIZE: f64 = 104.0;
const STEP: f64 = 26.0;
const GAP: f64 = 34.0;
const ABOVE: f64 = 24.0;
const THICK: f64 = 9.0;
const CX: f64 = 168.0;
const TOP: f64 = 109.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LayerIcon {
    #[default]
    Bolt,
    Database,
    Plug,
    Spark,
    Pulse,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerStackItem {
    /// Main line of the entry.
    pub title: String,
    /// Short muted hint on the right.
    pub hint: Option<String>,
    /// Icon drawn onto the layer.
    pub icon: LayerIcon,
}

impl LayerStackItem {
    pub fn new(title: impl Into<String>, hint: Option<&str>, icon: LayerIcon) -> Self {
        Self {
            title: title.into(),
            hint: hint.map(str::to_string),
            icon,
        }
    }
}

fn default_items() -> Vec<LayerStackItem> {
    vec![
        LayerStackItem::new("Triggered by events", Some("Webhooks"), LayerIcon::Bolt),
        LayerStackItem::new(
            "Grounded in your data",
            Some("Context"),
            LayerIcon::Database,
        ),
        LayerStackItem::new("Connected to your apps", Some("APIs"), LayerIcon::Plug),
        LayerStackItem::new("Works with any model", Some("Routing"), LayerIcon::Spark),
        LayerStackItem::new("Traced at every step", Some("Logs"), LayerIcon::Pulse),
    ]
}

/// Resting y of layer `i` in the stack. Gaps open below and above the chosen layer; the bottom
/// layer has nothing below it, so when it is chosen it stays put and only the layers above rise.
pub fn layer_y(i: usize, active: usize, last: usize) -> f64 {
    let base = TOP + i as f64 * STEP;
    if i < active {
        base - (GAP + ABOVE)
    } else if i == active && active < last {
        base - GAP
    } else {
        base
    }
}

fn icon_svg(kind: LayerIcon) -> Element {
    match kind {
        LayerIcon::Bolt => rsx! { path { d: "M13 3 5 13.5h6L10 21l8-10.5h-6L13 3Z" } },
        LayerIcon::Database => rsx! {
            ellipse { cx: "12", cy: "6", rx: "7", ry: "3" }
            path { d: "M5 6v12c0 1.7 3.1 3 7 3s7-1.3 7-3V6M5 12c0 1.7 3.1 3 7 3s7-1.3 7-3" }
        },
        LayerIcon::Plug => {
            rsx! { path { d: "M9 3v5M15 3v5M6.5 8h11v3a5.5 5.5 0 0 1-11 0V8ZM12 16.5V21" } }
        }
        LayerIcon::Spark => rsx! {
            path { d: "M12 3c.6 4.6 2.4 6.4 7 7-4.6.6-6.4 2.4-7 7-.6-4.6-2.4-6.4-7-7 4.6-.6 6.4-2.4 7-7ZM18.5 15.5c.2 1.5.8 2.1 2.3 2.3-1.5.2-2.1.8-2.3 2.3-.2-1.5-.8-2.1-2.3-2.3 1.5-.2 2.1-.8 2.3-2.3Z" }
        },
        LayerIcon::Pulse => rsx! { path { d: "M3 12h4l2.5-6 5 12 2.5-6h4" } },
    }
}

#[component]
pub fn LayerStack(
    #[props(default = default_items())] items: Vec<LayerStackItem>,
    #[props(default)] default_index: usize,
    #[props(default = 2600)] interval: u32,
    #[props(default)] on_change: Option<EventHandler<usize>>,
    #[props(default, into)] class: String,
) -> Element {
    let id = use_hook(|| format!("ui-lyr{}", next_id()));
    let list = if items.is_empty() {
        default_items()
    } else {
        items
    };
    let last = list.len() - 1;
    let mut active = use_signal(|| default_index.min(last));
    let mut paused = use_signal(|| false);
    let mut touched = use_signal(|| false);
    let mut motion = use_signal(|| false);

    use_effect(move || {
        let mut channel = document::eval(&format!("dioxus.send({});", prefers_motion_script()));
        spawn(async move {
            if let Ok(prefers) = channel.recv::<bool>().await {
                motion.set(prefers);
            }
        });
    });

    let count = list.len();
    use_effect(move || {
        if count < 2 {
            return;
        }
        spawn(async move {
            while !touched() {
                sleep(interval).await;
                if touched() || paused() || !motion() {
                    continue;
                }
                active.set((active() + 1) % count);
            }
        });
    });

    let mut pick = move |i: usize| {
        touched.set(true);
        active.set(i);
        if let Some(handler) = on_change {
            handler.call(i);
        }
    };

    let selected = active();
    let class = class_list("ui-lyr", &class);
    let pill_offset = format!("translateY(calc({selected} * var(--ui-lyr-row)))");
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class,
            onmouseenter: move |_| paused.set(true),
            onmouseleave: move |_| paused.set(false),
            onfocus: move |_| paused.set(true),
            onblur: move |_| paused.set(false),
            svg { class: "ui-lyr-stack", "viewBox": "0 0 600 400", "aria-hidden": "true",
                defs {
                    filter { id: "{id}-glow", x: "-60%", y: "-120%", width: "220%", height: "340%",
                        feGaussianBlur { "stdDeviation": "14" }
                    }
                }
                for i in (0..count).rev() {
                    LayerShape {
                        key: "{i}",
                        id: id.clone(),
                        index: i,
                        active: selected,
                        last,
                        icon: list[i].icon,
                    }
                }
            }

            div { class: "ui-lyr-menu", role: "tablist", "aria-label": "Layers", "aria-orientation": "vertical",
                span { class: "ui-lyr-pill", "aria-hidden": "true", style: "transform: {pill_offset}" }
                for (i, item) in list.iter().enumerate() {
                    button {
                        key: "{i}",
                        id: "{id}-row-{i}",
                        r#type: "button",
                        role: "tab",
                        "aria-selected": "{i == selected}",
                        tabindex: if i == selected { "0" } else { "-1" },
                        class: "ui-lyr-row",
                        onclick: move |_| pick(i),
                        onkeydown: {
                            let id = id.clone();
                            move |event| {
                                let next = match event.key().to_string().as_str() {
                                    "ArrowDown" => i + 1,
                                    "ArrowUp" => i + count - 1,
                                    _ => return,
                                };
                                event.prevent_default();
                                let target = next % count;
                                pick(target);
                                let row_id = format!("{id}-row-{target}");
                                document::eval(&format!("document.getElementById({row_id:?})?.focus();"));
                            }
                        },
                        span { class: "ui-lyr-title", "{item.title}" }
                        if let Some(hint) = item.hint.clone() {
                            span { class: "ui-lyr-hint", "{hint}" }
                        }
                    }
                }
            }
        }
    }
}

/// One layer of the isometric stack: glow, thickness, top face and icon.
#[component]
fn LayerShape(id: String, index: usize, active: usize, last: usize, icon: LayerIcon) -> Element {
    let y = layer_y(index, active, last);
    let style = format!("transform: translate({CX}px, {y}px)");
    let glow_cy = SIZE / 2.0 + THICK + 18.0;
    let icon_offset = SIZE / 2.0 - 18.0;
    let glow = format!("url(#{id}-glow)");
    let is_active = index == active;
    let is_above = index < active;
    rsx! {
        g { class: "ui-lyr-layer", style, "data-active": is_active.then_some(""), "data-above": is_above.then_some(""),
            ellipse { class: "ui-lyr-glow", cx: "0", cy: "{glow_cy}", rx: "70", ry: "24", filter: "{glow}" }
            g { transform: "translate(0 {THICK}) matrix(1 0.5 -1 0.5 0 0)",
                rect { class: "ui-lyr-side", width: "{SIZE}", height: "{SIZE}", rx: "9" }
                rect { class: "ui-lyr-side-on", width: "{SIZE}", height: "{SIZE}", rx: "9" }
            }
            g { transform: "matrix(1 0.5 -1 0.5 0 0)",
                rect { class: "ui-lyr-face", width: "{SIZE}", height: "{SIZE}", rx: "9" }
                rect { class: "ui-lyr-face-on", width: "{SIZE}", height: "{SIZE}", rx: "9" }
                g { class: "ui-lyr-icon", transform: "translate({icon_offset} {icon_offset}) scale(1.5)",
                    {icon_svg(icon)}
                }
            }
        }
    }
}
