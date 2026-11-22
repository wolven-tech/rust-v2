//! Deliberation digest: one compact card condensing how an agent spent its
//! effort on an answer, by kind of work, by model, by source and by step.
//!
//! Design choice: times are seconds as `f64`; a NaN or negative time counts as
//! zero, so malformed upstream values never break the total.
//! Design choice: `kind_table` entries are merged over the built-in kinds, so
//! a caller can add or override a kind without restating the others.
//! Design choice: logos are image URLs; without one the first letter is shown.
//! Design choice: the live announcement is raised once, when reasoning stops.
//! Browser checks: the sheen on the running segment, the growth of the card when
//! a list opens, the fade of unrelated items on hover or focus, and the role
//! being hidden at narrow width.

use std::collections::BTreeMap;

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id, use_now_ms};

const CSS: Asset = asset!("/assets/deliberation-digest.css");

#[derive(Clone, PartialEq, Debug)]
pub struct DigestStep {
    pub step_id: String,
    pub kind_key: String,
    pub step_label: String,
    /// Seconds; omitted while the step runs.
    pub elapsed_time: Option<f64>,
    pub is_running: bool,
    pub model_id: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Effort {
    Low,
    Medium,
    High,
    Max,
}

impl Effort {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Max => "max",
        }
    }

    /// Lit bars out of four.
    pub fn bars(self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
            Self::Max => 4,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TokenCounts {
    pub input: u64,
    pub output: u64,
}

#[derive(Clone, PartialEq, Debug)]
pub struct DigestModel {
    pub model_id: String,
    pub display_name: String,
    pub logo: Option<String>,
    pub role_label: String,
    pub effort_level: Effort,
    /// Seconds.
    pub time_taken: f64,
    pub provider_name: String,
    pub token_counts: Option<TokenCounts>,
    pub spend: Option<f64>,
    pub selection_note: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct DigestSource {
    pub source_name: String,
    pub source_detail: Option<String>,
    pub source_href: Option<String>,
    pub logo: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Emphasis {
    Quiet,
    #[default]
    Normal,
    Strong,
}

impl Emphasis {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Quiet => "quiet",
            Self::Normal => "normal",
            Self::Strong => "strong",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct KindSpec {
    pub kind_label: String,
    pub kind_emphasis: Emphasis,
    pub kind_mark: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct AgentIdentity {
    pub agent_id: String,
    pub agent_emphasis: Emphasis,
}

#[derive(Clone, PartialEq, Debug)]
pub struct KindTotal {
    pub key: String,
    pub label: String,
    pub mark: Option<String>,
    pub emphasis: Emphasis,
    pub total: f64,
    pub count: usize,
}

/// What the pointer or keyboard is on; related items stay clear, others fade.
#[derive(Clone, PartialEq, Debug)]
pub enum Focus {
    None,
    Kind(String),
    Model(String),
}

/// Built-in kinds of work, overridable through `kind_table`.
pub fn builtin_kinds() -> BTreeMap<String, KindSpec> {
    let entries = [
        ("reasoning", "Reasoning", Emphasis::Normal, "◇"),
        ("search", "Searching", Emphasis::Normal, "⌕"),
        ("read", "Reading", Emphasis::Quiet, "▤"),
        ("write", "Writing", Emphasis::Strong, "✎"),
        ("tool", "Tool calls", Emphasis::Normal, "⚙"),
        ("check", "Checking", Emphasis::Normal, "✓"),
    ];
    entries
        .into_iter()
        .map(|(key, label, emphasis, mark)| {
            (
                key.to_string(),
                KindSpec {
                    kind_label: label.to_string(),
                    kind_emphasis: emphasis,
                    kind_mark: Some(mark.to_string()),
                },
            )
        })
        .collect()
}

/// Built-in kinds with the caller's entries laid over them.
pub fn merged_kinds(overrides: &BTreeMap<String, KindSpec>) -> BTreeMap<String, KindSpec> {
    let mut table = builtin_kinds();
    for (key, spec) in overrides {
        table.insert(key.clone(), spec.clone());
    }
    table
}

/// Non-numeric, NaN and negative times count as zero.
pub fn sanitize_time(value: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        0.0
    }
}

pub fn total_time(steps: &[DigestStep]) -> f64 {
    steps
        .iter()
        .map(|s| sanitize_time(s.elapsed_time.unwrap_or(0.0)))
        .sum()
}

/// Segment widths in percent of the bar, one per step, plus a trailing remainder
/// when `expected` exceeds the elapsed total.
pub fn segment_widths(steps: &[DigestStep], expected: Option<f64>) -> (Vec<f64>, f64) {
    let total = total_time(steps);
    let expected = expected.map(sanitize_time).unwrap_or(0.0);
    let span = total.max(expected);
    if span <= 0.0 {
        return (vec![0.0; steps.len()], 0.0);
    }
    let widths = steps
        .iter()
        .map(|s| sanitize_time(s.elapsed_time.unwrap_or(0.0)) / span * 100.0)
        .collect();
    let remainder = if expected > total {
        (expected - total) / span * 100.0
    } else {
        0.0
    };
    (widths, remainder)
}

/// Totals per kind in first-appearance order. Unknown kinds get their key as name and no mark.
pub fn kind_totals(steps: &[DigestStep], table: &BTreeMap<String, KindSpec>) -> Vec<KindTotal> {
    let mut out: Vec<KindTotal> = Vec::new();
    for step in steps {
        let time = sanitize_time(step.elapsed_time.unwrap_or(0.0));
        match out.iter_mut().find(|k| k.key == step.kind_key) {
            Some(entry) => {
                entry.total += time;
                entry.count += 1;
            }
            None => {
                let spec = table.get(&step.kind_key);
                out.push(KindTotal {
                    key: step.kind_key.clone(),
                    label: spec
                        .map(|s| s.kind_label.clone())
                        .unwrap_or_else(|| step.kind_key.clone()),
                    mark: spec.and_then(|s| s.kind_mark.clone()),
                    emphasis: spec.map(|s| s.kind_emphasis).unwrap_or_default(),
                    total: time,
                    count: 1,
                });
            }
        }
    }
    out
}

fn plural(n: u64) -> &'static str {
    if n == 1 { "" } else { "s" }
}

/// `11 seconds`, `1 minute`, `2 min 5 s`.
pub fn format_duration(seconds: f64) -> String {
    let total = sanitize_time(seconds).round() as u64;
    if total < 60 {
        format!("{total} second{}", plural(total))
    } else {
        let minutes = total / 60;
        let rest = total % 60;
        if rest == 0 {
            format!("{minutes} minute{}", plural(minutes))
        } else {
            format!("{minutes} min {rest} s")
        }
    }
}

/// `Working` while running; `Took 14 seconds across 6 steps` when finished.
pub fn header_text(is_reasoning: bool, total: f64, steps: usize) -> String {
    if is_reasoning {
        "Working".to_string()
    } else {
        format!(
            "Took {} across {steps} step{}",
            format_duration(total),
            plural(steps as u64)
        )
    }
}

/// Grouping separators below ten thousand, compact units above.
pub fn format_count(value: u64) -> String {
    if value >= 1_000_000 {
        format!("{:.1}M", value as f64 / 1_000_000.0)
    } else if value >= 10_000 {
        format!("{:.1}k", value as f64 / 1_000.0)
    } else {
        let digits = value.to_string();
        let mut out = String::new();
        for (i, c) in digits.chars().enumerate() {
            if i > 0 && (digits.len() - i).is_multiple_of(3) {
                out.push(',');
            }
            out.push(c);
        }
        out
    }
}

pub fn is_related(step: &DigestStep, focus: &Focus) -> bool {
    match focus {
        Focus::None => true,
        Focus::Kind(key) => &step.kind_key == key,
        Focus::Model(id) => step.model_id.as_deref() == Some(id.as_str()),
    }
}

/// The first letter of a name, uppercased; empty for an empty name.
pub fn initial(name: &str) -> String {
    name.chars()
        .next()
        .map(|c| c.to_uppercase().collect())
        .unwrap_or_default()
}

fn focus_id(id: &str) {
    document::eval(&format!("document.getElementById({id:?})?.focus();"));
}

/// The digest card.
#[component]
pub fn DeliberationDigest(
    #[props(default)] step_list: Vec<DigestStep>,
    #[props(default)] model_list: Vec<DigestModel>,
    #[props(default)] source_list: Vec<DigestSource>,
    #[props(default)] total_source_count: Option<usize>,
    #[props(default)] is_reasoning: bool,
    #[props(default)] expected_time: Option<f64>,
    #[props(default)] kind_table: BTreeMap<String, KindSpec>,
    #[props(default)] agent_identity: Option<AgentIdentity>,
    #[props(default)] start_open: bool,
    /// Epoch ms of the running step's start. Its elapsed time is `clock_ms` minus this.
    #[props(default)]
    running_started_ms: Option<u64>,
    #[props(default)] clock_ms: Option<u64>,
    #[props(default, into)] root_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-digest-{}", next_id()));
    let now_ms = use_now_ms(clock_ms, 250);
    let mut open_model = use_signal(|| Option::<String>::None);
    let mut sources_open = use_signal(|| false);
    let mut steps_open = use_signal(move || start_open);
    let mut focus = use_signal(|| Focus::None);
    let mut focused_kind = use_signal(|| Option::<usize>::None);
    let mut focused_model = use_signal(|| Option::<usize>::None);
    let mut announce = use_signal(String::new);
    let mut was_reasoning = use_signal(|| is_reasoning);

    let table = merged_kinds(&kind_table);
    let live_steps: Vec<DigestStep> = step_list
        .iter()
        .map(|step| {
            let mut live = step.clone();
            if let (true, Some(started)) = (step.is_running, running_started_ms) {
                live.elapsed_time = Some(now_ms.saturating_sub(started) as f64 / 1000.0);
            }
            live
        })
        .collect();
    let total = total_time(&live_steps);
    let (widths, remainder) = segment_widths(&live_steps, expected_time);
    let kinds = kind_totals(&live_steps, &table);
    let header = header_text(is_reasoning, total, step_list.len());
    let current_focus = focus();
    let model_open = open_model();
    let sources_expanded = if sources_open() { "true" } else { "false" };
    let steps_expanded = if steps_open() { "true" } else { "false" };
    let root_cls = class_list("kit-digest", &root_class);
    let sources_count = total_source_count.unwrap_or(source_list.len());
    let sources_label = format!("References ({sources_count})");
    let steps_label = format!("Step log ({})", step_list.len());
    let agent_mark = agent_identity.as_ref().map(|a| {
        (
            format!("kit-digest__agent is-{}", a.agent_emphasis.as_str()),
            initial(&a.agent_id),
        )
    });
    let total_text = format_duration(total);
    let remainder_style = format!("width: {remainder:.2}%;");
    let detail_id = format!("{base}-detail");
    let open_detail = model_open
        .as_deref()
        .and_then(|open| model_list.iter().find(|m| m.model_id == open))
        .map(|model| {
            (
                model.provider_name.clone(),
                model.token_counts.map(|t| {
                    format!(
                        "in {} · out {}",
                        format_count(t.input),
                        format_count(t.output)
                    )
                }),
                model.spend.map(|s| format!("${s:.2}")),
                model.selection_note.clone(),
            )
        });
    let open_name = model_open
        .as_deref()
        .and_then(|open| model_list.iter().find(|m| m.model_id == open))
        .map(|m| m.display_name.clone())
        .unwrap_or_default();
    let announcement = announce();
    let reasoning_now = is_reasoning;
    let summary_now = header.clone();

    use_effect(use_reactive(
        (&reasoning_now, &summary_now),
        move |(now, summary)| {
            if *was_reasoning.peek() && !now {
                announce.set(summary);
            }
            was_reasoning.set(now);
        },
    ));

    let kind_nav = {
        let focus_ids: Vec<String> = (0..kinds.len())
            .map(|i| format!("{base}-kind-{i}"))
            .collect();
        let kinds_keys: Vec<String> = kinds.iter().map(|k| k.key.clone()).collect();
        move |event: KeyboardEvent| {
            let step: isize = match event.key() {
                Key::ArrowRight | Key::ArrowDown => 1,
                Key::ArrowLeft | Key::ArrowUp => -1,
                Key::Home => {
                    event.prevent_default();
                    focus.set(Focus::None);
                    focus_id(&focus_ids[0]);
                    return;
                }
                Key::End => {
                    event.prevent_default();
                    focus_id(&focus_ids[focus_ids.len() - 1]);
                    return;
                }
                Key::Escape => {
                    focus.set(Focus::None);
                    return;
                }
                _ => return,
            };
            event.prevent_default();
            let current = focused_kind().unwrap_or(0) as isize;
            let next = (current + step).clamp(0, focus_ids.len() as isize - 1) as usize;
            focused_kind.set(Some(next));
            focus.set(Focus::Kind(kinds_keys[next].clone()));
            focus_id(&focus_ids[next]);
        }
    };

    let model_nav = {
        let ids: Vec<String> = (0..model_list.len())
            .map(|i| format!("{base}-model-{i}"))
            .collect();
        let model_keys: Vec<String> = model_list.iter().map(|m| m.model_id.clone()).collect();
        move |event: KeyboardEvent| {
            let step: isize = match event.key() {
                Key::ArrowRight | Key::ArrowDown => 1,
                Key::ArrowLeft | Key::ArrowUp => -1,
                Key::Home | Key::End => {
                    event.prevent_default();
                    let index = if matches!(event.key(), Key::Home) {
                        0
                    } else {
                        ids.len().saturating_sub(1)
                    };
                    if let Some(id) = ids.get(index) {
                        focus_id(id);
                    }
                    return;
                }
                Key::Escape => {
                    open_model.set(None);
                    return;
                }
                _ => return,
            };
            event.prevent_default();
            if ids.is_empty() {
                return;
            }
            let current = focused_model().unwrap_or(0) as isize;
            let next = (current + step).clamp(0, ids.len() as isize - 1) as usize;
            focused_model.set(Some(next));
            focus.set(Focus::Model(model_keys[next].clone()));
            focus_id(&ids[next]);
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: root_cls, "aria-label": "How the answer was reasoned",
            onkeydown: move |event: KeyboardEvent| {
                if matches!(event.key(), Key::Escape) {
                    open_model.set(None);
                    sources_open.set(false);
                    steps_open.set(false);
                    focus.set(Focus::None);
                }
            },
            header { class: "kit-digest__head",
                if let Some((class, letter)) = agent_mark.clone() {
                    span { class: "{class}", "aria-hidden": "true", "{letter}" }
                }
                h2 { class: if is_reasoning { "kit-digest__title is-running" } else { "kit-digest__title" }, "{header}" }
                if is_reasoning {
                    span { class: "kit-digest__pulse", "aria-hidden": "true" }
                }
            }
            div { class: "kit-digest__bar", role: "img", "aria-label": "Time per step, {total_text}",
                for (index, (width, step)) in widths.iter().copied().zip(live_steps.iter()).enumerate() {
                    {
                        let tone = kinds.iter().position(|k| k.key == step.kind_key).unwrap_or(0) % 6 + 1;
                        let style = format!("width: {width:.2}%; --seg-t: var(--kit-series-{tone});");
                        let kind = step.kind_key.clone();
                        let title = format!("{}: {}", step.step_label, format_duration(step.elapsed_time.unwrap_or(0.0)));
                        let related = is_related(step, &current_focus);
                        let related_attr = if related { "true" } else { "false" };
                        let model = step.model_id.clone();
                        let running = if step.is_running { "true" } else { "false" };
                        let key = format!("{base}-seg-{index}");
                        rsx! {
                            span {
                                key: "{key}",
                                class: "kit-digest__seg",
                                style: "{style}",
                                "data-kind": "{kind}",
                                "data-related": "{related_attr}",
                                "data-running": "{running}",
                                title: "{title}",
                                onmouseenter: move |_| {
                                    if let Some(id) = model.clone() {
                                        focus.set(Focus::Model(id));
                                    }
                                },
                                onmouseleave: move |_| focus.set(Focus::None),
                            }
                        }
                    }
                }
                if remainder > 0.0 {
                    span { class: "kit-digest__seg is-remainder", style: "{remainder_style}" }
                }
            }
            div { class: "kit-digest__kinds", role: "group", "aria-label": "Kinds of work",
                onkeydown: kind_nav,
                for (index, kind) in kinds.iter().enumerate() {
                    {
                        let key = kind.key.clone();
                        let cell_id = format!("{base}-kind-{index}");
                        let label = kind.label.clone();
                        let mark = kind.mark.clone().unwrap_or_else(|| "■".to_string());
                        let time = format_duration(kind.total);
                        let emphasis = kind.emphasis.as_str();
                        let hover_key = key.clone();
                        let focus_key = key.clone();
                        let kind_name = format!("{label}, {time}, {} steps", kind.count);
                        let swatch_tone = index % 6 + 1;
                        let kind_style = format!("--dig-t: var(--kit-series-{swatch_tone});");
                        rsx! {
                            button {
                                key: "{cell_id}",
                                id: "{cell_id}",
                                r#type: "button",
                                class: "kit-digest__kind",
                                style: "{kind_style}",
                                "data-kind": "{key}",
                                "data-emphasis": "{emphasis}",
                                "aria-label": "{kind_name}",
                                onmouseenter: move |_| focus.set(Focus::Kind(hover_key.clone())),
                                onmouseleave: move |_| focus.set(Focus::None),
                                onfocus: move |_| {
                                    focused_kind.set(Some(index));
                                    focus.set(Focus::Kind(focus_key.clone()));
                                },
                                onblur: move |_| focus.set(Focus::None),
                                span { class: "kit-digest__kind-mark", "aria-hidden": "true", "{mark}" }
                                span { class: "kit-digest__kind-label", "{label}" }
                                span { class: "kit-digest__kind-time", "{time}" }
                            }
                        }
                    }
                }
            }
            div { class: "kit-digest__models", role: "group", "aria-label": "Models",
                onkeydown: model_nav,
                for (index, model) in model_list.iter().enumerate() {
                    {
                        let id = model.model_id.clone();
                        let row_id = format!("{base}-model-{index}");
                        let expanded = model_open.as_deref() == Some(id.as_str());
                        let expanded_attr = if expanded { "true" } else { "false" };
                        let effort = model.effort_level;
                        let bars: Vec<bool> = (1..=4).map(|n| n <= effort.bars()).collect();
                        let effort_name = format!("{} effort", effort.as_str());
                        let time = format_duration(model.time_taken);
                        let name_label = format!(
                            "{}, {}, {}, {effort_name}",
                            model.display_name, model.role_label, time
                        );
                        let logo_letter = initial(&model.display_name);
                        let logo = model.logo.clone();
                        let hover_id = id.clone();
                        let click_id = id.clone();
                        let row_class = if expanded { "kit-digest__model is-open" } else { "kit-digest__model" };
                        let role = model.role_label.clone();
                        let name = model.display_name.clone();
                        let focus_model = id.clone();
                        rsx! {
                            div { key: "{id}", class: row_class,
                                button {
                                    id: "{row_id}",
                                    r#type: "button",
                                    class: "kit-digest__model-row",
                                    "aria-label": "{name_label}",
                                    "aria-expanded": "{expanded_attr}",
                                    "aria-controls": "{detail_id}",
                                    onmouseenter: move |_| focus.set(Focus::Model(hover_id.clone())),
                                    onmouseleave: move |_| focus.set(Focus::None),
                                    onfocus: move |_| {
                                        focused_model.set(Some(index));
                                        focus.set(Focus::Model(focus_model.clone()));
                                    },
                                    onclick: move |_| {
                                        if open_model().as_deref() == Some(click_id.as_str()) {
                                            open_model.set(None);
                                        } else {
                                            open_model.set(Some(click_id.clone()));
                                        }
                                    },
                                    span { class: "kit-digest__logo", "aria-hidden": "true",
                                        if let Some(url) = logo {
                                            img { src: "{url}", alt: "" }
                                        } else {
                                            "{logo_letter}"
                                        }
                                    }
                                    span { class: "kit-digest__model-name",
                                        span { class: "kit-digest__model-title", "{name}" }
                                        span { class: "kit-digest__role", "{role}" }
                                    }
                                    span { class: "kit-digest__effort", "aria-hidden": "true",
                                        for (bar_index, lit) in bars.into_iter().enumerate() {
                                            span { key: "{bar_index}", class: if lit { "kit-digest__bar-lit" } else { "kit-digest__bar-dim" } }
                                        }
                                    }
                                    span { class: "kit-digest__model-time", "{time}" }
                                }
                            }
                        }
                    }
                }
                div { id: "{detail_id}", class: "kit-digest__slot",
                    if let Some((provider, tokens, spend, note)) = open_detail {
                        div { class: "kit-digest__detail", role: "region", "aria-label": "{open_name} details",
                            dl {
                                div { dt { "Provider" } dd { "{provider}" } }
                                if let Some(tokens) = tokens {
                                    div { dt { "Tokens" } dd { "{tokens}" } }
                                }
                                if let Some(spend) = spend {
                                    div { dt { "Spend" } dd { "{spend}" } }
                                }
                            }
                            if let Some(note) = note {
                                p { class: "kit-digest__note", "{note}" }
                            }
                        }
                    }
                }
            }
            div { class: "kit-digest__lists",
                button {
                    r#type: "button",
                    class: "kit-digest__toggle",
                    "aria-expanded": "{sources_expanded}",
                    onclick: move |_| sources_open.set(!sources_open()),
                    "{sources_label}"
                }
                if sources_open() {
                    div { class: "kit-digest__stack", "aria-hidden": "true",
                        for url in source_list.iter().filter_map(|source| source.logo.clone()) {
                            span { class: "kit-digest__logo",
                                img { src: "{url}", alt: "" }
                            }
                        }
                    }
                    ul { class: "kit-digest__sources",
                        for source in source_list.iter() {
                            {
                                let name = source.source_name.clone();
                                let detail = source.source_detail.clone().unwrap_or_default();
                                let letter = initial(&name);
                                let logo = source.logo.clone();
                                let href = source.source_href.clone();
                                rsx! {
                                    li { key: "{name}",
                                        span { class: "kit-digest__logo", "aria-hidden": "true",
                                            if let Some(url) = logo {
                                                img { src: "{url}", alt: "" }
                                            } else {
                                                "{letter}"
                                            }
                                        }
                                        if let Some(href) = href {
                                            a { href: "{href}", "{name}" }
                                        } else {
                                            span { "{name}" }
                                        }
                                        if !detail.is_empty() {
                                            span { class: "kit-digest__detail-text", "{detail}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                button {
                    r#type: "button",
                    class: "kit-digest__toggle",
                    "aria-expanded": "{steps_expanded}",
                    onclick: move |_| steps_open.set(!steps_open()),
                    "{steps_label}"
                }
                if steps_open() {
                    ol { class: "kit-digest__steps",
                        for step in live_steps.iter() {
                            {
                                let label = step.step_label.clone();
                                let kind_name = table
                                    .get(&step.kind_key)
                                    .map(|s| s.kind_label.clone())
                                    .unwrap_or_else(|| step.kind_key.clone());
                                let mark = table
                                    .get(&step.kind_key)
                                    .and_then(|s| s.kind_mark.clone())
                                    .unwrap_or_else(|| "■".to_string());
                                let time = match step.elapsed_time {
                                    Some(t) => format_duration(t),
                                    None => "running".to_string(),
                                };
                                let related = is_related(step, &current_focus);
                                let related_attr = if related { "true" } else { "false" };
                                let running_attr = if step.is_running { "true" } else { "false" };
                                let key = step.step_id.clone();
                                rsx! {
                                    li { key: "{key}", class: "kit-digest__step", "data-related": "{related_attr}", "data-running": "{running_attr}",
                                        span { class: "kit-digest__kind-mark", "aria-hidden": "true", "{mark}" }
                                        span { class: "kit-digest__step-label", title: "{label}", "{label}" }
                                        span { class: "kit-digest__step-kind", "{kind_name}" }
                                        span { class: "kit-digest__step-time", "{time}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            p { class: "kit-digest__sr", role: "status", "aria-live": "polite", "{announcement}" }
        }
    }
}
