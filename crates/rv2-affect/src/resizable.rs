//! Resizable: panes side by side or stacked, with a handle between each pair that drags, or moves from the keyboard.
//! Sizes are percentages of the group. A collapsible pane snaps shut when dragged past half its minimum.
//!
//! Port losses: `ref` is not forwarded. Pane order comes from the DOM (read after mount and whenever a pane comes or
//! goes), so the first paint uses each pane's flex share before the sizes are applied. Pointer capture is set from
//! `document::eval`, since Dioxus has no pointer capture API.

use std::collections::HashMap;

use dioxus::prelude::*;

use crate::internal::{class_list, next_id, sleep};

const CSS: Asset = asset!("/assets/resizable.css");

/// Sizes closer than this are the same size.
pub const EPS: f64 = 0.01;

/// Reads the pane ids in on-screen order: the group's own panes, not those of a group inside one of them.
const PANE_ORDER: &str = r#"
const group = document.getElementById("__ROOT__");
if (!group) return [];
return [...group.children].filter((c) => c.hasAttribute("data-rsz-panel")).map((c) => c.getAttribute("data-rsz-panel"));
"#;

/// The id of the pane before a handle.
const PANE_BEFORE: &str = r#"
const handle = document.getElementById("__HANDLE__");
let before = handle && handle.previousElementSibling;
while (before && !before.hasAttribute("data-rsz-panel")) before = before.previousElementSibling;
return before ? before.getAttribute("data-rsz-panel") : null;
"#;

/// Captures the pointer for a drag and measures the panes in pixels, so a pointer move turns into a percentage.
const DRAG_START: &str = r#"
const handle = document.getElementById("__HANDLE__");
if (!handle) return null;
handle.setPointerCapture(__POINTER__);
handle.focus({ preventScroll: true });
const group = handle.parentElement;
const horizontal = __HORIZONTAL__;
const rect = group.getBoundingClientRect();
const scale = horizontal ? rect.width / (group.offsetWidth || 1) : rect.height / (group.offsetHeight || 1);
const panes = [...group.children].filter((c) => c.hasAttribute("data-rsz-panel"));
const px = panes.reduce((sum, c) => sum + (horizontal ? c.offsetWidth : c.offsetHeight), 0) * (scale || 1);
return { px: Math.max(1, px), rtl: horizontal && getComputedStyle(handle).direction === "rtl" };
"#;

/// Whether the handle's row runs right to left, for the arrow keys and the drag direction.
const RTL: &str = r#"
const handle = document.getElementById("__HANDLE__");
return !!handle && getComputedStyle(handle).direction === "rtl";
"#;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Direction {
    /// Panes side by side.
    #[default]
    Horizontal,
    /// Panes stacked.
    Vertical,
}

impl Direction {
    fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

/// The limits and starting size of one pane, all in percent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaneConfig {
    pub min: f64,
    pub max: f64,
    pub collapsible: bool,
    pub collapsed: f64,
    pub initial: Option<f64>,
}

/// The limits of a pane as given. `max` is held at or above `min`, `collapsed` at or below it.
pub fn pane_config(
    min_size: f64,
    max_size: f64,
    collapsible: bool,
    collapsed_size: f64,
    initial: Option<f64>,
) -> PaneConfig {
    let min = clamp(min_size, 0.0, 100.0);
    PaneConfig {
        min,
        max: clamp(max_size, min, 100.0),
        collapsible,
        collapsed: clamp(collapsed_size, 0.0, min),
        initial,
    }
}

/// The sizes of all panes, summing to 100. A pane with a size keeps it; a pane with a starting size takes that; the
/// rest split what is left evenly. A group with no sizes at all gets equal shares.
pub fn distribute(current: &[Option<f64>], initial: &[Option<f64>]) -> Vec<f64> {
    let known: Vec<f64> = current
        .iter()
        .zip(initial)
        .filter_map(|(c, i)| c.or(*i))
        .collect();
    let given: f64 = known.iter().sum();
    let unknown = current.len() - known.len();
    let share = if unknown > 0 {
        (100.0 - given).max(0.0) / unknown as f64
    } else {
        0.0
    };
    let next: Vec<f64> = current
        .iter()
        .zip(initial)
        .map(|(c, i)| c.or(*i).unwrap_or(share))
        .collect();
    let total: f64 = next.iter().sum();
    let total = if total == 0.0 { 1.0 } else { total };
    next.iter().map(|s| s * 100.0 / total).collect()
}

