//! Parallel span timeline: several workers and the units of work each has
//! done or is doing, on one shared time axis that ends at the present. A
//! person sees who is busy, who is unassigned, and which unit is held for a
//! decision.
//!
//! Design choice: no wall clock, since `SystemTime` panics on
//! `wasm32-unknown-unknown`. Without `clock_ms`, the present starts at the
//! latest unit time on mount and advances one second per tick.
//! Design choice: `tool_marks` icons are not taken; the tooltip shows the
//! highlights only.
//! Design choice: worker marks are the first letter of the label.
//! Design choice: without `on_confirm` or `on_decline`, the component sets the
//! held unit to `settled` or `broken` itself. The source does not document a
//! decline with no handler; `broken` is chosen here.
//! Design choice: a unit that ends after the present is drawn to the present.
//! Browser checks: segments lengthen with the clock; the window slides and
//! clipped units leave at the left edge; the decision popover, its focus and
//! Escape; hand-off curves; hover and focus tooltips; narrow containers hiding
//! worker labels; reduced motion stops the pulse and the live indicator.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, focus_after_frame, next_id, sleep};

const CSS: Asset = asset!("/assets/parallel-span-timeline.css");

/// Height of one worker row in the connector drawing units.
const ROW_H: f64 = 48.0;
/// Highlights shown per unit, at most.
const HIGHLIGHT_LIMIT: usize = 3;
/// Default window: ten minutes before the present.
const DEFAULT_WINDOW_MS: u64 = 600_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitPhase {
    Settled,
    Active,
    Held,
    Broken,
}

