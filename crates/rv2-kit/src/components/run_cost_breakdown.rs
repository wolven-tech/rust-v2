//! Run cost breakdown: the cost and tokens of one agent run, step by step, so the
//! expensive step is easy to spot. Sub-agents fold; a row opens its prompt and result,
//! or its error when it failed.
//!
//! Design choice: a sub-agent's cost and tokens are the sum of its children. Callers
//! supply leaf values only for steps that have children.
//! Design choice: a non-finite cost shows a dash and counts as zero. A negative cost is a
//! credit: it takes the credit style and lowers the running total.
//! Design choice: the costliest step is chosen among top-level steps; ties keep order.
//! Design choice: the tree is a roving-focus `role="tree"`. The list handles the keys, so
//! exactly one row holds the tab stop.
//! Design choice: `detail_step_id` and `folded_step_ids` are `Option` controlled props,
//! following the lane convention. Without them the component keeps its own state.
//! Not implemented: the step icon (`step_icon`) and the sliding entry of new rows.
//! Browser checks: the moving highlight on in-progress bars, the bar spanning full width
//! below narrow widths, detail boxes reachable with Tab, touch target sizes.

use std::collections::BTreeSet;

use dioxus::prelude::*;

use crate::components::series_trend_chart::{finite_or_zero, focus_element, non_negative};
use crate::internal::{KIT_CSS, class_list, next_id};

const CSS: Asset = asset!("/assets/run-cost-breakdown.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepKind {
    #[default]
    Agent,
    ModelCall,
    ToolCall,
}