/// Moves the handle after pane `at` by `delta` percent (positive: toward the end). The pane the handle moves away
/// from grows up to its max; the panes on the other side give way in turn, each down to its min. The pane beside the
/// handle snaps shut when it is collapsible and pushed past half its minimum. A folded pane opens to at least its
/// minimum once it is pulled past half of it, or at once from the keyboard.
pub fn shift(start: &[f64], cfg: &[PaneConfig], at: usize, delta: f64, keyboard: bool) -> Vec<f64> {
    let mut next = start.to_vec();
    if delta.abs() < EPS || at + 1 >= start.len() {
        return next;
    }
    let grow = if delta > 0.0 { at } else { at + 1 };
    let order: Vec<usize> = if delta > 0.0 {
        (at + 1..start.len()).collect()
    } else {
        (0..=at).rev().collect()
    };
    let g = cfg[grow];
    let mut want = delta.abs();
    let opening = g.collapsible && start[grow] <= g.collapsed + EPS && g.min > g.collapsed;
    if opening {
        if !keyboard && want < (g.min - g.collapsed) / 2.0 {
            return next;
        }
        want = want.max(g.min - g.collapsed);
    }
    want = want.min(g.max - start[grow]);
    if want <= EPS {
        return next;
    }
    let mut taken = 0.0;
    for (k, &i) in order.iter().enumerate() {
        let c = cfg[i];
        let need = want - taken;
        if need <= EPS {
            break;
        }
        let room = (start[i] - c.min).max(0.0);
        let open = !(c.collapsible && start[i] <= c.collapsed + EPS);
        if k == 0
            && c.collapsible
            && open
            && need > room + (c.min - c.collapsed) / 2.0
            && start[grow] + (start[i] - c.collapsed) <= g.max + EPS
        {
            taken += start[i] - c.collapsed;
            next[i] = c.collapsed;
            break;
        }
        if !open {
            continue;
        }
        let take = room.min(need);
        next[i] = start[i] - take;
        taken += take;
    }
    if opening && taken < g.min - g.collapsed - EPS {
        return start.to_vec();
    }
    next[grow] = start[grow] + taken;
    next
}

/// The result of Enter on a handle: the new sizes, the pane that was folded (if any) and the size it had before.
#[derive(Clone, Debug, PartialEq)]
pub struct Toggle {
    pub sizes: Vec<f64>,
    pub side: usize,
    pub remember: Option<f64>,
}

/// Folds the collapsible pane beside handle `at`, or brings a folded one back to where it was. The pane before the
/// handle is tried first. `restore` gives the size a pane had before it was folded.
pub fn toggle(
    start: &[f64],
    cfg: &[PaneConfig],
    at: usize,
    restore: impl Fn(usize) -> Option<f64>,
) -> Option<Toggle> {
    let side = if cfg.get(at)?.collapsible {
        at
    } else if cfg.get(at + 1)?.collapsible {
        at + 1
    } else {
        return None;
    };
    let c = cfg[side];
    let sign = if side == at { 1.0 } else { -1.0 };
    if start[side] <= c.collapsed + EPS {
        let back = clamp(restore(side).or(c.initial).unwrap_or(c.min), c.min, c.max);
        let sizes = shift(start, cfg, at, (back - start[side]) * sign, true);
        return Some(Toggle {
            sizes,
            side,
            remember: None,
        });
    }
    let sizes = shift(start, cfg, at, (c.collapsed - start[side]) * sign, false);
    Some(Toggle {
        sizes,
        side,
        remember: Some(start[side]),
    })
}