impl UnitPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Settled => "settled",
            Self::Active => "active",
            Self::Held => "held",
            Self::Broken => "broken",
        }
    }

    fn word(self) -> &'static str {
        match self {
            Self::Settled => "Settled",
            Self::Active => "Active",
            Self::Held => "Held",
            Self::Broken => "Broken",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Confirm,
    Decline,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorkerRow {
    pub worker_key: String,
    pub worker_label: String,
    pub model_label: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Unit {
    pub unit_key: String,
    pub worker_key: String,
    pub unit_label: String,
    pub phase: UnitPhase,
    pub begins_at_ms: u64,
    /// `None` means the unit reaches the present.
    pub ends_at_ms: Option<u64>,
    pub hands_off_to_key: Option<String>,
    pub decision_prompt: Option<String>,
    pub highlights: Vec<String>,
}

/// Where a unit sits on the axis, as percentages of the window.
#[derive(Clone, Debug, PartialEq)]
pub struct Placement {
    pub left: f64,
    pub width: f64,
    pub clipped: bool,
}

/// The phase a unit shows: a host-supplied phase is replaced by the
/// component's own outcome when a decision was made without a handler.
pub fn effective_phase(unit: &Unit, overrides: &[(String, UnitPhase)]) -> UnitPhase {
    overrides
        .iter()
        .find(|(key, _)| *key == unit.unit_key)
        .map_or(unit.phase, |(_, phase)| *phase)
}

/// The phase a decision leaves a unit in when the host supplied no handler.
/// `None` when the host decides.
pub fn decision_outcome(decision: Decision, has_handler: bool) -> Option<UnitPhase> {
    if has_handler {
        return None;
    }
    Some(match decision {
        Decision::Confirm => UnitPhase::Settled,
        Decision::Decline => UnitPhase::Broken,
    })
}

/// Percent of the window at which time `t` falls, clamped to the window.
pub fn position_pct(t: u64, window_start: u64, span: u64) -> f64 {
    if span == 0 {
        return 0.0;
    }
    (t.saturating_sub(window_start) as f64 / span as f64 * 100.0).clamp(0.0, 100.0)
}

/// Where a unit sits in the window ending at `now`. `None` when it lies wholly
/// outside the window or after the present.
pub fn placement(unit: &Unit, now: u64, span: u64) -> Option<Placement> {
    let start = now.saturating_sub(span);
    let end = unit.ends_at_ms.unwrap_or(now).min(now);
    if unit.begins_at_ms > now || end <= start {
        return None;
    }
    let left = position_pct(unit.begins_at_ms.max(start), start, span);
    let right = position_pct(end, start, span);
    Some(Placement {
        left,
        width: (right - left).max(0.0),
        clipped: unit.begins_at_ms < start,
    })
}

/// The units of one worker, earliest first. This is also the tab order within a row.
pub fn row_units(units: &[Unit], worker_key: &str) -> Vec<Unit> {
    let mut mine: Vec<Unit> = units
        .iter()
        .filter(|u| u.worker_key == worker_key)
        .cloned()
        .collect();
    mine.sort_by_key(|u| u.begins_at_ms);
    mine
}

/// `(active, held)` counts, from the phases the units show.
pub fn header_counts(units: &[Unit], overrides: &[(String, UnitPhase)]) -> (usize, usize) {
    units.iter().fold((0, 0), |(active, held), u| {
        match effective_phase(u, overrides) {
            UnitPhase::Active => (active + 1, held),
            UnitPhase::Held => (active, held + 1),
            _ => (active, held),
        }
    })
}

/// The polite line for the held count.
pub fn held_announcement(held: usize) -> String {
    match held {
        0 => "No units waiting for input.".to_string(),
        1 => "1 unit waiting for input.".to_string(),
        n => format!("{n} units waiting for input."),
    }
}

/// `"now"`, `"Ns ago"`, `"Nm ago"` or `"Nh ago"`.
pub fn format_ago(ms: u64) -> String {
    let secs = ms / 1000;
    if secs == 0 {
        "now".to_string()
    } else if secs < 60 {
        format!("{secs}s ago")
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else {
        format!("{}h ago", secs / 3600)
    }
}

/// `"from 6m ago to now"` for a unit's time range.
pub fn range_text(unit: &Unit, now: u64) -> String {
    let end = unit.ends_at_ms.unwrap_or(now).min(now);
    format!(
        "from {} to {}",
        format_ago(now.saturating_sub(unit.begins_at_ms)),
        format_ago(now.saturating_sub(end))
    )
}

/// Axis ticks as `(percent, label)`, with the present at the right edge.
pub fn axis_ticks(span: u64, divisions: u64) -> Vec<(f64, String)> {
    (0..=divisions)
        .map(|k| {
            let pct = k as f64 / divisions as f64 * 100.0;
            let ago = span - span * k / divisions;
            (pct, format_ago(ago))
        })
        .collect()
}

/// A hand-off curve from `(from_x, from_row)` to `(to_x, to_row)`, in
/// connector units: `x` is percent of the track, `row` is the worker row.
pub fn connector_path(from: (f64, usize), to: (f64, usize)) -> String {
    let y1 = from.1 as f64 * ROW_H + ROW_H / 2.0;
    let y2 = to.1 as f64 * ROW_H + ROW_H / 2.0;
    let mid = (from.0 + to.0) / 2.0;
    format!(
        "M {:.2} {y1:.1} C {mid:.2} {y1:.1} {mid:.2} {y2:.1} {:.2} {y2:.1}",
        from.0, to.0
    )
}

fn initial_of(label: &str) -> String {
    label
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_default()
}

fn slug(key: &str) -> String {
    key.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

#[derive(Clone, Debug, PartialEq)]
struct UnitView {
    unit: Unit,
    label: String,
    unit_id: String,
    confirm_id: String,
    phase_key: &'static str,
    phase_word: &'static str,
    left: String,
    width: String,
    clipped: bool,
    live: bool,
    held: bool,
    expanded: Option<&'static str>,
    open: bool,
    tip: bool,
    aria: String,
    range: String,
    highlights: Vec<String>,
    prompt: String,
}

#[derive(Clone, Debug, PartialEq)]
struct RowView {
    worker_key: String,
    label: String,
    model: Option<String>,
    mark: String,
    unassigned: bool,
    has_held: bool,
    units: Vec<UnitView>,
}

/// Several workers and their units on one time axis, with hand-offs and
/// decisions for units that wait on the person.
#[component]
pub fn ParallelSpanTimeline(
    worker_rows: Vec<WorkerRow>,
    units: Vec<Unit>,
    #[props(default = DEFAULT_WINDOW_MS)] window_span: u64,
    #[props(default = "Workers".to_string(), into)] heading: String,
    #[props(default)] clock_ms: Option<u64>,
    #[props(default)] on_confirm: Option<EventHandler<Unit>>,
    #[props(default)] on_decline: Option<EventHandler<Unit>>,
    #[props(default)] on_details: Option<EventHandler<Unit>>,
    #[props(default)] extra_class: String,
) -> Element {
    let uid = use_hook(next_id);
    let heading_id = format!("kit-span-{uid}-head");
    let initial_now = units
        .iter()
        .map(|u| u.ends_at_ms.unwrap_or(u.begins_at_ms).max(u.begins_at_ms))
        .max()
        .unwrap_or(0);
    let mut local_now = use_signal(|| initial_now);
    let mut overrides = use_signal(Vec::<(String, UnitPhase)>::new);
    let mut open_decision = use_signal(|| None::<String>);
    let mut tip = use_signal(|| None::<String>);
    let mut announce = use_signal(String::new);
    let (_, held_at_mount) = header_counts(&units, &[]);
    let mut seen_held = use_signal(|| held_at_mount);

    use_future(move || async move {
        loop {
            sleep(1000).await;
            if clock_ms.is_none() {
                let next = *local_now.peek() + 1000;
                local_now.set(next);
            }
        }
    });

    let now = clock_ms.unwrap_or_else(|| local_now());
    let current = overrides();
    let (active, held) = header_counts(&units, &current);
    let open_key = open_decision();
    let tip_key = tip();
    let class = class_list("kit-span", &extra_class);
    let has_details = on_details.is_some();

    use_effect(use_reactive!(|(held,)| {
        if *seen_held.peek() != held {
            seen_held.set(held);
            announce.set(held_announcement(held));
        }
    }));

    let window_text = if window_span >= 60_000 {
        format!("Last {} min", window_span / 60_000)
    } else {
        format!("Last {} s", window_span / 1000)
    };
    let active_text = format!("{active} active");
    let held_text = format!("{held} held");
    let line = announce();
    let ticks: Vec<(String, String)> = axis_ticks(window_span, 4)
        .into_iter()
        .map(|(pct, label)| (format!("{pct:.2}"), label))
        .collect();

    let rows: Vec<RowView> = worker_rows
        .iter()
        .map(|worker| {
            let mine = row_units(&units, &worker.worker_key);
            let views: Vec<UnitView> = mine
                .iter()
                .filter_map(|unit| {
                    let placed = placement(unit, now, window_span)?;
                    let phase = effective_phase(unit, &current);
                    let unit_id = format!("kit-span-{uid}-{}", slug(&unit.unit_key));
                    let range = range_text(unit, now);
                    let aria = format!(
                        "{}, {}, {}, {}",
                        worker.worker_label,
                        unit.unit_label,
                        phase.word(),
                        range
                    );
                    let held_unit = phase == UnitPhase::Held;
                    let open = open_key.as_deref() == Some(unit.unit_key.as_str());
                    Some(UnitView {
                        confirm_id: format!("{unit_id}-confirm"),
                        unit_id,
                        phase_key: phase.as_str(),
                        phase_word: phase.word(),
                        left: format!("{:.2}%", placed.left),
                        width: format!("{:.2}%", placed.width),
                        clipped: placed.clipped,
                        live: phase == UnitPhase::Active,
                        held: held_unit,
                        expanded: held_unit.then_some(if open { "true" } else { "false" }),
                        open: held_unit && open,
                        tip: tip_key.as_deref() == Some(unit.unit_key.as_str())
                            && !(held_unit && open),
                        aria,
                        range,
                        highlights: unit
                            .highlights
                            .iter()
                            .take(HIGHLIGHT_LIMIT)
                            .cloned()
                            .collect(),
                        prompt: unit.decision_prompt.clone().unwrap_or_default(),
                        label: unit.unit_label.clone(),
                        unit: unit.clone(),
                    })
                })
                .collect();
            let has_held = views.iter().any(|v| v.held);
            RowView {
                worker_key: worker.worker_key.clone(),
                label: worker.worker_label.clone(),
                model: worker.model_label.clone(),
                mark: initial_of(&worker.worker_label),
                unassigned: views.is_empty(),
                has_held,
                units: views,
            }
        })
        .collect();

    let row_index = |key: &str| rows.iter().position(|r| r.worker_key == key);
    let connectors: Vec<String> = units
        .iter()
        .filter_map(|from| {
            let to_key = from.hands_off_to_key.as_ref()?;
            let to = units.iter().find(|u| &u.unit_key == to_key)?;
            let from_row = row_index(&from.worker_key)?;
            let to_row = row_index(&to.worker_key)?;
            let from_placed = placement(from, now, window_span)?;
            let to_placed = placement(to, now, window_span)?;
            let from_x = from_placed.left + from_placed.width;
            Some(connector_path((from_x, from_row), (to_placed.left, to_row)))
        })
        .collect();
    let total_h = format!("0 0 100 {}", rows.len() as f64 * ROW_H);
    let rows_h = format!("{}px", rows.len() as f64 * ROW_H);

    let decide = move |(unit, decision): (Unit, Decision)| {
        let handler = match decision {
            Decision::Confirm => on_confirm,
            Decision::Decline => on_decline,
        };
        let unit_id = format!("kit-span-{uid}-{}", slug(&unit.unit_key));
        match handler {
            Some(handler) => handler.call(unit),
            None => {
                if let Some(phase) = decision_outcome(decision, false) {
                    let key = unit.unit_key.clone();
                    overrides.with_mut(|list| {
                        list.retain(|(k, _)| *k != key);
                        list.push((key, phase));
                    });
                }
            }
        }
        open_decision.set(None);
        focus_after_frame(&unit_id);
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class, "aria-labelledby": "{heading_id}",
            header { class: "kit-span__head",
                h2 { id: "{heading_id}", class: "kit-span__heading", "{heading}" }
                p { class: "kit-span__counts",
                    span { class: "kit-span__count", "{active_text}" }
                    span { class: "kit-span__count kit-span__count--held", "{held_text}" }
                    span { class: "kit-span__window", "{window_text}" }
                }
            }
            p { class: "kit-span__sr", role: "status", "aria-live": "polite", "{line}" }
            div { class: "kit-span__board",
                div { class: "kit-span__axis", "aria-hidden": "true",
                    for (pct, label) in ticks {
                        span { class: "kit-span__tick", style: "left: {pct}%", "{label}" }
                    }
                }
                div { class: "kit-span__rows", style: "min-height: {rows_h}",
                    svg {
                        class: "kit-span__links",
                        "aria-hidden": "true",
                        view_box: "{total_h}",
                        preserve_aspect_ratio: "none",
                        for d in connectors {
                            path { class: "kit-span__link", d: "{d}" }
                        }
                    }
                    for RowView { worker_key, label, model, mark, unassigned, has_held, units: views } in rows {
                        div {
                            class: "kit-span__row",
                            role: "group",
                            "aria-label": "{label}",
                            "data-worker": "{worker_key}",
                            div { class: "kit-span__id",
                                span { class: "kit-span__mark", "aria-hidden": "true", "{mark}" }
                                span { class: "kit-span__label", "{label}" }
                                if let Some(model) = model {
                                    span { class: "kit-span__model", "{model}" }
                                }
                                if unassigned {
                                    span { class: "kit-span__tag", "Unassigned" }
                                }
                                if has_held {
                                    span { class: "kit-span__tag kit-span__tag--held", "Input needed" }
                                }
                            }
                            div { class: "kit-span__track",
                                for UnitView {
                                    unit, label, unit_id, confirm_id, phase_key, phase_word, left, width,
                                    clipped, live, held, expanded, open, tip: show_tip, aria, range,
                                    highlights, prompt,
                                } in views
                                {
                                    button {
                                        r#type: "button",
                                        id: "{unit_id}",
                                        class: "kit-span__unit",
                                        "data-phase": phase_key,
                                        "data-clipped": clipped.then_some("true"),
                                        style: "left: {left}; width: {width}",
                                        "aria-label": "{aria}",
                                        "aria-expanded": expanded,
                                        "aria-haspopup": held.then_some("dialog"),
                                        onmouseenter: {
                                            let key = unit.unit_key.clone();
                                            move |_| tip.set(Some(key.clone()))
                                        },
                                        onmouseleave: move |_| tip.set(None),
                                        onfocus: {
                                            let key = unit.unit_key.clone();
                                            move |_| tip.set(Some(key.clone()))
                                        },
                                        onblur: move |_| tip.set(None),
                                        onclick: {
                                            let key = unit.unit_key.clone();
                                            let is_held = held;
                                            let confirm = confirm_id.clone();
                                            move |_| {
                                                if is_held {
                                                    open_decision.set(Some(key.clone()));
                                                    focus_after_frame(&confirm);
                                                }
                                            }
                                        },
                                        if live {
                                            span { class: "kit-span__live", "aria-hidden": "true" }
                                        }
                                        if held {
                                            span { class: "kit-span__decision", "aria-hidden": "true" }
                                        }
                                        span { class: "kit-span__sr", "{phase_word}" }
                                    }
                                    if open {
                                        div {
                                            class: "kit-span__popover",
                                            role: "dialog",
                                            "aria-label": "Decision",
                                            style: "left: {left}",
                                            onkeydown: {
                                                let unit_id = unit_id.clone();
                                                move |event: KeyboardEvent| {
                                                    if event.key() == Key::Escape {
                                                        event.prevent_default();
                                                        open_decision.set(None);
                                                        focus_after_frame(&unit_id);
                                                    }
                                                }
                                            },
                                            p { class: "kit-span__prompt", "{prompt}" }
                                            div { class: "kit-span__decide",
                                                button {
                                                    r#type: "button",
                                                    id: "{confirm_id}",
                                                    class: "kit-span__confirm",
                                                    onclick: {
                                                        let unit = unit.clone();
                                                        move |_| {
                                                            let mut decide = decide;
                                                            decide((unit.clone(), Decision::Confirm))
                                                        }
                                                    },
                                                    "Confirm"
                                                }
                                                button {
                                                    r#type: "button",
                                                    class: "kit-span__decline",
                                                    onclick: {
                                                        let unit = unit.clone();
                                                        move |_| {
                                                            let mut decide = decide;
                                                            decide((unit.clone(), Decision::Decline))
                                                        }
                                                    },
                                                    "Decline"
                                                }
                                            }
                                        }
                                    }
                                    if show_tip && !open {
                                        div { class: "kit-span__tip", role: "tooltip", style: "left: {left}",
                                            p { class: "kit-span__tip-range", "{range}" }
                                            if !held {
                                                ul { class: "kit-span__highlights",
                                                    for highlight in highlights {
                                                        li { "{highlight}" }
                                                    }
                                                }
                                            }
                                            if has_details && !held {
                                                button {
                                                    r#type: "button",
                                                    class: "kit-span__details",
                                                    "aria-label": "Details: {label}",
                                                    onclick: {
                                                        let unit = unit.clone();
                                                        move |_| {
                                                            if let Some(handler) = on_details {
                                                                handler.call(unit.clone());
                                                            }
                                                        }
                                                    },
                                                    "Details"
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
        }
    }
}
