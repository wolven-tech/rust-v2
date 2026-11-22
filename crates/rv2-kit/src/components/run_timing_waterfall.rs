//! Run timing waterfall: the time and cost of an agent run on one timeline. Every model
//! call, tool call and sub-agent gets a bar; the longest finished model or tool call is
//! flagged; any branch collapses into its parent; selecting a node opens its particulars.
//!
//! Design choice: a parent spans from its earliest child start to its latest child end.
//! Its cost and tokens are the sum of its children, so callers supply leaf values only.
//! Design choice: the axis is a 1-2-5 round number at or above the run's end and
//! `min_span`; it widens in steps as the run grows.
//! Design choice: `collapsed_ids` and `selected_node_id` are `Option` controlled props
//! (`selected_node_id` is `Option<Option<_>>`, so the host can hold "nothing selected").
//! Design choice: the list is a roving-focus tree, and the particulars panel is a dialog
//! named after its node.
//! Not implemented: the live clock from `started_epoch`. There is no timer, so a live run
//! takes its position from `clock_point`; supply that from the host.
//! Browser checks: bar growth on live runs, the axis step easing, the panel's scale and
//! fade from its bar, the stacked layout at moderate and narrow widths.

use std::collections::BTreeSet;

use dioxus::prelude::*;

use crate::components::agent_context_map::focus_on_mount;
use crate::components::series_trend_chart::{finite_or_zero, focus_element, non_negative};
use crate::internal::{KIT_CSS, class_list, next_id};

const CSS: Asset = asset!("/assets/run-timing-waterfall.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NodeType {
    #[default]
    Agent,
    Subagent,
    Model,
    Tool,
}