fn clamp(v: f64, lo: f64, hi: f64) -> f64 {
    v.max(lo).min(hi)
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// What the group shares with its panes and handles.
#[derive(Clone, Copy)]
struct Group {
    dom: usize,
    direction: Direction,
    step: f64,
    order: Signal<Vec<String>>,
    sizes: Signal<HashMap<String, f64>>,
    configs: Signal<HashMap<String, PaneConfig>>,
    snapping: Signal<bool>,
    dragging: Signal<bool>,
    on_layout: Option<EventHandler<Vec<f64>>>,
}

impl Group {
    fn refresh_order(self) {
        let script = PANE_ORDER.replace("__ROOT__", &format!("ui-rsz-{}", self.dom));
        spawn(async move {
            let mut order = self.order;
            let ids = document::eval(&script)
                .await
                .ok()
                .and_then(|v| {
                    v.as_array().map(|items| {
                        items
                            .iter()
                            .filter_map(|x| x.as_str().map(str::to_string))
                            .collect()
                    })
                })
                .unwrap_or_default();
            order.set(ids);
        });
    }

    fn index_of(self, id: Option<&str>) -> Option<usize> {
        let id = id?;
        self.order.peek().iter().position(|p| p == id)
    }

    /// Applies `change` around the handle after pane `before`. The change gets the sizes, the limits and the index of
    /// that pane, and returns the new sizes or `None` to leave everything as it is.
    fn resize(
        self,
        before: Option<&str>,
        change: impl FnOnce(&[f64], &[PaneConfig], usize) -> Option<Vec<f64>>,
        animate: bool,
    ) {
        let Some(at) = self.index_of(before) else {
            return;
        };
        let ids = self.order.peek().clone();
        if at + 1 >= ids.len() {
            return;
        }
        let sizes = self.sizes.peek().clone();
        let configs = self.configs.peek().clone();
        let cur: Vec<f64> = ids
            .iter()
            .map(|id| sizes.get(id).copied().unwrap_or(0.0))
            .collect();
        let cfg: Vec<PaneConfig> = ids
            .iter()
            .map(|id| {
                configs
                    .get(id)
                    .copied()
                    .unwrap_or(pane_config(0.0, 100.0, false, 0.0, None))
            })
            .collect();
        let Some(next) = change(&cur, &cfg, at) else {
            return;
        };
        if next.iter().zip(&cur).all(|(a, b)| (a - b).abs() < EPS) {
            return;
        }
        if animate {
            let mut snapping = self.snapping;
            snapping.set(true);
            spawn(async move {
                sleep(240).await;
                snapping.set(false);
            });
        }
        let mut out = self.sizes;
        out.set(ids.iter().cloned().zip(next.iter().copied()).collect());
        if let Some(handler) = self.on_layout {
            handler.call(next.iter().map(|s| round2(*s)).collect());
        }
    }

    fn set_dragging(self, on: bool) {
        let mut dragging = self.dragging;
        dragging.set(on);
    }
}

/// The panes and the handles between them. Each pane's size is a share of the group, out of 100.
#[component]
pub fn ResizablePanelGroup(
    #[props(default)] direction: Direction,
    #[props(default)] on_layout: Option<EventHandler<Vec<f64>>>,
    #[props(default = 5.0)] keyboard_step: f64,
    #[props(default)] class: String,
    #[props(default)] style: String,
    children: Element,
    #[props(extends = GlobalAttributes, extends = div)] attributes: Vec<Attribute>,
) -> Element {
    let dom = use_hook(next_id);
    let order = use_signal(Vec::<String>::new);
    let sizes = use_signal(HashMap::<String, f64>::new);
    let configs = use_signal(HashMap::<String, PaneConfig>::new);
    let snapping = use_signal(|| false);
    let dragging = use_signal(|| false);
    let group = Group {
        dom,
        direction,
        step: keyboard_step,
        order,
        sizes,
        configs,
        snapping,
        dragging,
        on_layout,
    };
    use_context_provider(|| group);

    // A pane's size is worked out again when the panes or their limits change; a size the user set is kept.
    use_effect(move || {
        let ids = order();
        let cfgs = configs();
        if ids.is_empty() {
            return;
        }
        let mut sizes = sizes;
        let current = sizes.peek().clone();
        let cur: Vec<Option<f64>> = ids.iter().map(|id| current.get(id).copied()).collect();
        let init: Vec<Option<f64>> = ids
            .iter()
            .map(|id| cfgs.get(id).and_then(|c| c.initial))
            .collect();
        let next: HashMap<String, f64> = ids.iter().cloned().zip(distribute(&cur, &init)).collect();
        let same = next.len() == current.len()
            && next
                .iter()
                .all(|(id, s)| current.get(id).is_some_and(|c| (c - s).abs() < EPS));
        if !same {
            sizes.set(next);
        }
    });

    let class = class_list("ui-rsz", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            id: "ui-rsz-{dom}",
            class,
            "data-direction": direction.as_str(),
            "data-resizing": dragging().then_some(""),
            "data-snap": snapping().then_some(""),
            style: "{style}",
            ..attributes,
            {children}
        }
    }
}

