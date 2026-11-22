//! Agent context map: an agent at the centre, what it knows on the left and the tools it reaches on the right.
//!
//! Also the home of the lane's shared layout maths (`arc_positions`, `tree_positions`,
//! `bracket_centres`, `treemap`), reused by the tree, bracket, ring and territory diagrams.
//!
//! Design choice: marks are text glyphs (`*_mark: Option<String>`), not nodes, so the data stays comparable.
//! Design choice: no wall-clock default. `clock_ms` unset omits relative ages, keeping SSR deterministic
//! (`SystemTime::now` panics on `wasm32-unknown-unknown`).
//! Design choice: the memory and tool rings hold six slots each; overflow reads "+N more".
//! Design choice: switches report through callbacks and also set a local override, so the card dims at once
//! even when the caller has not yet fed the new data back.
//! Browser checks: pulse along connectors, card rise-in, drop-zone drag feedback, focus rings, Tab stops.

use dioxus::prelude::*;

use crate::internal::*;

const CSS: Asset = asset!("/assets/agent-context-map.css");

/// Memory slots on the left arc; overflow past this reads "+N more".
pub const MEMORY_SLOTS: usize = 6;
/// Tool slots on the right arc.
pub const TOOL_SLOTS: usize = 6;

/// Design-unit canvas the ring is drawn in; card positions are percentages of it.
pub const RING_W: f64 = 1000.0;
pub const RING_H: f64 = 720.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemKind {
    Memory,
    Tool,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct AgentNode {
    pub agent_id: String,
    pub agent_name: String,
    pub agent_subtitle: String,
    pub agent_mark: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct SourceArtifact {
    pub artifact_name: String,
    pub artifact_kind: Option<String>,
    pub artifact_mark: Option<String>,
    pub artifact_link: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Memory {
    pub memory_id: String,
    pub memory_text: String,
    pub memory_kind: Option<String>,
    pub memory_source_line: Option<String>,
    pub source_artifact: SourceArtifact,
    pub learned_via_tool_id: Option<String>,
    pub replaces_memory_id: Option<String>,
    pub is_outdated: bool,
    pub related_memory_ids: Vec<String>,
    pub learned_at: Option<f64>,
    pub use_count: u32,
    pub is_disabled: bool,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Tool {
    pub tool_id: String,
    pub tool_mark: Option<String>,
    pub tool_name: String,
    pub access_level: String,
    pub scope_text: String,
    pub last_activity_text: String,
    pub last_used_at: Option<f64>,
    pub call_count: u32,
    pub is_disabled: bool,
}

/// Points `n` items evenly along one half of an ellipse, top to bottom.
/// `left` picks the left arc (x <= cx), otherwise the right arc.
/// Slots stacked evenly down one side of an ellipse: `n` centres at equal vertical steps, each pushed out
/// by the ellipse's x at that height, so the column hugs the hub without crowding its top and bottom slots.
pub fn side_slots(n: usize, left: bool, cx: f64, cy: f64, rx: f64, ry: f64) -> Vec<(f64, f64)> {
    (0..n)
        .map(|i| {
            let u = (2.0 * i as f64 + 1.0) / n as f64 - 1.0;
            let dx = rx * (1.0 - u * u).sqrt();
            let x = if left { cx - dx } else { cx + dx };
            (x, cy + u * ry)
        })
        .collect()
}

pub fn arc_positions(n: usize, left: bool, cx: f64, cy: f64, rx: f64, ry: f64) -> Vec<(f64, f64)> {
    use std::f64::consts::PI;
    (0..n)
        .map(|i| {
            let t = (i as f64 + 0.5) / n as f64;
            let theta = if left {
                3.0 * PI / 2.0 - t * PI
            } else {
                -PI / 2.0 + t * PI
            };
            (cx + rx * theta.cos(), cy + ry * theta.sin())
        })
        .collect()
}

/// Lays a tree out top-down: leaves take consecutive x slots, parents sit over the mean of their children.
/// `children[i]` lists the child indices of node `i`. Returns one (x, y) per node; nodes unreachable from
/// `root` stay at the origin.
pub fn tree_positions(
    children: &[Vec<usize>],
    root: usize,
    leaf_gap: f64,
    level_gap: f64,
) -> Vec<(f64, f64)> {
    fn place(
        children: &[Vec<usize>],
        node: usize,
        depth: usize,
        leaf_gap: f64,
        level_gap: f64,
        next_leaf: &mut usize,
        pos: &mut [(f64, f64)],
    ) -> f64 {
        let x = if children[node].is_empty() || depth > children.len() {
            let x = *next_leaf as f64 * leaf_gap;
            *next_leaf += 1;
            x
        } else {
            let xs: Vec<f64> = children[node]
                .iter()
                .map(|&c| place(children, c, depth + 1, leaf_gap, level_gap, next_leaf, pos))
                .collect();
            xs.iter().sum::<f64>() / xs.len() as f64
        };
        pos[node] = (x, depth as f64 * level_gap);
        x
    }
    let mut pos = vec![(0.0, 0.0); children.len()];
    if root < children.len() {
        let mut next_leaf = 0;
        place(
            children,
            root,
            0,
            leaf_gap,
            level_gap,
            &mut next_leaf,
            &mut pos,
        );
    }
    pos
}

/// Centres of every pairing in a knockout bracket, round by round. Each entrant owns one row of
/// `row_h`; round `r` pairing `i` sits midway between the two groups it joins, at column `r * col_w`.
/// Returns empty when `entrants` is not a power of two of at least two.
pub fn bracket_centres(entrants: usize, row_h: f64, col_w: f64) -> Vec<Vec<(f64, f64)>> {
    if entrants < 2 || !entrants.is_power_of_two() {
        return Vec::new();
    }
    let rounds = entrants.trailing_zeros() as usize;
    (0..rounds)
        .map(|r| {
            let span = (1usize << r) as f64;
            (0..entrants >> (r + 1))
                .map(|i| (r as f64 * col_w, row_h * (2.0 * span * i as f64 + span)))
                .collect()
        })
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// Binary-split treemap: each recursion halves the weight along the longer side. Order-preserving,
/// so the same input order always yields the same rectangles. A non-positive total falls back to equal weights.
pub fn treemap(weights: &[f64], x: f64, y: f64, w: f64, h: f64) -> Vec<Rect> {
    let total: f64 = weights.iter().sum();
    let weights: Vec<f64> = if total > 0.0 {
        weights.to_vec()
    } else {
        vec![1.0; weights.len()]
    };
    let mut out = Vec::with_capacity(weights.len());
    split_rect(&weights, Rect { x, y, w, h }, &mut out);
    out
}

fn split_rect(weights: &[f64], rect: Rect, out: &mut Vec<Rect>) {
    match weights.len() {
        0 => {}
        1 => out.push(rect),
        n => {
            let total: f64 = weights.iter().sum();
            let mut acc = 0.0;
            let mut cut = 1;
            let mut best = f64::MAX;
            for k in 1..n {
                acc += weights[k - 1];
                let gap = (acc - total / 2.0).abs();
                if gap < best {
                    best = gap;
                    cut = k;
                }
            }
            let left: f64 = weights[..cut].iter().sum::<f64>() / total;
            let (a, b) = if rect.w >= rect.h {
                let aw = rect.w * left;
                (
                    Rect {
                        x: rect.x,
                        y: rect.y,
                        w: aw,
                        h: rect.h,
                    },
                    Rect {
                        x: rect.x + aw,
                        y: rect.y,
                        w: rect.w - aw,
                        h: rect.h,
                    },
                )
            } else {
                let ah = rect.h * left;
                (
                    Rect {
                        x: rect.x,
                        y: rect.y,
                        w: rect.w,
                        h: ah,
                    },
                    Rect {
                        x: rect.x,
                        y: rect.y + ah,
                        w: rect.w,
                        h: rect.h - ah,
                    },
                )
            };
            split_rect(&weights[..cut], a, out);
            split_rect(&weights[cut..], b, out);
        }
    }
}

/// Splits `total` items into a visible count and an overflow count for `slots` positions.
pub fn visible_slots(total: usize, slots: usize) -> (usize, usize) {
    let shown = total.min(slots);
    (shown, total - shown)
}

/// The "+N more" entry text.
pub fn overflow_label(n: usize) -> String {
    format!("+{n} more")
}

/// Column centres in percent of the ring width. Memories sit left of the agent, tools right of it.
const MEMORY_COLUMN_X: f64 = 17.0;
const TOOL_COLUMN_X: f64 = 83.0;

/// Vertical centre of slot `i` of `slots`, in percent of the ring height.
fn slot_y(i: usize, slots: usize) -> f64 {
    (i as f64 + 0.5) / slots as f64 * 100.0
}

fn slot_style(x: f64, y: f64) -> String {
    format!("--actx-x: {x:.2}%; --actx-y: {y:.2}%;")
}

/// The "+N more" entry sits at the foot of its column, under the last slot.
fn more_entry_style(x: f64) -> String {
    slot_style(x, 100.0)
}

/// Replacement bracket between two memory cards. Its width is set in CSS from the card width, so
/// the arms meet the cards' left edges at any container width.
fn bracket_style(x: f64, from_y: f64, to_y: f64) -> String {
    format!(
        "--actx-x: {x:.2}%; --actx-top: {from_y:.2}%; --actx-h: {:.2}%;",
        to_y - from_y
    )
}

/// `(replaced, replacement)` index pairs into `memories`, one per memory that names a replaced memory.
pub fn replacement_links(memories: &[Memory]) -> Vec<(usize, usize)> {
    memories
        .iter()
        .enumerate()
        .filter_map(|(new_i, m)| {
            let old = m.replaces_memory_id.as_deref()?;
            let old_i = memories.iter().position(|o| o.memory_id == old)?;
            Some((old_i, new_i))
        })
        .collect()
}

/// Ids of memories that a newer memory replaces, plus memories flagged outdated themselves.
pub fn outdated_ids(memories: &[Memory]) -> Vec<String> {
    let replaced: Vec<&str> = memories
        .iter()
        .filter_map(|m| m.replaces_memory_id.as_deref())
        .collect();
    memories
        .iter()
        .filter(|m| m.is_outdated || replaced.contains(&m.memory_id.as_str()))
        .map(|m| m.memory_id.clone())
        .collect()
}

/// A switch set locally wins over the item's own flag.
pub fn effective_disabled(item_disabled: bool, local: Option<bool>) -> bool {
    match local {
        Some(enabled) => !enabled,
        None => item_disabled,
    }
}

/// Recalled ids that are still switched on, in their given order.
pub fn recalled_visible(recalled: &[String], disabled_ids: &[String]) -> Vec<String> {
    recalled
        .iter()
        .filter(|id| !disabled_ids.contains(id))
        .cloned()
        .collect()
}

/// State words spelled out on a memory card, never shade alone.
pub fn memory_state_words(outdated: bool, disabled: bool, recalled: bool) -> Vec<&'static str> {
    let mut words = Vec::new();
    if outdated {
        words.push("outdated");
    }
    if disabled {
        words.push("switched off");
    }
    if recalled && !disabled {
        words.push("recalled");
    }
    words
}

/// State words spelled out on a tool card.
pub fn tool_state_words(in_use: bool, disabled: bool) -> Vec<&'static str> {
    let mut words = Vec::new();
    if in_use && !disabled {
        words.push("in use");
    }
    if disabled {
        words.push("switched off");
    }
    words
}

/// "just now", "12m ago", "3h ago", "2d ago". Empty when either time is unknown.
pub fn relative_age(clock_ms: Option<f64>, then_ms: Option<f64>) -> String {
    let (Some(now), Some(then)) = (clock_ms, then_ms) else {
        return String::new();
    };
    let secs = ((now - then) / 1000.0).max(0.0) as u64;
    match secs {
        0..=59 => "just now".to_string(),
        60..=3599 => format!("{}m ago", secs / 60),
        3600..=86_399 => format!("{}h ago", secs / 3600),
        _ => format!("{}d ago", secs / 86_400),
    }
}

/// Short month and day for a Unix-epoch millisecond value, in UTC.
pub fn short_date(ms: f64) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    if !ms.is_finite() {
        return "–".to_string();
    }
    let days = (ms / 86_400_000.0).floor() as i64;
    let (_, month, day) = civil_from_days(days);
    format!("{} {day}", MONTHS[(month - 1) as usize])
}

/// Days since 1970-01-01 to (year, month, day). Howard Hinnant's civil-calendar algorithm.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

/// Script that moves focus to an element by its DOM id.
pub fn focus_script(id: &str) -> String {
    format!("document.getElementById({id:?})?.focus();")
}

/// Moves focus into a dialog panel when it mounts; the panel is `tabindex="-1"`.
pub(crate) fn focus_on_mount(event: MountedEvent) {
    spawn(async move {
        let _ = event.set_focus(true).await;
    });
}

/// Next index for an arrow key on a list of `len` items; `None` when the list is empty or the key is not a list key.
pub fn list_step(len: usize, current: usize, key: &Key) -> Option<usize> {
    if len == 0 {
        return None;
    }
    match key {
        Key::ArrowDown => Some((current + 1).min(len - 1)),
        Key::ArrowUp => Some(current.saturating_sub(1)),
        Key::Home => Some(0),
        Key::End => Some(len - 1),
        _ => None,
    }
}

#[derive(Clone, PartialEq, Debug)]
enum Selection {
    Memory { id: String, dom_id: String },
    Tool { id: String, dom_id: String },
    Add,
}

/// Agent with memories on the left and tools on the right. Activating an item opens its details;
/// activating the agent opens Add. Escape closes whichever is open and returns focus to its card.
#[component]
pub fn AgentContextMap(
    agent_node: AgentNode,
    #[props(default)] memory_items: Vec<Memory>,
    #[props(default)] tool_items: Vec<Tool>,
    #[props(default)] active_tool_ids: Vec<String>,
    #[props(default)] recalled_memory_ids: Vec<String>,
    #[props(default)] available_tools: Vec<Tool>,
    #[props(default)] on_switch_change: Option<EventHandler<(ItemKind, String, bool)>>,
    #[props(default)] on_forget: Option<EventHandler<Memory>>,
    #[props(default)] on_add_tool: Option<EventHandler<Tool>>,
    #[props(default)] on_add_files: Option<EventHandler<Vec<String>>>,
    #[props(default)] on_select_memory: Option<EventHandler<Option<Memory>>>,
    #[props(default = "Context".to_string(), into)] heading_text: String,
    #[props(default)] clock_ms: Option<f64>,
    #[props(default)] format_date: Option<Callback<f64, String>>,
    #[props(default)] extra_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-actx-{}", next_id()));
    let mut selected = use_signal(|| None::<Selection>);
    let local_switch = use_signal(Vec::<(String, bool)>::new);

    let switch_of = |key: String| {
        local_switch()
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, on)| *on)
    };
    let memories_view: Vec<Memory> = memory_items
        .iter()
        .map(|m| {
            let mut m = m.clone();
            m.is_disabled =
                effective_disabled(m.is_disabled, switch_of(format!("m:{}", m.memory_id)));
            m
        })
        .collect();
    let tools_view: Vec<Tool> = tool_items
        .iter()
        .map(|t| {
            let mut t = t.clone();
            t.is_disabled =
                effective_disabled(t.is_disabled, switch_of(format!("t:{}", t.tool_id)));
            t
        })
        .collect();

    let disabled_ids: Vec<String> = memories_view
        .iter()
        .filter(|m| m.is_disabled)
        .map(|m| m.memory_id.clone())
        .collect();
    let recalled = recalled_visible(&recalled_memory_ids, &disabled_ids);
    let outdated = outdated_ids(&memory_items);
    let (mem_shown, mem_over) = visible_slots(memories_view.len(), MEMORY_SLOTS);
    let (tool_shown, tool_over) = visible_slots(tools_view.len(), TOOL_SLOTS);
    let replacements: Vec<(usize, usize)> = replacement_links(&memory_items)
        .into_iter()
        .filter(|(a, b)| *a < mem_shown && *b < mem_shown)
        .collect();

    let active_names: Vec<String> = tools_view
        .iter()
        .filter(|t| active_tool_ids.contains(&t.tool_id) && !t.is_disabled)
        .map(|t| t.tool_name.clone())
        .collect();
    let announce = if active_names.is_empty() {
        String::new()
    } else {
        format!("Using {}", active_names.join(", "))
    };
    let subtitle = if announce.is_empty() {
        agent_node.agent_subtitle.clone()
    } else {
        announce.clone()
    };
    let memory_count = memory_items.len();
    let tool_count = tool_items.len();
    let agent_name = agent_node.agent_name.clone();
    let agent_mark = agent_node
        .agent_mark
        .clone()
        .unwrap_or_else(|| agent_name.chars().take(1).collect());
    let class = class_list("kit-actx", &extra_class);
    let is_empty = memory_items.is_empty() && tool_items.is_empty();
    let open = selected();
    let agent_dom = format!("{base}-agent");

    let detail = match open.clone() {
        Some(Selection::Memory { id, dom_id }) => memories_view
            .iter()
            .find(|m| m.memory_id == id)
            .cloned()
            .map(|m| {
                let is_outdated = outdated.contains(&m.memory_id);
                rsx! {
                    MemoryDetail {
                        memory: m,
                        outdated: is_outdated,
                        on_switch_change,
                        on_forget,
                        on_close: move |_| {
                            selected.set(None);
                            document::eval(&focus_script(&dom_id));
                        },
                    }
                }
            }),
        Some(Selection::Tool { id, dom_id }) => tools_view
            .iter()
            .find(|t| t.tool_id == id)
            .cloned()
            .map(|t| {
                rsx! {
                    ToolDetail {
                        tool: t,
                        on_switch_change,
                        on_close: move |_| {
                            selected.set(None);
                            document::eval(&focus_script(&dom_id));
                        },
                    }
                }
            }),
        _ => None,
    };

    let mem_style = |i: usize| slot_style(MEMORY_COLUMN_X, slot_y(i, MEMORY_SLOTS));
    let tool_style = |i: usize| slot_style(TOOL_COLUMN_X, slot_y(i, TOOL_SLOTS));
    let more_mem_style = more_entry_style(MEMORY_COLUMN_X);
    let more_tool_style = more_entry_style(TOOL_COLUMN_X);
    let logos: Vec<(String, String, &'static str)> = tools_view
        .iter()
        .map(|t| {
            let state = if t.is_disabled {
                "off"
            } else if active_tool_ids.contains(&t.tool_id) {
                "active"
            } else {
                "idle"
            };
            let mark = t
                .tool_mark
                .clone()
                .unwrap_or_else(|| t.tool_name.chars().take(1).collect());
            (mark, t.tool_name.clone(), state)
        })
        .collect();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{class}", "aria-labelledby": "{base}-title", "data-empty": is_empty.then_some("true"),
            header { class: "kit-actx__head",
                h2 { id: "{base}-title", class: "kit-actx__title", "{heading_text}" }
                p { class: "kit-actx__counts", "{memory_count} memories · {tool_count} tools" }
            }
            div { class: "kit-actx__ring",
                svg { class: "kit-actx__wires", view_box: "0 0 100 100", preserve_aspect_ratio: "none", "aria-hidden": "true",
                    for i in 0..mem_shown {
                        line { key: "m{i}", class: "kit-actx__wire", "data-kind": "memory", x1: "50", y1: "50", x2: "{MEMORY_COLUMN_X}", y2: "{slot_y(i, MEMORY_SLOTS)}" }
                    }
                    for i in 0..tool_shown {
                        line {
                            key: "t{i}",
                            class: "kit-actx__wire",
                            "data-kind": "tool",
                            "data-state": if tools_view[i].is_disabled { "off" } else if active_tool_ids.contains(&tools_view[i].tool_id) { "active" } else { "idle" },
                            x1: "50", y1: "50", x2: "{TOOL_COLUMN_X}", y2: "{slot_y(i, TOOL_SLOTS)}",
                        }
                    }
                }
                for (from, to) in replacements.iter().copied() {
                    div { key: "r{from}-{to}", class: "kit-actx__bracket", "data-kind": "replaces",
                        style: "{bracket_style(MEMORY_COLUMN_X, slot_y(from, MEMORY_SLOTS), slot_y(to, MEMORY_SLOTS))}" }
                }
                ul { class: "kit-actx__side", "data-side": "memory", "aria-label": "Memories",
                    for (i, m) in memories_view.iter().take(mem_shown).enumerate() {
                        {
                            let style = mem_style(i);
                            let dom_id = format!("{base}-m-{i}");
                            let is_recalled = recalled.contains(&m.memory_id);
                            let words = memory_state_words(outdated.contains(&m.memory_id), m.is_disabled, is_recalled);
                            let state = if m.is_disabled { "off" } else if is_recalled { "recalled" } else { "idle" };
                            let words_text = if words.is_empty() { String::new() } else { format!(", {}", words.join(", ")) };
                            let label = format!("Memory: {}, from {}{words_text}", m.memory_text, m.source_artifact.artifact_name);
                            let when = relative_age(clock_ms, m.learned_at);
                            let learned = m.learned_at.map(|ms| match format_date {
                                Some(f) => f.call(ms),
                                None => short_date(ms),
                            });
                            let m_click = m.clone();
                            let id_click = m.memory_id.clone();
                            let dom_click = dom_id.clone();
                            let source_line = m.memory_source_line.clone();
                            let memory_text = m.memory_text.clone();
                            let artifact_name = m.source_artifact.artifact_name.clone();
                            let source_mark = m
                                .source_artifact
                                .artifact_mark
                                .clone()
                                .unwrap_or_else(|| artifact_name.chars().take(1).collect());
                            rsx! {
                                li { key: "{m.memory_id}", class: "kit-actx__slot", style: "{style}",
                                    button {
                                        id: "{dom_id}", r#type: "button", class: "kit-actx__card", "data-kind": "memory", "data-state": state,
                                        title: "{memory_text}", "aria-label": "{label}",
                                        onclick: move |_| {
                                            selected.set(Some(Selection::Memory { id: id_click.clone(), dom_id: dom_click.clone() }));
                                            if let Some(h) = on_select_memory {
                                                h.call(Some(m_click.clone()));
                                            }
                                        },
                                        span { class: "kit-actx__mark", "aria-hidden": "true", "{source_mark}" }
                                        span { class: "kit-actx__body",
                                            span { class: "kit-actx__text", "{memory_text}" }
                                            span { class: "kit-actx__source", "{artifact_name}" }
                                            if let Some(line) = source_line { span { class: "kit-actx__source", "{line}" } }
                                            if let Some(date) = learned { span { class: "kit-actx__meta", "{date} {when}" } }
                                            span { class: "kit-actx__foot",
                                                for word in words.iter() { span { class: "kit-actx__tag", "{word}" } }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if mem_over > 0 {
                        li { class: "kit-actx__slot kit-actx__slot--more", style: "{more_mem_style}",
                            span { class: "kit-actx__more", "{overflow_label(mem_over)}" }
                        }
                    }
                }
                div { class: "kit-actx__hub",
                button {
                    id: "{base}-agent", r#type: "button", class: "kit-actx__agent", "aria-haspopup": "dialog",
                    "aria-expanded": (open == Some(Selection::Add)).then_some("true"),
                    onclick: move |_| {
                        let next = if selected() == Some(Selection::Add) { None } else { Some(Selection::Add) };
                        selected.set(next);
                    },
                    span { class: "kit-actx__mark", "aria-hidden": "true", "{agent_mark}" }
                    span { class: "kit-actx__agent-name", "{agent_name}" }
                    span { class: "kit-actx__subtitle", "{subtitle}" }
                }
                ul { class: "kit-actx__logos", "aria-label": "Tools in reach",
                    for (mark, name, state) in logos.iter().cloned() {
                        li { key: "{name}", class: "kit-actx__logo", "data-state": state, title: "{name}",
                            span { class: "kit-actx__logo-mark", "aria-hidden": "true", "{mark}" }
                            span { class: "kit-actx__logo-name", "{name}" }
                        }
                    }
                }
                }
                ul { class: "kit-actx__side", "data-side": "tool", "aria-label": "Tools",
                    for (i, t) in tools_view.iter().take(tool_shown).enumerate() {
                        {
                            let style = tool_style(i);
                            let dom_id = format!("{base}-t-{i}");
                            let in_use = active_tool_ids.contains(&t.tool_id);
                            let words = tool_state_words(in_use, t.is_disabled);
                            let state = if t.is_disabled { "off" } else if in_use { "active" } else { "idle" };
                            let mark = t.tool_mark.clone().unwrap_or_else(|| t.tool_name.chars().take(1).collect());
                            let status = if words.is_empty() { "idle".to_string() } else { words.join(", ") };
                            let label = format!("Tool: {}, {}, {status}", t.tool_name, t.access_level);
                            let tool_id = t.tool_id.clone();
                            let tool_name = t.tool_name.clone();
                            let access = format!("{} · {}", t.access_level, t.scope_text);
                            let activity = if t.last_activity_text.is_empty() {
                                format!("{} calls", t.call_count)
                            } else {
                                format!("{} · {} calls", t.last_activity_text, t.call_count)
                            };
                            rsx! {
                                li { key: "{t.tool_id}", class: "kit-actx__slot", style: "{style}",
                                    button {
                                        id: "{dom_id}", r#type: "button", class: "kit-actx__card", "data-kind": "tool", "data-state": state,
                                        "aria-label": "{label}",
                                        onclick: move |_| selected.set(Some(Selection::Tool { id: tool_id.clone(), dom_id: dom_id.clone() })),
                                        span { class: "kit-actx__mark", "aria-hidden": "true", "{mark}" }
                                        span { class: "kit-actx__body",
                                            span { class: "kit-actx__text", "{tool_name}" }
                                            span { class: "kit-actx__source", "{access}" }
                                            span { class: "kit-actx__foot",
                                                span { class: "kit-actx__meta", "{activity}" }
                                                for word in words.iter() { span { class: "kit-actx__tag", "{word}" } }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if tool_over > 0 {
                        li { class: "kit-actx__slot kit-actx__slot--more", style: "{more_tool_style}",
                            span { class: "kit-actx__more", "{overflow_label(tool_over)}" }
                        }
                    }
                }
            }
            p { class: "kit-actx__live", role: "status", "aria-live": "polite", "{announce}" }
            if let Some(view) = detail {
                {view}
            }
            if open == Some(Selection::Add) {
                section {
                    class: "kit-actx__panel", role: "dialog", "aria-label": "Add to {agent_name}", tabindex: "-1",
                    onmounted: focus_on_mount,
                    onkeydown: move |event| {
                        if event.key() == Key::Escape {
                            event.prevent_default();
                            selected.set(None);
                            document::eval(&focus_script(&agent_dom));
                        }
                    },
                    h3 { class: "kit-actx__panel-title", "Add to {agent_name}" }
                    label { class: "kit-actx__drop",
                        span { "Drop files or choose them" }
                        input {
                            r#type: "file", multiple: true,
                            onchange: move |event| {
                                if let Some(h) = on_add_files {
                                    let names: Vec<String> = event.files().iter().map(|f| f.name()).collect();
                                    h.call(names);
                                }
                            },
                        }
                    }
                    ul { class: "kit-actx__choices", "aria-label": "Tools that can be connected",
                        for t in available_tools.iter() {
                            li { key: "{t.tool_id}",
                                button {
                                    r#type: "button", class: "kit-actx__choice",
                                    onclick: {
                                        let t = t.clone();
                                        move |_| {
                                            if let Some(h) = on_add_tool {
                                                h.call(t.clone());
                                            }
                                        }
                                    },
                                    "Connect {t.tool_name}"
                                }
                            }
                        }
                    }
                }
            }
            if is_empty {
                p { class: "kit-actx__empty", "Nothing learned or connected yet. Open {agent_name} to add files or tools." }
            }
        }
    }
}

#[component]
fn MemoryDetail(
    memory: Memory,
    outdated: bool,
    #[props(default)] on_switch_change: Option<EventHandler<(ItemKind, String, bool)>>,
    #[props(default)] on_forget: Option<EventHandler<Memory>>,
    on_close: EventHandler<()>,
) -> Element {
    let on = !memory.is_disabled;
    let id = memory.memory_id.clone();
    let forget_target = memory.clone();
    let title = format!("Memory: {}", memory.memory_text);
    rsx! {
        section {
            class: "kit-actx__panel", role: "dialog", "aria-label": "{title}", tabindex: "-1",
            onmounted: focus_on_mount,
            onkeydown: move |event| {
                if event.key() == Key::Escape {
                    event.prevent_default();
                    on_close.call(());
                }
            },
            h3 { class: "kit-actx__panel-title", "{memory.memory_text}" }
            p { class: "kit-actx__panel-line", "From {memory.source_artifact.artifact_name}" }
            if outdated { p { class: "kit-actx__panel-line", "Outdated" } }
            if let Some(link) = memory.source_artifact.artifact_link.clone() {
                a { class: "kit-actx__link", href: "{link}", "Open source" }
            }
            button {
                r#type: "button", class: "kit-actx__switch", role: "switch", "aria-checked": "{on}", "data-on": on.then_some("true"),
                onclick: move |_| {
                    if let Some(h) = on_switch_change {
                        h.call((ItemKind::Memory, id.clone(), !on));
                    }
                },
                if on { "On" } else { "Off" }
            }
            button {
                r#type: "button", class: "kit-actx__forget",
                onclick: move |_| {
                    if let Some(h) = on_forget {
                        h.call(forget_target.clone());
                    }
                },
                "Forget"
            }
            button { r#type: "button", class: "kit-actx__close", onclick: move |_| on_close.call(()), "Close" }
        }
    }
}

#[component]
fn ToolDetail(
    tool: Tool,
    #[props(default)] on_switch_change: Option<EventHandler<(ItemKind, String, bool)>>,
    on_close: EventHandler<()>,
) -> Element {
    let on = !tool.is_disabled;
    let id = tool.tool_id.clone();
    let title = format!("Tool: {}", tool.tool_name);
    rsx! {
        section {
            class: "kit-actx__panel", role: "dialog", "aria-label": "{title}", tabindex: "-1",
            onmounted: focus_on_mount,
            onkeydown: move |event| {
                if event.key() == Key::Escape {
                    event.prevent_default();
                    on_close.call(());
                }
            },
            h3 { class: "kit-actx__panel-title", "{tool.tool_name}" }
            p { class: "kit-actx__panel-line", "{tool.access_level} · {tool.scope_text}" }
            p { class: "kit-actx__panel-line", "{tool.call_count} calls · {tool.last_activity_text}" }
            button {
                r#type: "button", class: "kit-actx__switch", role: "switch", "aria-checked": "{on}", "data-on": on.then_some("true"),
                onclick: move |_| {
                    if let Some(h) = on_switch_change {
                        h.call((ItemKind::Tool, id.clone(), !on));
                    }
                },
                if on { "On" } else { "Off" }
            }
            button { r#type: "button", class: "kit-actx__close", onclick: move |_| on_close.call(()), "Close" }
        }
    }
}