impl StepKind {
    fn label(self) -> &'static str {
        match self {
            Self::Agent => "Agent",
            Self::ModelCall => "Model call",
            Self::ToolCall => "Tool call",
        }
    }

    /// Labels of the detail view: task and result, prompt and response, input and output.
    fn detail_labels(self) -> (&'static str, &'static str) {
        match self {
            Self::Agent => ("Task", "Result"),
            Self::ModelCall => ("Prompt", "Response"),
            Self::ToolCall => ("Input", "Output"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepStatus {
    InProgress,
    #[default]
    Done,
    Failed,
}

impl StepStatus {
    fn label(self) -> &'static str {
        match self {
            Self::InProgress => "in progress",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RunState {
    InProgress,
    #[default]
    Done,
    Failed,
}

impl RunState {
    fn label(self) -> &'static str {
        match self {
            Self::InProgress => "In progress",
            Self::Done => "Done",
            Self::Failed => "Failed",
        }
    }

    fn key(self) -> &'static str {
        match self {
            Self::InProgress => "in_progress",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Measure {
    #[default]
    Cost,
    Tokens,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    pub step_id: String,
    pub step_name: String,
    pub step_kind: StepKind,
    pub step_status: StepStatus,
    pub model_name: Option<String>,
    pub step_note: Option<String>,
    pub in_tokens: Option<f64>,
    pub out_tokens: Option<f64>,
    pub step_cost: Option<f64>,
    pub step_elapsed: Option<f64>,
    pub prompt_text: Option<String>,
    pub result_text: Option<String>,
    pub error_text: Option<String>,
    pub child_steps: Vec<Step>,
}

/// Cost of a step; a step with children sums them.
pub fn step_cost(step: &Step) -> f64 {
    if step.child_steps.is_empty() {
        finite_or_zero(step.step_cost.unwrap_or(0.0))
    } else {
        step.child_steps.iter().map(step_cost).sum()
    }
}

/// Input and output tokens of a step; a step with children sums them.
pub fn step_tokens(step: &Step) -> (f64, f64) {
    if step.child_steps.is_empty() {
        (
            non_negative(step.in_tokens.unwrap_or(0.0)),
            non_negative(step.out_tokens.unwrap_or(0.0)),
        )
    } else {
        step.child_steps
            .iter()
            .map(step_tokens)
            .fold((0.0, 0.0), |(a, b), (c, d)| (a + c, b + d))
    }
}

/// The figure a bar encodes under `measure`.
pub fn step_value(step: &Step, measure: Measure) -> f64 {
    match measure {
        Measure::Cost => step_cost(step),
        Measure::Tokens => {
            let (input, output) = step_tokens(step);
            input + output
        }
    }
}

/// The run's state: a given override, else In progress while any step runs, else Failed
/// if any step failed, else Done.
pub fn run_state(steps: &[Step], override_state: Option<RunState>) -> RunState {
    if let Some(state) = override_state {
        return state;
    }
    fn walk(steps: &[Step], running: &mut bool, failed: &mut bool) {
        for step in steps {
            match step.step_status {
                StepStatus::InProgress => *running = true,
                StepStatus::Failed => *failed = true,
                StepStatus::Done => {}
            }
            walk(&step.child_steps, running, failed);
        }
    }
    let (mut running, mut failed) = (false, false);
    walk(steps, &mut running, &mut failed);
    if running {
        RunState::InProgress
    } else if failed {
        RunState::Failed
    } else {
        RunState::Done
    }
}

/// Index of the costliest top-level step; ties keep the first.
pub fn costliest_index(steps: &[Step]) -> Option<usize> {
    let mut best: Option<(usize, f64)> = None;
    for (i, step) in steps.iter().enumerate() {
        let cost = step_cost(step);
        if best.is_none_or(|(_, b)| cost > b) {
            best = Some((i, cost));
        }
    }
    best.map(|(i, _)| i)
}

/// A row of the flattened tree.
#[derive(Clone, Debug, PartialEq)]
pub struct FlatRow {
    pub step: Step,
    pub depth: usize,
    pub parent: Option<usize>,
    pub has_children: bool,
    pub expanded: bool,
}

/// Rows in display order; children of a folded step are left out.
pub fn flatten(steps: &[Step], folded: &BTreeSet<String>) -> Vec<FlatRow> {
    fn push(
        steps: &[Step],
        depth: usize,
        parent: Option<usize>,
        folded: &BTreeSet<String>,
        out: &mut Vec<FlatRow>,
    ) {
        for step in steps {
            let index = out.len();
            let has_children = !step.child_steps.is_empty();
            let expanded = has_children && !folded.contains(&step.step_id);
            out.push(FlatRow {
                step: step.clone(),
                depth,
                parent,
                has_children,
                expanded,
            });
            if expanded {
                push(&step.child_steps, depth + 1, Some(index), folded, out);
            }
        }
    }
    let mut out = Vec::new();
    push(steps, 0, None, folded, &mut out);
    out
}

/// Dollars to three decimals; a non-finite value is a dash.
pub fn format_dollars(amount: f64) -> String {
    if amount.is_finite() {
        format!("${amount:.3}")
    } else {
        "–".to_string()
    }
}

fn token_text(value: f64) -> String {
    format!("{:.0}", non_negative(value))
}

/// A panel showing one run's steps as a tree with cost and token bars.
#[component]
pub fn RunCostBreakdown(
    step_list: Vec<Step>,
    #[props(default = "Run cost".to_string(), into)] run_label: String,
    #[props(default)] run_summary: Option<String>,
    #[props(default)] status_override: Option<RunState>,
    #[props(default)] measure: Option<Measure>,
    #[props(default)] starting_measure: Measure,
    #[props(default)] on_measure_change: Option<EventHandler<Measure>>,
    #[props(default)] folded_step_ids: Option<Vec<String>>,
    #[props(default)] starting_folded_step_ids: Vec<String>,
    #[props(default)] on_fold_change: Option<EventHandler<Vec<String>>>,
    #[props(default)] detail_step_id: Option<Option<String>>,
    #[props(default)] starting_detail_step_id: Option<String>,
    #[props(default)] on_detail_change: Option<EventHandler<(Option<String>, Option<Step>)>>,
    #[props(default)] max_list_height: Option<f64>,
    #[props(default = format_dollars as fn(f64) -> String)] format_money: fn(f64) -> String,
    #[props(default, into)] extra_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-rcb-{}", next_id()));
    let mut measure_local = use_signal(|| starting_measure);
    let mut folded_local = use_signal(|| {
        starting_folded_step_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<String>>()
    });
    let mut detail_local = use_signal(|| starting_detail_step_id.clone());
    let mut focus_index = use_signal(|| 0usize);

    let current_measure = measure.unwrap_or(measure_local());
    let folded: BTreeSet<String> = match &folded_step_ids {
        Some(ids) => ids.iter().cloned().collect(),
        None => folded_local(),
    };
    let detail: Option<String> = match &detail_step_id {
        Some(value) => value.clone(),
        None => detail_local(),
    };
    let folded_controlled = folded_step_ids.is_some();
    let detail_controlled = detail_step_id.is_some();
    let state = run_state(&step_list, status_override);
    let rows = flatten(&step_list, &folded);
    let max_abs = rows
        .iter()
        .map(|r| step_value(&r.step, current_measure).abs())
        .fold(0.0_f64, f64::max);
    let total_cost: f64 = step_list.iter().map(step_cost).sum();
    let (total_in, total_out) = step_list
        .iter()
        .map(step_tokens)
        .fold((0.0, 0.0), |(a, b), (c, d)| (a + c, b + d));
    let costliest = costliest_index(&step_list).map(|i| step_list[i].clone());
    let focus = focus_index().min(rows.len().saturating_sub(1));
    let class = class_list("kit-rcb", &extra_class);
    let list_style = max_list_height
        .map(|h| format!("max-height: {h:.0}px"))
        .unwrap_or_default();
    let heading = run_label.clone();
    let summary = run_summary.clone();
    let total_text = format_money(total_cost);
    let tokens_text = format!(
        "{} in · {} out",
        token_text(total_in),
        token_text(total_out)
    );
    let key_base = base.clone();
    let cost_is_measure = current_measure == Measure::Cost;
    let tokens_is_measure = current_measure == Measure::Tokens;

    let commit_fold = move |next: BTreeSet<String>| {
        if !folded_controlled {
            folded_local.set(next.clone());
        }
        if let Some(handler) = &on_fold_change {
            handler.call(next.into_iter().collect());
        }
    };
    let commit_detail = move |id: Option<String>| {
        if !detail_controlled {
            detail_local.set(id.clone());
        }
        if let Some(handler) = &on_detail_change {
            handler.call((id.clone(), None));
        }
    };
    let mut set_measure = move |next: Measure| {
        if measure.is_none() {
            measure_local.set(next);
        }
        if let Some(handler) = &on_measure_change {
            handler.call(next);
        }
    };

    let mut list_elements: Vec<Element> = Vec::new();
    for (idx, row) in rows.iter().enumerate() {
        let step = row.step.clone();
        let row_folded = folded.clone();
        let id = step.step_id.clone();
        let row_id = format!("{key_base}-row-{idx}");
        let (input, output) = step_tokens(&step);
        let value_raw = step_value(&step, current_measure);
        let width = if max_abs > 0.0 {
            format!("width: {:.2}%", value_raw.abs() / max_abs * 100.0)
        } else {
            "width: 0%".to_string()
        };
        let figure = match current_measure {
            Measure::Cost => format_money(step_cost(&step)),
            Measure::Tokens => token_text(input + output),
        };
        let time_text = match step.step_elapsed {
            Some(t) if t.is_finite() => format!("{t:.1}s"),
            _ => "–".to_string(),
        };
        let credit = value_raw < 0.0;
        let failed = step.step_status == StepStatus::Failed;
        let running = step.step_status == StepStatus::InProgress;
        let aria = format!(
            "{}, {}, {}, {}, {}",
            step.step_name,
            step.step_kind.label(),
            step.step_status.label(),
            figure,
            time_text
        );
        let name = step.step_name.clone();
        let kind = step.step_kind.label();
        let model = step.model_name.clone().unwrap_or_default();
        let in_text = token_text(input);
        let out_text = token_text(output);
        let is_open = detail.as_deref() == Some(id.as_str());
        let folded_now = folded.contains(&id);
        let depth_attr = (row.depth + 1).to_string();
        let pad = format!("padding-left: {}px", row.depth * 16);
        let status_name = step.step_status.label();
        let is_focus = idx == focus;
        let fold_label = if folded_now { "Expand" } else { "Collapse" };
        let toggle_id = id.clone();
        let click_id = id.clone();
        let mut commit_fold_row = commit_fold;
        let mut commit_detail_row = commit_detail;
        let expanded_attr = row
            .has_children
            .then_some(if row.expanded { "true" } else { "false" });
        list_elements.push(rsx! {
            li {
                key: "{row_id}",
                id: "{row_id}",
                role: "treeitem",
                class: "kit-rcb__item",
                "aria-level": "{depth_attr}",
                "aria-expanded": expanded_attr,
                "aria-label": "{aria}",
                tabindex: if is_focus { "0" } else { "-1" },
                "data-status": status_name,
                "data-failed": failed.then_some("true"),
                "data-running": running.then_some("true"),
                "data-credit": credit.then_some("true"),
                "data-open": is_open.then_some("true"),
                style: pad,
                onfocus: move |_| focus_index.set(idx),
                onclick: move |_| {
                    focus_index.set(idx);
                    commit_detail_row(if is_open { None } else { Some(click_id.clone()) });
                },
                div { class: "kit-rcb__row",
                    if row.has_children {
                        button {
                            r#type: "button",
                            class: "kit-rcb__fold",
                            tabindex: "-1",
                            "aria-label": "{fold_label} {name}",
                            onclick: move |event: MouseEvent| {
                                event.stop_propagation();
                                let mut next = row_folded.clone();
                                if next.contains(&toggle_id) {
                                    next.remove(&toggle_id);
                                } else {
                                    next.insert(toggle_id.clone());
                                }
                                commit_fold_row(next);
                            },
                            "▸"
                        }
                    } else {
                        span { class: "kit-rcb__fold-spacer", "aria-hidden": "true" }
                    }
                    span { class: "kit-rcb__name", "{name}" }
                    span { class: "kit-rcb__kind", "{kind}" }
                    span { class: "kit-rcb__model", "{model}" }
                    span { class: "kit-rcb__in", "{in_text}" }
                    span { class: "kit-rcb__out", "{out_text}" }
                    span { class: "kit-rcb__figure", "data-credit": credit.then_some("true"), "{figure}" }
                    span { class: "kit-rcb__time", "{time_text}" }
                }
                div { class: "kit-rcb__track", "aria-hidden": "true",
                    span { class: "kit-rcb__bar", style: width }
                }
            }
        });

        if is_open {
            let (in_label, out_label) = step.step_kind.detail_labels();
            let (body, body_label) = if step.step_status == StepStatus::Failed {
                (
                    step.error_text.clone().unwrap_or_default(),
                    "Error".to_string(),
                )
            } else {
                (
                    step.result_text.clone().unwrap_or_default(),
                    out_label.to_string(),
                )
            };
            let prompt = step.prompt_text.clone().unwrap_or_default();
            let detail_id = format!("{key_base}-detail-{idx}");
            let detail_label = format!("{} details", step.step_name);
            let escape_row_id = row_id.clone();
            let mut commit_escape = commit_detail;
            list_elements.push(rsx! {
                li { key: "{detail_id}", role: "none", class: "kit-rcb__detail-wrap",
                    div {
                        id: "{detail_id}",
                        class: "kit-rcb__detail",
                        role: "region",
                        "aria-label": "{detail_label}",
                        onkeydown: move |event: KeyboardEvent| {
                            if event.key() == Key::Escape {
                                event.prevent_default();
                                commit_escape(None);
                                focus_element(&escape_row_id);
                            }
                        },
                        if !prompt.is_empty() {
                            div { class: "kit-rcb__io",
                                span { class: "kit-rcb__io-label", "{in_label}" }
                                pre { class: "kit-rcb__io-body", tabindex: "0", "{prompt}" }
                            }
                        }
                        if !body.is_empty() {
                            div { class: "kit-rcb__io",
                                span { class: "kit-rcb__io-label", "{body_label}" }
                                pre { class: "kit-rcb__io-body", tabindex: "0", "{body}" }
                            }
                        }
                    }
                }
            });
        }
    }

    let keys_rows = rows.clone();
    let keys_folded = folded.clone();
    let keys_detail = detail.clone();
    let keys_base = base.clone();
    let mut keys_fold = commit_fold;
    let mut keys_detail_commit = commit_detail;
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
                    let mut next = keys_folded.clone();
                    next.remove(&row.step.step_id);
                    keys_fold(next);
                } else if row.expanded && current + 1 < count {
                    move_to(current + 1);
                }
            }
            Key::ArrowLeft => {
                event.prevent_default();
                if row.expanded {
                    let mut next = keys_folded.clone();
                    next.insert(row.step.step_id.clone());
                    keys_fold(next);
                } else if let Some(parent) = row.parent {
                    move_to(parent);
                }
            }
            Key::Enter => {
                event.prevent_default();
                let id = row.step.step_id.clone();
                keys_detail_commit(if keys_detail.as_deref() == Some(id.as_str()) {
                    None
                } else {
                    Some(id)
                });
            }
            Key::Character(c) if c == " " => {
                event.prevent_default();
                let id = row.step.step_id.clone();
                keys_detail_commit(if keys_detail.as_deref() == Some(id.as_str()) {
                    None
                } else {
                    Some(id)
                });
            }
            _ => {}
        }
    };

    let costliest_view = costliest.map(|step| {
        let cid = step.step_id.clone();
        let label = format!(
            "Costliest: {} · {}",
            step.step_name,
            format_money(step_cost(&step))
        );
        (cid, label)
    });
    let state_label = state.label();
    let state_key = state.key();
    let keys_base_for_costliest = base.clone();
    let mut costly_fold = commit_fold;
    let mut costly_detail = commit_detail;
    let costly_steps = step_list.clone();
    let costly_folded = folded.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: class, "aria-label": heading.clone(), "data-state": state_key,
            header { class: "kit-rcb__head",
                div { class: "kit-rcb__titles",
                    h3 { class: "kit-rcb__title", "{heading}" }
                    if let Some(text) = summary.clone() {
                        p { class: "kit-rcb__summary", "{text}" }
                    }
                }
                span { class: "kit-rcb__state", role: "status", "aria-live": "polite", "data-state": state_key, "{state_label}" }
            }
            div { class: "kit-rcb__measures", role: "group", "aria-label": "Measure",
                button {
                    r#type: "button",
                    class: "kit-rcb__measure",
                    "aria-pressed": if cost_is_measure { "true" } else { "false" },
                    onclick: move |_| set_measure(Measure::Cost),
                    "Cost"
                }
                button {
                    r#type: "button",
                    class: "kit-rcb__measure",
                    "aria-pressed": if tokens_is_measure { "true" } else { "false" },
                    onclick: move |_| set_measure(Measure::Tokens),
                    "Tokens"
                }
            }
            div { class: "kit-rcb__totals",
                span { class: "kit-rcb__total-cost", "{total_text}" }
                span { class: "kit-rcb__total-tokens", "{tokens_text}" }
                if let Some((cid, label)) = costliest_view {
                    button {
                        r#type: "button",
                        class: "kit-rcb__costliest",
                        onclick: move |_| {
                            let mut next = costly_folded.clone();
                            next.remove(&cid);
                            costly_fold(next.clone());
                            costly_detail(Some(cid.clone()));
                            let index = flatten(&costly_steps, &next)
                                .iter()
                                .position(|r| r.step.step_id == cid)
                                .unwrap_or(0);
                            focus_element(&format!("{keys_base_for_costliest}-row-{index}"));
                        },
                        "{label}"
                    }
                }
            }
            if step_list.is_empty() {
                p { class: "kit-rcb__empty", "No steps yet" }
            } else {
                ul {
                    class: "kit-rcb__list",
                    role: "tree",
                    "aria-label": "{heading}",
                    style: list_style,
                    onkeydown: list_keys,
                    {list_elements.into_iter()}
                }
            }
        }
    }
}