/// A pane of a group. Its size is a share of the group: `default_size` before the user changes it.
#[component]
pub fn ResizablePanel(
    #[props(default)] default_size: Option<f64>,
    #[props(default)] min_size: f64,
    #[props(default = 100.0)] max_size: f64,
    #[props(default)] collapsible: bool,
    #[props(default)] collapsed_size: f64,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] style: String,
    children: Element,
    #[props(extends = GlobalAttributes, extends = div)] attributes: Vec<Attribute>,
) -> Element {
    let group = try_use_context::<Group>();
    let auto = use_hook(|| format!("ui-rsz-pane-{}", next_id()));
    let key = id.unwrap_or(auto);
    let cfg = pane_config(
        min_size,
        max_size,
        collapsible,
        collapsed_size,
        default_size,
    );

    let reg_key = key.clone();
    use_effect(use_reactive((&cfg,), move |(cfg,)| {
        if let Some(group) = group {
            let mut configs = group.configs;
            configs.write().insert(reg_key.clone(), cfg);
        }
    }));

    use_effect(move || {
        if let Some(group) = group {
            group.refresh_order();
        }
    });
    let drop_key = key.clone();
    use_drop(move || {
        if let Some(group) = group {
            let mut configs = group.configs;
            configs.write().remove(&drop_key);
            group.refresh_order();
        }
    });

    let size = group
        .and_then(|g| g.sizes.read().get(&key).copied())
        .or(default_size);
    let folded = cfg.collapsible && size.is_some_and(|s| s <= cfg.collapsed + EPS);
    // Folded all the way: out of the Tab order and away from screen readers too.
    let hidden = folded && size.is_some_and(|s| s <= EPS);
    let flex = format!("flex: {} 1 0px; {style}", size.unwrap_or(1.0));
    let class = class_list("ui-rsz__panel", &class);

    rsx! {
        div {
            id: "{key}",
            class,
            "data-rsz-panel": "{key}",
            "data-collapsed": folded.then_some(""),
            "inert": hidden.then_some(""),
            style: "{flex}",
            ..attributes,
            {children}
        }
    }
}