impl NodeType {
    fn label(self) -> &'static str {
        match self {
            Self::Agent => "Agent",
            Self::Subagent => "Sub-agent",
            Self::Model => "Model",
            Self::Tool => "Tool",
        }
    }

    fn is_call(self) -> bool {
        matches!(self, Self::Model | Self::Tool)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NodeStatus {
    Running,
    #[default]
    Done,
    Failed,
}

impl NodeStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TraceNode {
    pub node_id: String,
    pub node_label: String,
    pub node_type: NodeType,
    pub node_status: NodeStatus,
    pub begin_offset: f64,
    /// Absent while the node runs.
    pub finish_offset: Option<f64>,
    pub node_note: Option<String>,
    pub node_tag: Option<String>,
    pub model_label: Option<String>,
    pub prompt_tokens: Option<f64>,
    pub completion_tokens: Option<f64>,
    pub node_cost: Option<f64>,
    pub input_excerpt: Option<String>,
    pub output_excerpt: Option<String>,
    pub failure_text: Option<String>,
    pub children: Vec<TraceNode>,
}

/// Start and end of a node. A node with children spans them, and a running node ends at
/// `clock`.
pub fn node_span(node: &TraceNode, clock: f64) -> (f64, f64) {
    let mut start = node.begin_offset;
    let mut end = node.finish_offset.unwrap_or(clock).max(node.begin_offset);
    for child in &node.children {
        let (cs, ce) = node_span(child, clock);
        start = start.min(cs);
        end = end.max(ce);
    }
    (start, end)
}

/// Cost of a node; a node with children sums them.
pub fn node_cost_total(node: &TraceNode) -> f64 {
    if node.children.is_empty() {
        finite_or_zero(node.node_cost.unwrap_or(0.0))
    } else {
        node.children.iter().map(node_cost_total).sum()
    }
}

/// Prompt and completion tokens of a node; a node with children sums them.
pub fn node_tokens_total(node: &TraceNode) -> f64 {
    if node.children.is_empty() {
        non_negative(node.prompt_tokens.unwrap_or(0.0))
            + non_negative(node.completion_tokens.unwrap_or(0.0))
    } else {
        node.children.iter().map(node_tokens_total).sum()
    }
}

/// Id of the longest finished model or tool call. Ties keep the first in tree order.
pub fn slowest_id(root: &TraceNode) -> Option<String> {
    fn walk(node: &TraceNode, best: &mut Option<(String, f64)>) {
        if node.node_type.is_call() {
            if let Some(finish) = node.finish_offset {
                let duration = finish - node.begin_offset;
                if best.as_ref().is_none_or(|(_, d)| duration > *d) {
                    *best = Some((node.node_id.clone(), duration));
                }
            }
        }
        for child in &node.children {
            walk(child, best);
        }
    }
    let mut best = None;
    walk(root, &mut best);
    best.map(|(id, _)| id)
}

/// Axis end for a run ending at `run_end`, never below `min_span`, on a round step.
pub fn axis_end(run_end: f64, min_span: f64) -> f64 {
    let wanted = run_end.max(min_span).max(f64::EPSILON);
    let magnitude = 10f64.powf(wanted.log10().floor());
    [1.0, 2.0, 5.0, 10.0]
        .into_iter()
        .map(|factor| magnitude * factor)
        .find(|step| *step >= wanted)
        .unwrap_or(wanted)
}

/// Counts of the run: model calls, tool calls and failed nodes.
pub fn counts(root: &TraceNode) -> (usize, usize, usize) {
    fn walk(node: &TraceNode, acc: &mut (usize, usize, usize)) {
        match node.node_type {
            NodeType::Model => acc.0 += 1,
            NodeType::Tool => acc.1 += 1,
            _ => {}
        }
        if node.node_status == NodeStatus::Failed {
            acc.2 += 1;
        }
        for child in &node.children {
            walk(child, acc);
        }
    }
    let mut acc = (0, 0, 0);
    walk(root, &mut acc);
    acc
}

/// A row of the flattened trace.
#[derive(Clone, Debug, PartialEq)]
pub struct TraceRow {
    pub node: TraceNode,
    pub depth: usize,
    pub parent: Option<usize>,
    pub has_children: bool,
    pub expanded: bool,
}

/// Rows in display order; the children of a collapsed node are left out.
pub fn flatten_trace(root: &TraceNode, collapsed: &BTreeSet<String>) -> Vec<TraceRow> {
    fn push(
        node: &TraceNode,
        depth: usize,
        parent: Option<usize>,
        collapsed: &BTreeSet<String>,
        out: &mut Vec<TraceRow>,
    ) {
        let index = out.len();
        let has_children = !node.children.is_empty();
        let expanded = has_children && !collapsed.contains(&node.node_id);
        out.push(TraceRow {
            node: node.clone(),
            depth,
            parent,
            has_children,
            expanded,
        });
        if expanded {
            for child in &node.children {
                push(child, depth + 1, Some(index), collapsed, out);
            }
        }
    }
    let mut out = Vec::new();
    push(root, 0, None, collapsed, &mut out);
    out
}

/// Start offsets of the descendants hidden under a collapsed node: its calls become ticks.
fn hidden_ticks(node: &TraceNode, out: &mut Vec<f64>) {
    for child in &node.children {
        if child.node_type.is_call() {
            out.push(child.begin_offset);
        }
        hidden_ticks(child, out);
    }
}

fn format_currency(amount: f64) -> String {
    format!("${:.2}", finite_or_zero(amount))
}

fn seconds(value: f64) -> String {
    format!("{:.1}s", finite_or_zero(value))
}

/// Run clock for the heading totals: whole seconds read as "9 seconds".
fn clock_total(value: f64) -> String {
    let value = finite_or_zero(value);
    let unit = if value == 1.0 { "second" } else { "seconds" };
    if value.fract() == 0.0 {
        format!("{value:.0} {unit}")
    } else {
        format!("{value:.1} {unit}")
    }
}

/// Whole count with thousands separators: `6,200`.
fn grouped_count(value: f64) -> String {
    let digits = format!("{:.0}", finite_or_zero(value).max(0.0));
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

/// A timeline of the run's nodes with a particulars dialog for the selected node.
#[component]
pub fn RunTimingWaterfall(
    trace_tree: TraceNode,
    #[props(default)] clock_point: Option<f64>,
    #[props(default)] collapsed_ids: Option<Vec<String>>,
    #[props(default)] initial_collapsed_ids: Vec<String>,
    #[props(default)] on_collapse_change: Option<EventHandler<Vec<String>>>,
    #[props(default)] selected_node_id: Option<Option<String>>,
    #[props(default)] initial_selected_node_id: Option<String>,
    #[props(default)] on_selection_change: Option<
        EventHandler<(Option<String>, Option<TraceNode>)>,
    >,
    #[props(default = true)] flag_slowest: bool,
    #[props(default = 1.0)] min_span: f64,
    #[props(default = format_currency as fn(f64) -> String)] format_cost: fn(f64) -> String,
    #[props(default = "Trace".to_string(), into)] heading: String,
    #[props(default, into)] root_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-wf-{}", next_id()));
    let mut collapsed_local = use_signal(|| {
        initial_collapsed_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<String>>()
    });
    let mut selected_local = use_signal(|| initial_selected_node_id.clone());
    let mut focus_index = use_signal(|| 0usize);

    let collapsed: BTreeSet<String> = match &collapsed_ids {
        Some(ids) => ids.iter().cloned().collect(),
        None => collapsed_local(),
    };
    let selected: Option<String> = match &selected_node_id {
        Some(value) => value.clone(),
        None => selected_local(),
    };
    let collapsed_controlled = collapsed_ids.is_some();
    let selected_controlled = selected_node_id.is_some();

    let (run_start, run_end) = node_span(&trace_tree, clock_point.unwrap_or(0.0));
    let clock = clock_point.unwrap_or(run_end);
    let (run_start, run_end) = (run_start, run_end.max(clock));
    let live = trace_tree_running(&trace_tree);
    let status_label = if live {
        "In flight"
    } else if trace_tree.node_status == NodeStatus::Failed || counts(&trace_tree).2 > 0 {
        "Failed"
    } else {
        "Done"
    };
    let status_key = if live {
        "live"
    } else if status_label == "Failed" {
        "failed"
    } else {
        "done"
    };
    let axis_max = axis_end(run_end - run_start, min_span);
    let span_of = |start: f64, end: f64| -> (f64, f64) {
        let left = ((start - run_start) / axis_max * 100.0).clamp(0.0, 100.0);
        let width = ((end - start) / axis_max * 100.0).clamp(0.0, 100.0 - left);
        (left, width)
    };
    let rows = flatten_trace(&trace_tree, &collapsed);
    let slowest = if flag_slowest {
        slowest_id(&trace_tree)
    } else {
        None
    };
    let focus = focus_index().min(rows.len().saturating_sub(1));
    let total_cost = format_cost(node_cost_total(&trace_tree));
    let total_tokens = grouped_count(node_tokens_total(&trace_tree));
    let (model_calls, tool_calls, errors) = counts(&trace_tree);
    let elapsed = clock_total(run_end - run_start);
    let clock_left = span_of(clock, clock).0;
    let show_clock = live;
    let class = class_list("kit-wf", &root_class);
    let root_label = trace_tree.node_label.clone();
    let key_base = base.clone();
    let selected_row = selected
        .as_ref()
        .and_then(|id| rows.iter().position(|r| r.node.node_id == *id));
    let selected_node = selected.as_ref().and_then(|id| {
        rows.iter()
            .find(|r| r.node.node_id == *id)
            .map(|r| r.node.clone())
    });

    let commit_collapse = move |next: BTreeSet<String>| {
        if !collapsed_controlled {
            collapsed_local.set(next.clone());
        }
        if let Some(handler) = &on_collapse_change {
            handler.call(next.into_iter().collect());
        }
    };
    let commit_selection = move |id: Option<String>, node: Option<TraceNode>| {
        if !selected_controlled {
            selected_local.set(id.clone());
        }
        if let Some(handler) = &on_selection_change {
            handler.call((id, node));
        }
    };

    let mut row_elements: Vec<Element> = Vec::new();
    for (idx, row) in rows.iter().enumerate() {
        let node = row.node.clone();
        let row_collapsed = collapsed.clone();
        let row_id = format!("{key_base}-row-{idx}");
        let (start, end) = node_span(&node, clock);
        let (left, width) = span_of(start, end);
        let is_slowest = slowest.as_deref() == Some(node.node_id.as_str());
        let is_selected = selected.as_deref() == Some(node.node_id.as_str());
        let is_focus = idx == focus;
        let folded = collapsed.contains(&node.node_id);
        let cost_text = format_cost(node_cost_total(&node));
        let elapsed_text = seconds(end - start);
        let depth_attr = (row.depth + 1).to_string();
        let pad = format!("padding-left: calc({} * var(--kit-space-4))", row.depth);
        let bar_style = format!("left: {left:.3}%; width: {width:.3}%");
        let mut ticks: Vec<String> = Vec::new();
        if folded {
            let mut offsets = Vec::new();
            hidden_ticks(&node, &mut offsets);
            for offset in offsets {
                let tick_left = span_of(offset, offset).0;
                ticks.push(format!("left: {tick_left:.3}%"));
            }
        }
        let aria = format!(
            "{}, {}, {}, {}, {}",
            node.node_label,
            node.node_type.label(),
            node.node_status.label(),
            elapsed_text,
            cost_text
        );
        let label = node.node_label.clone();
        let type_label = node.node_type.label();
        let type_tile: String = type_label.chars().take(1).collect();
        let status_name = node.node_status.label();
        let toggle_id = node.node_id.clone();
        let click_id = node.node_id.clone();
        let click_node = node.clone();
        let mut commit_collapse_row = commit_collapse;
        let mut commit_selection_row = commit_selection;
        let expanded_attr = row
            .has_children
            .then_some(if row.expanded { "true" } else { "false" });
        row_elements.push(rsx! {
            li {
                key: "{row_id}",
                id: "{row_id}",
                role: "treeitem",
                class: "kit-wf__row",
                "aria-level": "{depth_attr}",
                "aria-expanded": expanded_attr,
                "aria-label": "{aria}",
                "aria-selected": is_selected.then_some("true"),
                tabindex: if is_focus { "0" } else { "-1" },
                "data-status": status_name,
                "data-kind": "{type_label}",
                "data-slowest": is_slowest.then_some("true"),
                "data-selected": is_selected.then_some("true"),
                onfocus: move |_| focus_index.set(idx),
                div { class: "kit-wf__label", style: pad,
                    if row.has_children {
                        button {
                            r#type: "button",
                            class: "kit-wf__toggle",
                            tabindex: "-1",
                            "data-expanded": (!folded).then_some("true"),
                            "aria-label": "{label} branch",
                            onclick: move |event: MouseEvent| {
                                event.stop_propagation();
                                let mut next = row_collapsed.clone();
                                if next.contains(&toggle_id) {
                                    next.remove(&toggle_id);
                                } else {
                                    next.insert(toggle_id.clone());
                                }
                                commit_collapse_row(next);
                            },
                            "▸"
                        }
                    } else {
                        span { class: "kit-wf__toggle-spacer", "aria-hidden": "true" }
                    }
                    span { class: "kit-wf__tile", "aria-hidden": "true", "{type_tile}" }
                    span { class: "kit-wf__name", "{label}" }
                    span { class: "kit-wf__type", "{type_label}" }
                    if is_slowest {
                        span { class: "kit-wf__tag", "Slowest" }
                    }
                }
                div {
                    class: "kit-wf__timeline",
                    "aria-hidden": "true",
                    onclick: move |_| {
                        focus_index.set(idx);
                        commit_selection_row(
                            if is_selected { None } else { Some(click_id.clone()) },
                            if is_selected { None } else { Some(click_node.clone()) },
                        );
                    },
                    span { class: "kit-wf__bar", style: bar_style,
                        span { class: "kit-wf__caption", "{elapsed_text} · {cost_text}" }
                    }
                    for (t, tick) in ticks.into_iter().enumerate() {
                        span { key: "{t}", class: "kit-wf__tick", style: tick }
                    }
                }
                span { class: "kit-wf__figures", "{cost_text}" }
            }
        });
    }

    let panel = selected_node.clone().map(|node| {
        let (start, end) = node_span(&node, clock);
        let timing = format!(
            "{} to {} ({})",
            seconds(start - run_start),
            seconds(end - run_start),
            seconds(end - start)
        );
        let model = node.model_label.clone().unwrap_or_else(|| "–".to_string());
        let tokens = format!(
            "{:.0} in · {:.0} out",
            non_negative(node.prompt_tokens.unwrap_or(0.0)),
            non_negative(node.completion_tokens.unwrap_or(0.0))
        );
        let cost = format_cost(node_cost_total(&node));
        let input = node.input_excerpt.clone().unwrap_or_default();
        let output = node.output_excerpt.clone().unwrap_or_default();
        let failure = node.failure_text.clone();
        let panel_label = node.node_label.clone();
        let type_name = node.node_type.label();
        let status_name = node.node_status.label();
        let note = node.node_note.clone().unwrap_or_default();
        (
            panel_label,
            type_name,
            status_name,
            timing,
            model,
            tokens,
            cost,
            input,
            output,
            failure,
            note,
        )
    });
    let panel_key = selected.clone().unwrap_or_default();
    let escape_row = selected_row.map(|i| format!("{key_base}-row-{i}"));
    let mut close_panel = commit_selection;
    let keys_rows = rows.clone();
    let keys_base = base.clone();
    let keys_collapsed = collapsed.clone();
    let mut keys_collapse = commit_collapse;
    let mut keys_selection = commit_selection;
    let keys_selected = selected.clone();

    let list_keys = move |event: KeyboardEvent| {
        let count = keys_rows.len();
        if count == 0 {
            return;
        }
        let current = focus_index().min(count - 1);
        let row = &keys_rows[current];
        let row_id_of = |i: usize| format!("{keys_base}-row-{i}");
        let mut move_to = |i: usize| {
            focus_index.set(i);
            focus_element(&row_id_of(i));
        };
        match event.key() {
            Key::ArrowDown => {
                event.prevent_default();
                move_to((current + 1).min(count - 1));
            }
            Key::ArrowUp => {
                event.prevent_default();
                move_to(current.saturating_sub(1));
            }
            Key::Home => {
                event.prevent_default();
                move_to(0);
            }
            Key::End => {
                event.prevent_default();
                move_to(count - 1);
            }
            Key::ArrowRight => {
                event.prevent_default();
                if row.has_children && !row.expanded {
                    let mut next = keys_collapsed.clone();
                    next.remove(&row.node.node_id);
                    keys_collapse(next);
                } else if row.expanded && current + 1 < count {
                    move_to(current + 1);
                }
            }
            Key::ArrowLeft => {
                event.prevent_default();
                if row.expanded {
                    let mut next = keys_collapsed.clone();
                    next.insert(row.node.node_id.clone());
                    keys_collapse(next);
                } else if let Some(parent) = row.parent {
                    move_to(parent);
                }
            }
            Key::Enter => {
                event.prevent_default();
                let id = row.node.node_id.clone();
                let open = keys_selected.as_deref() == Some(id.as_str());
                keys_selection(
                    if open { None } else { Some(id) },
                    if open { None } else { Some(row.node.clone()) },
                );
            }
            Key::Character(c) if c == " " => {
                event.prevent_default();
                let id = row.node.node_id.clone();
                let open = keys_selected.as_deref() == Some(id.as_str());
                keys_selection(
                    if open { None } else { Some(id) },
                    if open { None } else { Some(row.node.clone()) },
                );
            }
            _ => {}
        }
    };

    let ticks: Vec<(usize, String, String)> = (0..=4)
        .map(|i| {
            let value = axis_max * i as f64 / 4.0;
            let pos = span_of(run_start + value, run_start + value).0;
            (i, format!("left: {pos:.3}%"), seconds(value))
        })
        .collect();
    let clock_style = format!("left: {clock_left:.3}%");

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: class, "aria-label": heading.clone(), "data-status": status_key,
            header { class: "kit-wf__head",
                div { class: "kit-wf__titles",
                    span { class: "kit-wf__status", role: "status", "aria-live": "polite", "data-status": status_key, "{status_label}" }
                    span { class: "kit-wf__root", "{root_label}" }
                }
                dl { class: "kit-wf__totals",
                    div { dt { "Clock" } dd { "{elapsed}" } }
                    div { dt { "Charges" } dd { "{total_cost}" } }
                    div { dt { "Token volume" } dd { "{total_tokens}" } }
                    div { dt { "Model turns" } dd { "{model_calls}" } }
                    div { dt { "Tool runs" } dd { "{tool_calls}" } }
                    div { dt { "Failures" } dd { "{errors}" } }
                }
            }
            ul { class: "kit-wf__legend", "aria-label": "Node kinds",
                li { span { class: "kit-wf__key", "data-key": "agent", "aria-hidden": "true" }, "Agent" }
                li { span { class: "kit-wf__key", "data-key": "model", "aria-hidden": "true" }, "Model" }
                li { span { class: "kit-wf__key", "data-key": "tool", "aria-hidden": "true" }, "Tool" }
                li { span { class: "kit-wf__key", "data-key": "subagent", "aria-hidden": "true" }, "Sub-agent" }
            }
            div { class: "kit-wf__body",
                div { class: "kit-wf__plot",
                    ul {
                        class: "kit-wf__list",
                        role: "tree",
                        "aria-label": "{heading}",
                        onkeydown: list_keys,
                        {row_elements.into_iter()}
                    }
                    if show_clock {
                        div { class: "kit-wf__clock-track", "aria-hidden": "true",
                            span { class: "kit-wf__clock", style: clock_style }
                        }
                    }
                    div { class: "kit-wf__timeline-axis", "aria-hidden": "true",
                        div { class: "kit-wf__ticks",
                            for (i, pos, text) in ticks.clone() {
                                span { key: "{i}", class: "kit-wf__axis-tick", style: pos, "{text}" }
                            }
                        }
                    }
                }
            }
            if let Some((panel_label, type_name, status_name, timing, model, tokens, cost, input, output, failure, note)) = panel {
                div {
                    class: "kit-wf__panel",
                    role: "dialog",
                    "aria-label": "{panel_label}",
                    "data-key": "{panel_key}",
                    tabindex: "-1",
                    onmounted: focus_on_mount,
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key() == Key::Escape {
                            event.prevent_default();
                            close_panel(None, None);
                            if let Some(row_id) = escape_row.clone() {
                                focus_element(&row_id);
                            }
                        }
                    },
                    header { class: "kit-wf__panel-head",
                        span { class: "kit-wf__panel-title", "{panel_label}" }
                        span { class: "kit-wf__panel-type", "{type_name} · {status_name}" }
                    }
                    if !note.is_empty() {
                        p { class: "kit-wf__panel-note", "{note}" }
                    }
                    dl { class: "kit-wf__panel-facts",
                        div { dt { "Timing" } dd { "{timing}" } }
                        div { dt { "Model" } dd { "{model}" } }
                        div { dt { "Tokens" } dd { "{tokens}" } }
                        div { dt { "Cost" } dd { "{cost}" } }
                    }
                    if !input.is_empty() {
                        div { class: "kit-wf__excerpt",
                            span { class: "kit-wf__excerpt-label", "Input" }
                            pre { class: "kit-wf__excerpt-body", tabindex: "0", "{input}" }
                        }
                    }
                    if !output.is_empty() {
                        div { class: "kit-wf__excerpt",
                            span { class: "kit-wf__excerpt-label", "Output" }
                            pre { class: "kit-wf__excerpt-body", tabindex: "0", "{output}" }
                        }
                    }
                    if let Some(text) = failure {
                        p { class: "kit-wf__failure", role: "alert", "{text}" }
                    }
                }
            }
        }
    }
}

fn trace_tree_running(node: &TraceNode) -> bool {
    node.node_status == NodeStatus::Running || node.children.iter().any(trace_tree_running)
}