/// The line between two panes. Arrow keys move it, Home and End go to its limits, Enter folds a collapsible pane and
/// brings it back, and a double-click puts it back where it started.
#[component]
pub fn ResizableHandle(
    #[props(default)] with_handle: bool,
    #[props(default = "Resize".to_string(), into)] aria_label: String,
    #[props(default)] class: String,
    #[props(default)] onfocus: Option<EventHandler<FocusEvent>>,
    #[props(default)] onblur: Option<EventHandler<FocusEvent>>,
    #[props(default)] onkeydown: Option<EventHandler<KeyboardEvent>>,
    #[props(extends = GlobalAttributes, extends = div)] attributes: Vec<Attribute>,
) -> Element {
    let group = try_use_context::<Group>();
    let handle_id = use_hook(|| format!("ui-rsz-handle-{}", next_id()));
    let mut before = use_signal(|| None::<String>);
    let restore = use_signal(HashMap::<String, f64>::new);
    let mut active = use_signal(|| false);
    let mut drag = use_signal(|| None::<Drag>);
    let rtl = use_signal(|| false);

    let before_id = handle_id.clone();
    use_effect(move || {
        let Some(group) = group else { return };
        let _ = group.order.read();
        let script = PANE_BEFORE.replace("__HANDLE__", &before_id);
        spawn(async move {
            let id = document::eval(&script)
                .await
                .ok()
                .and_then(|v| v.as_str().map(str::to_string));
            before.set(id);
        });
    });

    let rtl_id = handle_id.clone();
    use_effect(move || {
        let Some(group) = group else { return };
        if group.direction != Direction::Horizontal {
            return;
        }
        let script = RTL.replace("__HANDLE__", &rtl_id);
        let mut rtl = rtl;
        spawn(async move {
            let is_rtl = document::eval(&script)
                .await
                .ok()
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            rtl.set(is_rtl);
        });
    });

    let Some(group) = group else { return rsx! {} };
    let horizontal = group.direction == Direction::Horizontal;
    let pane = before();
    let cfg = pane
        .as_deref()
        .and_then(|id| group.configs.read().get(id).copied());
    let now = pane
        .as_deref()
        .and_then(|id| group.sizes.read().get(id).copied());
    let value_min = cfg
        .map(|c| {
            let base = if c.collapsible { c.collapsed } else { c.min };
            base.round()
        })
        .unwrap_or(0.0);
    let value_max = cfg.map(|c| c.max.round()).unwrap_or(100.0);
    let orientation = if horizontal { "vertical" } else { "horizontal" };
    let class = class_list("ui-rsz__handle", &class);

    let down_id = handle_id.clone();
    let drag_start = move |event: PointerEvent| {
        if event.trigger_button() != Some(dioxus::html::input_data::MouseButton::Primary) {
            return;
        }
        event.prevent_default();
        let pos = if horizontal {
            event.client_coordinates().x
        } else {
            event.client_coordinates().y
        };
        let pointer = event.data().pointer_id();
        let script = DRAG_START
            .replace("__HANDLE__", &down_id)
            .replace("__POINTER__", &pointer.to_string())
            .replace("__HORIZONTAL__", if horizontal { "true" } else { "false" });
        let sizes = group
            .order
            .peek()
            .iter()
            .map(|id| group.sizes.peek().get(id).copied().unwrap_or(0.0))
            .collect();
        drag.set(Some(Drag {
            pos,
            px: 0.0,
            sizes,
            sign: 1.0,
        }));
        active.set(true);
        group.set_dragging(true);
        spawn(async move {
            let measured = document::eval(&script).await.ok();
            let px = measured
                .as_ref()
                .and_then(|v| v.get("px"))
                .and_then(|x| x.as_f64())
                .unwrap_or(0.0);
            let is_rtl = measured
                .as_ref()
                .and_then(|v| v.get("rtl"))
                .and_then(|x| x.as_bool())
                .unwrap_or(false);
            if let Some(d) = drag.write().as_mut() {
                d.px = px;
                d.sign = if is_rtl { -1.0 } else { 1.0 };
            }
        });
    };

    let drag_move = move |event: PointerEvent| {
        let Some(d) = drag() else { return };
        if d.px <= 0.0 {
            return;
        }
        let cur = if horizontal {
            event.client_coordinates().x
        } else {
            event.client_coordinates().y
        };
        let delta = (cur - d.pos) / d.px * 100.0 * d.sign;
        let sizes = d.sizes.clone();
        group.resize(
            pane_id(before).as_deref(),
            move |_, cfg, at| Some(shift(&sizes, cfg, at, delta, false)),
            false,
        );
    };

    let drag_end = move |_: PointerEvent| {
        if drag.peek().is_some() {
            drag.set(None);
            active.set(false);
            group.set_dragging(false);
        }
    };

    let reset = move |_: MouseEvent| {
        group.resize(
            pane_id(before).as_deref(),
            |sizes, cfg, at| {
                cfg[at]
                    .initial
                    .map(|initial| shift(sizes, cfg, at, initial - sizes[at], true))
            },
            true,
        );
    };

    let on_keys = move |event: KeyboardEvent| {
        let rtl = rtl();
        let step = group.step
            * if event.modifiers().contains(Modifiers::SHIFT) {
                2.0
            } else {
                1.0
            };
        let back = match (horizontal, rtl) {
            (true, false) => Key::ArrowLeft,
            (true, true) => Key::ArrowRight,
            (false, _) => Key::ArrowUp,
        };
        let fwd = match (horizontal, rtl) {
            (true, false) => Key::ArrowRight,
            (true, true) => Key::ArrowLeft,
            (false, _) => Key::ArrowDown,
        };
        let key = event.key();
        if key == back || key == fwd {
            event.prevent_default();
            let delta = if key == fwd { step } else { -step };
            group.resize(
                pane_id(before).as_deref(),
                move |sizes, cfg, at| Some(shift(sizes, cfg, at, delta, true)),
                true,
            );
        } else if key == Key::Home || key == Key::End {
            event.prevent_default();
            let to_start = key == Key::Home;
            group.resize(
                pane_id(before).as_deref(),
                move |sizes, cfg, at| {
                    let target = if to_start { cfg[at].min } else { cfg[at].max };
                    Some(shift(sizes, cfg, at, target - sizes[at], true))
                },
                true,
            );
        } else if key == Key::Enter {
            event.prevent_default();
            toggle_at(group, restore, pane_id(before).as_deref());
        }
    };

    rsx! {
        div {
            id: "{handle_id}",
            class,
            role: "separator",
            "tabindex": "0",
            "aria-orientation": orientation,
            aria_label: "{aria_label}",
            "aria-controls": pane.clone(),
            "aria-valuenow": now.map(|v| v.round().to_string()),
            "aria-valuemin": "{value_min}",
            "aria-valuemax": "{value_max}",
            "data-dragging": active().then_some(""),
            onpointerdown: drag_start,
            onpointermove: drag_move,
            onpointerup: drag_end,
            onpointercancel: drag_end,
            onlostpointercapture: drag_end,
            ondoubleclick: reset,
            onkeydown: move |event: KeyboardEvent| {
                on_keys(event.clone());
                if let Some(handler) = &onkeydown {
                    handler.call(event);
                }
            },
            onfocus: move |event| {
                if let Some(handler) = &onfocus {
                    handler.call(event);
                }
            },
            onblur: move |event| {
                if let Some(handler) = &onblur {
                    handler.call(event);
                }
            },
            ..attributes,
            if with_handle {
                span { class: "ui-rsz__grip", "aria-hidden": "true" }
            }
        }
    }
}

/// The handle's drag in progress: where the pointer was, how many pixels the panes span, and the sizes at the start.
#[derive(Clone, Debug, PartialEq)]
struct Drag {
    pos: f64,
    px: f64,
    sizes: Vec<f64>,
    sign: f64,
}

/// Enter on a handle: folds or opens the pane beside it, and remembers the size a folded pane had.
fn toggle_at(group: Group, mut restore: Signal<HashMap<String, f64>>, before: Option<&str>) {
    let ids = group.order.peek().clone();
    group.resize(
        before,
        move |sizes, cfg, at| {
            let t = toggle(sizes, cfg, at, |side| {
                restore.peek().get(&ids[side]).copied()
            })?;
            if let Some(size) = t.remember {
                restore.write().insert(ids[t.side].clone(), size);
            }
            Some(t.sizes)
        },
        true,
    );
}

/// The id of the pane a handle follows, as an owned value a handler can keep.
fn pane_id(signal: Signal<Option<String>>) -> Option<String> {
    signal.peek().clone()
}
