//! Prompt test board: one prompt's versions, its figures and pass rate, proposed edits reviewed one at a time,
//! and fixed test cases with their latest results.
//!
//! Design choice: the pass-rate bars are `SeriesTrendChart` in `Bar` form; the selected version is its controlled `active` bar.
//! Design choice: `prompt_text` is an input the spec does not list; proposed changes are marked inline in it by matching
//! each change's `before` text. A change whose `before` is absent is listed under the text instead.
//! Design choice: a decision made on the board hides its change at once; the host still owns the list and persists it.
//! Design choice: the sidebar is drawn inline so the screen stands alone, without the workspace frame.

use dioxus::prelude::*;

use crate::components::series_trend_chart::{
    ChartVariant, SeriesTone, SeriesTrendChart, TrendSeries, group_digits,
};
use crate::internal::*;

const CSS: Asset = asset!("/assets/prompt-test-board.css");

/// Sidebar entries. Evals is the active entry.
pub const NAV_ITEMS: [&str; 5] = ["Chat", "Evals", "Datasets", "Analytics", "Prompts"];

#[derive(Clone, Debug, PartialEq)]
pub struct Account {
    pub name: String,
    pub avatar_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Version {
    pub id: String,
    pub label: String,
    pub created_at: String,
    pub pass_rate: f64,
}

/// Direction of a figure, and of a change against the previous version.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Trend {
    Up,
    Down,
    #[default]
    Flat,
}

impl Trend {
    pub fn word(self) -> &'static str {
        match self {
            Self::Up => "Up",
            Self::Down => "Down",
            Self::Flat => "Flat",
        }
    }

    fn key(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
            Self::Flat => "flat",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct KeyFigure {
    pub label: String,
    pub value: String,
    pub delta: String,
    pub trend: Trend,
    pub good_when: Trend,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PassPoint {
    pub version_label: String,
    pub pass_rate: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeKind {
    Add,
    Remove,
    Replace,
}

impl ChangeKind {
    pub fn word(self) -> &'static str {
        match self {
            Self::Add => "Add",
            Self::Remove => "Remove",
            Self::Replace => "Replace",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProposedChange {
    pub id: String,
    pub kind: ChangeKind,
    pub before: String,
    pub after: String,
    pub author: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaseStatus {
    Pass,
    Partial,
    Fail,
    Pending,
}

impl CaseStatus {
    pub fn word(self) -> &'static str {
        match self {
            Self::Pass => "Pass",
            Self::Partial => "Partial",
            Self::Fail => "Fail",
            Self::Pending => "Pending",
        }
    }

    fn key(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Partial => "partial",
            Self::Fail => "fail",
            Self::Pending => "pending",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TestCase {
    pub id: String,
    pub input: String,
    pub expected: String,
    pub output: Option<String>,
    pub score: Option<f64>,
    pub max_score: f64,
    pub latency_ms: Option<f64>,
    pub status: CaseStatus,
}

/// The status filter on the test case list.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CaseFilter {
    #[default]
    All,
    Pass,
    Partial,
    Fail,
}

impl CaseFilter {
    pub const ALL: [Self; 4] = [Self::All, Self::Pass, Self::Partial, Self::Fail];

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Pass => "Pass",
            Self::Partial => "Partial",
            Self::Fail => "Fail",
        }
    }

    pub fn admits(self, status: CaseStatus) -> bool {
        match self {
            Self::All => true,
            Self::Pass => status == CaseStatus::Pass,
            Self::Partial => status == CaseStatus::Partial,
            Self::Fail => status == CaseStatus::Fail,
        }
    }
}

/// A change's decision made on the board.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Accepted,
    Rejected,
}

/// One piece of the prompt text as shown: plain, a removed span, or an added span.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InlineSegment {
    Plain(String),
    Removed { change_id: String, text: String },
    Added { change_id: String, text: String },
}

/// Marks each undecided change inline in `text`. An accepted change reads as its `after` text and a rejected one
/// leaves the text as it was. A change whose `before` is not in the text is not placed here.
pub fn inline_segments(
    text: &str,
    changes: &[ProposedChange],
    decisions: &[(String, Decision)],
) -> Vec<InlineSegment> {
    let mut segments = vec![InlineSegment::Plain(text.to_string())];
    for change in changes {
        let decision = decisions
            .iter()
            .find(|(id, _)| *id == change.id)
            .map(|(_, d)| *d);
        if decision == Some(Decision::Rejected) {
            continue;
        }
        let accepted = decision == Some(Decision::Accepted);
        if change.before.is_empty() {
            if change.kind == ChangeKind::Add && !change.after.is_empty() {
                let piece = if accepted {
                    InlineSegment::Plain(change.after.clone())
                } else {
                    InlineSegment::Added {
                        change_id: change.id.clone(),
                        text: change.after.clone(),
                    }
                };
                segments.push(piece);
            }
            continue;
        }
        let found = segments.iter().enumerate().find_map(|(i, seg)| match seg {
            InlineSegment::Plain(s) => s.find(&change.before).map(|at| (i, at)),
            _ => None,
        });
        let Some((index, at)) = found else {
            continue;
        };
        let InlineSegment::Plain(whole) = segments[index].clone() else {
            continue;
        };
        let mut replacement = Vec::new();
        if at > 0 {
            replacement.push(InlineSegment::Plain(whole[..at].to_string()));
        }
        if accepted {
            if !change.after.is_empty() {
                replacement.push(InlineSegment::Plain(change.after.clone()));
            }
        } else {
            replacement.push(InlineSegment::Removed {
                change_id: change.id.clone(),
                text: change.before.clone(),
            });
            if !change.after.is_empty() {
                replacement.push(InlineSegment::Added {
                    change_id: change.id.clone(),
                    text: change.after.clone(),
                });
            }
        }
        let tail = &whole[at + change.before.len()..];
        if !tail.is_empty() {
            replacement.push(InlineSegment::Plain(tail.to_string()));
        }
        segments.splice(index..=index, replacement);
    }
    segments
}

/// Changes still waiting for a decision.
pub fn pending_count(changes: &[ProposedChange], decisions: &[(String, Decision)]) -> usize {
    changes
        .iter()
        .filter(|c| !decisions.iter().any(|(id, _)| *id == c.id))
        .count()
}

/// Test cases a filter admits, in their given order.
pub fn visible_cases(cases: &[TestCase], filter: CaseFilter) -> Vec<&TestCase> {
    cases.iter().filter(|c| filter.admits(c.status)).collect()
}

/// Whether a figure is a positive or a negative reading, given its trend and the direction that is good.
/// The filter a key on the filter tablist moves to, wrapping at both ends; `None` for keys it does not use.
pub fn filter_after_key(current: CaseFilter, key: &str) -> Option<CaseFilter> {
    let last = CaseFilter::ALL.len() - 1;
    let at = CaseFilter::ALL.iter().position(|f| *f == current)?;
    let next = match key {
        "ArrowRight" => (at + 1) % CaseFilter::ALL.len(),
        "ArrowLeft" => (at + last) % CaseFilter::ALL.len(),
        "Home" => 0,
        "End" => last,
        _ => return None,
    };
    Some(CaseFilter::ALL[next])
}

pub fn figure_tone(trend: Trend, good_when: Trend) -> &'static str {
    match trend {
        Trend::Flat => "neutral",
        t if t == good_when => "positive",
        _ => "negative",
    }
}

/// Milliseconds as `420 ms`, or seconds with one decimal from one second up.
pub fn format_latency(ms: f64) -> String {
    if ms >= 1000.0 {
        format!("{:.1} s", ms / 1000.0)
    } else {
        format!("{ms:.0} ms")
    }
}

fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

#[derive(Clone, PartialEq)]
struct CaseView {
    id: String,
    input: String,
    status_word: &'static str,
    status_key: &'static str,
    score_text: String,
    latency_text: String,
    expected: String,
    output_text: String,
    score_detail: String,
}

#[derive(Clone, PartialEq)]
struct ChangeView {
    id: String,
    kind_word: &'static str,
    after: String,
    author: String,
    aria: String,
}

/// Prompt test board: figures, pass rate by version, a reviewable prompt diff and the test case list.
#[component]
#[allow(clippy::too_many_arguments)]
pub fn PromptTestBoard(
    #[props(default = "Juniper Works".to_string(), into)] workspace_name: String,
    account: Account,
    #[props(default, into)] prompt_name: String,
    #[props(default)] prompt_names: Vec<String>,
    #[props(default)] versions: Vec<Version>,
    #[props(default, into)] current_version: String,
    #[props(default, into)] model: String,
    #[props(default, into)] last_run_text: String,
    #[props(default)] key_figures: Vec<KeyFigure>,
    #[props(default)] pass_rate_series: Vec<PassPoint>,
    #[props(default, into)] prompt_text: String,
    #[props(default)] proposed_changes: Vec<ProposedChange>,
    #[props(default)] test_cases: Vec<TestCase>,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] running: bool,
    #[props(default)] on_run_eval: EventHandler<()>,
    #[props(default)] on_select_version: EventHandler<String>,
    #[props(default)] on_accept_change: EventHandler<String>,
    #[props(default)] on_reject_change: EventHandler<String>,
    #[props(default)] on_accept_all: EventHandler<()>,
    #[props(default)] on_reject_all: EventHandler<()>,
    #[props(default)] on_add_case: EventHandler<()>,
    #[props(default)] on_open_case: EventHandler<String>,
) -> Element {
    let base = use_hook(|| format!("kit-ptb-{}", next_id()));
    let mut chosen = use_signal(|| current_version.clone());
    let mut menu_open = use_signal(|| false);
    let mut menu_index = use_signal(|| 0_usize);
    let mut decisions = use_signal(Vec::<(String, Decision)>::new);
    let mut filter = use_signal(CaseFilter::default);
    let mut open_case = use_signal(|| None::<String>);
    let mut announce = use_signal(String::new);

    let tabs_base = base.clone();
    let trigger_id = format!("{base}-version");
    let list_id = format!("{base}-versions");
    let decided = decisions();
    let pending = pending_count(&proposed_changes, &decided);
    let version_index = versions.iter().position(|v| v.id == chosen()).unwrap_or(0);
    let selected_label = versions
        .get(version_index)
        .map(|v| v.label.clone())
        .unwrap_or_default();
    let latest_id = versions.last().map(|v| v.id.clone()).unwrap_or_default();
    let is_latest = chosen() == latest_id;
    let segments = inline_segments(&prompt_text, &proposed_changes, &decided);
    let unplaced: Vec<String> = proposed_changes
        .iter()
        .filter(|c| !c.before.is_empty() && !prompt_text.contains(&c.before))
        .filter(|c| !decided.iter().any(|(id, _)| *id == c.id))
        .map(|c| format!("{}: {}", c.kind.word(), c.after))
        .collect();
    let change_views: Vec<ChangeView> = proposed_changes
        .iter()
        .enumerate()
        .filter(|(_, c)| !decided.iter().any(|(id, _)| *id == c.id))
        .map(|(i, c)| ChangeView {
            id: c.id.clone(),
            kind_word: c.kind.word(),
            after: c.after.clone(),
            author: c.author.clone(),
            aria: format!(
                "{} change {} of {} by {}. Press Y to accept, N to reject.",
                c.kind.word(),
                i + 1,
                proposed_changes.len(),
                c.author
            ),
        })
        .collect();
    let cases_now: Vec<TestCase> = test_cases
        .iter()
        .map(|c| {
            let mut case = c.clone();
            if running {
                case.status = CaseStatus::Pending;
            }
            case
        })
        .collect();
    let active_filter = filter();
    let case_views: Vec<CaseView> = visible_cases(&cases_now, active_filter)
        .into_iter()
        .map(|c| CaseView {
            id: c.id.clone(),
            input: c.input.clone(),
            status_word: c.status.word(),
            status_key: c.status.key(),
            score_text: match c.score {
                Some(s) => format!("{s:.1} / {:.1}", c.max_score),
                None => format!("– / {:.1}", c.max_score),
            },
            latency_text: c
                .latency_ms
                .map(format_latency)
                .unwrap_or_else(|| "–".into()),
            expected: c.expected.clone(),
            output_text: c.output.clone().unwrap_or_else(|| "No output yet".into()),
            score_detail: match c.score {
                Some(s) => format!("{s:.1} of {:.1}", c.max_score),
                None => "Not scored".into(),
            },
        })
        .collect();
    let filter_tabs: Vec<(CaseFilter, usize)> = CaseFilter::ALL
        .iter()
        .map(|f| (*f, visible_cases(&cases_now, *f).len()))
        .collect();
    let version_series = vec![TrendSeries {
        name: "Pass rate".into(),
        values: pass_rate_series.iter().map(|p| Some(p.pass_rate)).collect(),
        tone: SeriesTone::Ink,
        emphasis: true,
        is_earlier: false,
    }];
    let version_labels: Vec<String> = pass_rate_series
        .iter()
        .map(|p| p.version_label.clone())
        .collect();
    let chart_active = versions.get(version_index).and_then(|v| {
        pass_rate_series
            .iter()
            .position(|p| p.version_label == v.label)
    });
    let figure_views: Vec<(KeyFigure, &'static str)> = key_figures
        .iter()
        .map(|f| (f.clone(), figure_tone(f.trend, f.good_when)))
        .collect();
    let summary_rows: Vec<String> = pass_rate_series
        .iter()
        .map(|p| format!("{}: {:.0}%", p.version_label, p.pass_rate))
        .collect();
    let caption = format!(
        "{} versions, {} test cases",
        versions.len(),
        test_cases.len()
    );
    let run_label = if running {
        "Running…"
    } else {
        "Run evaluation"
    };
    let reject_ids: Vec<String> = proposed_changes.iter().map(|c| c.id.clone()).collect();
    let accept_ids = reject_ids.clone();
    let open_id = open_case();
    let options_menu = versions.clone();
    let options_keys = versions.clone();

    let on_trigger = {
        let list_id = list_id.clone();
        move |_| {
            if menu_open() {
                menu_open.set(false);
            } else {
                menu_index.set(version_index);
                menu_open.set(true);
                focus_after_frame(&list_id);
            }
        }
    };
    let on_menu_keys = {
        let trigger_id = trigger_id.clone();
        move |event: KeyboardEvent| {
            let last = options_keys.len().saturating_sub(1);
            match event.key() {
                Key::ArrowDown => {
                    event.prevent_default();
                    menu_index.set((menu_index() + 1).min(last));
                }
                Key::ArrowUp => {
                    event.prevent_default();
                    menu_index.set(menu_index().saturating_sub(1));
                }
                Key::Enter => {
                    event.prevent_default();
                    if let Some(v) = options_keys.get(menu_index()) {
                        chosen.set(v.id.clone());
                        announce.set(format!("Version {}", v.label));
                        on_select_version.call(v.id.clone());
                    }
                    menu_open.set(false);
                    focus_after_frame(&trigger_id);
                }
                Key::Escape => {
                    event.prevent_default();
                    menu_open.set(false);
                    focus_after_frame(&trigger_id);
                }
                _ => {}
            }
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "kit-ptb", "aria-labelledby": "{base}-title",
            nav { class: "kit-ptb__nav", "aria-label": "Sidebar",
                p { class: "kit-ptb__ws", "{workspace_name}" }
                ul { class: "kit-ptb__nav-list",
                    for (i, item) in NAV_ITEMS.iter().enumerate() {
                        li { key: "{i}",
                            span { class: "kit-ptb__nav-item", "aria-current": if *item == "Evals" { "page" } else { "false" },
                                "{item}"
                                if *item == "Evals" && !test_cases.is_empty() {
                                    span { class: "kit-ptb__count", "{test_cases.len()}" }
                                }
                            }
                        }
                    }
                }
                if !prompt_names.is_empty() {
                    p { class: "kit-ptb__group", "Prompts" }
                    ul { class: "kit-ptb__nav-list",
                        for (i, name) in prompt_names.iter().enumerate() {
                            li { key: "{i}",
                                span { class: "kit-ptb__nav-item", "aria-current": if *name == prompt_name { "true" } else { "false" }, "{name}" }
                            }
                        }
                    }
                }
                div { class: "kit-ptb__account",
                    span { class: "kit-ptb__avatar", "aria-hidden": "true",
                        if let Some(src) = account.avatar_url.clone() {
                            img { src: "{src}", alt: "" }
                        } else {
                            "{initials(&account.name)}"
                        }
                    }
                    span { class: "kit-ptb__account-name", "{account.name}" }
                }
            }

            div { class: "kit-ptb__page",
                header { class: "kit-ptb__head",
                    div { class: "kit-ptb__titlebox",
                        h1 { id: "{base}-title", class: "kit-ptb__title", "{prompt_name}" }
                        div { class: "kit-ptb__meta",
                            if !versions.is_empty() {
                                div { class: "kit-ptb__version",
                                    button { id: "{trigger_id}", class: "kit-ptb__btn kit-ptb__version-btn", r#type: "button",
                                        "aria-haspopup": "listbox", "aria-expanded": "{menu_open()}", "aria-controls": "{list_id}",
                                        onclick: on_trigger,
                                        "{selected_label}"
                                        span { class: "kit-ptb__caret", "aria-hidden": "true", "▾" }
                                    }
                                    if menu_open() {
                                        ul { id: "{list_id}", class: "kit-ptb__menu", role: "listbox", tabindex: "-1", "aria-label": "Prompt version",
                                            "aria-activedescendant": "{list_id}-{menu_index()}",
                                            onkeydown: on_menu_keys,
                                            for (i, v) in options_menu.iter().enumerate() {
                                                li { id: "{list_id}-{i}", key: "{v.id}", role: "option",
                                                    class: if i == menu_index() { "kit-ptb__opt is-active" } else { "kit-ptb__opt" },
                                                    "aria-selected": if v.id == chosen() { "true" } else { "false" },
                                                    onclick: {
                                                        let v = v.clone();
                                                        move |_| {
                                                            chosen.set(v.id.clone());
                                                            announce.set(format!("Version {}", v.label));
                                                            on_select_version.call(v.id.clone());
                                                            menu_open.set(false);
                                                        }
                                                    },
                                                    "{v.label}"
                                                    span { class: "kit-ptb__opt-date", "{v.created_at}" }
                                                }
                                            }
                                        }
                                    }
                                }
                                if is_latest {
                                    span { class: "kit-ptb__tag", "Current" }
                                }
                            }
                            if !model.is_empty() {
                                span { class: "kit-ptb__model", "{model}" }
                            }
                            if !last_run_text.is_empty() {
                                span { class: "kit-ptb__last", "{last_run_text}" }
                            }
                        }
                    }
                    button { class: "kit-ptb__btn kit-ptb__btn--primary", r#type: "button", "aria-busy": "{running}",
                        onclick: move |_| {
                            announce.set("Evaluation started".to_string());
                            on_run_eval.call(());
                        },
                        "{run_label}"
                    }
                }

                if is_loading {
                    div { class: "kit-ptb__figures", "aria-busy": "true",
                        for i in 0..4 {
                            div { key: "{i}", class: "kit-ptb__card kit-ptb__card--figure",
                                span { class: "kit-ptb__ph kit-ptb__ph--line" }
                                span { class: "kit-ptb__ph kit-ptb__ph--big" }
                            }
                        }
                    }
                } else if !is_empty {
                    div { class: "kit-ptb__figures",
                        for (i, (figure, tone)) in figure_views.iter().enumerate() {
                            section { key: "{i}", class: "kit-ptb__card kit-ptb__card--figure", role: "group",
                                "aria-label": "{figure.label}", "data-tone": "{tone}",
                                span { class: "kit-ptb__figure-label", "{figure.label}" }
                                span { class: "kit-ptb__figure-value", "{figure.value}" }
                                span { class: "kit-ptb__delta", "data-sign": figure.trend.key(),
                                    span { class: "kit-ptb__sign", "{figure.trend.word()}" }
                                    " {figure.delta}"
                                }
                            }
                        }
                    }
                }

                div { class: "kit-ptb__columns",
                    if is_loading {
                        section { class: "kit-ptb__card kit-ptb__card--chart", "aria-busy": "true",
                            h2 { class: "kit-ptb__card-title", "Pass rate by version" }
                            span { class: "kit-ptb__ph kit-ptb__ph--plot" }
                        }
                    } else if !is_empty {
                        section { class: "kit-ptb__card kit-ptb__card--chart", "aria-labelledby": "{base}-chart",
                            h2 { id: "{base}-chart", class: "kit-ptb__card-title", "Pass rate by version" }
                            p { class: "kit-ptb__caption", "{caption}" }
                            if !pass_rate_series.is_empty() {
                                SeriesTrendChart {
                                    x_labels: version_labels,
                                    series: version_series,
                                    variant: ChartVariant::Bar,
                                    height: 200.0,
                                    active: chart_active.map(Some),
                                    label: "Pass rate by version",
                                    fmt_value: group_digits,
                                }
                            }
                            ul { class: "kit-ptb__sr-list kit-sr",
                                for (i, line) in summary_rows.iter().enumerate() {
                                    li { key: "{i}", "{line}" }
                                }
                            }
                        }

                        section { class: "kit-ptb__card kit-ptb__changes", "aria-labelledby": "{base}-changes",
                            div { class: "kit-ptb__card-head",
                                h2 { id: "{base}-changes", class: "kit-ptb__card-title",
                                    "Changes "
                                    span { class: "kit-ptb__count", "{pending}" }
                                }
                                div { class: "kit-ptb__bulk",
                                    button { class: "kit-ptb__btn", r#type: "button", disabled: pending == 0,
                                        onclick: move |_| {
                                            let ids = reject_ids.clone();
                                            decisions.with_mut(|d| {
                                                for id in ids {
                                                    if !d.iter().any(|(x, _)| *x == id) {
                                                        d.push((id, Decision::Rejected));
                                                    }
                                                }
                                            });
                                            announce.set("Rejected all proposed changes".to_string());
                                            on_reject_all.call(());
                                        },
                                        "Reject all ({pending})"
                                    }
                                    button { class: "kit-ptb__btn kit-ptb__btn--primary", r#type: "button", disabled: pending == 0,
                                        onclick: move |_| {
                                            let ids = accept_ids.clone();
                                            decisions.with_mut(|d| {
                                                for id in ids {
                                                    if !d.iter().any(|(x, _)| *x == id) {
                                                        d.push((id, Decision::Accepted));
                                                    }
                                                }
                                            });
                                            announce.set("Accepted all proposed changes".to_string());
                                            on_accept_all.call(());
                                        },
                                        "Accept all ({pending})"
                                    }
                                }
                            }
                            p { class: "kit-ptb__text", "data-testid": "prompt-text",
                                for (i, seg) in segments.iter().enumerate() {
                                    {
                                        match seg {
                                            InlineSegment::Plain(t) => rsx! { span { key: "{i}", "{t}" } },
                                            InlineSegment::Removed { change_id, text } => rsx! {
                                                del { key: "{i}", class: "kit-ptb__del", "data-change": "{change_id}", "{text}" }
                                            },
                                            InlineSegment::Added { change_id, text } => rsx! {
                                                ins { key: "{i}", class: "kit-ptb__ins", "data-change": "{change_id}", "{text}" }
                                            },
                                        }
                                    }
                                }
                            }
                            if !unplaced.is_empty() {
                                ul { class: "kit-ptb__unplaced",
                                    for (i, line) in unplaced.iter().enumerate() {
                                        li { key: "{i}", "{line}" }
                                    }
                                }
                            }
                            ul { class: "kit-ptb__change-list", "aria-label": "Proposed changes",
                                for view in change_views.iter() {
                                    li { key: "{view.id}", class: "kit-ptb__change", tabindex: "0",
                                        "aria-label": "{view.aria}", "aria-keyshortcuts": "Y N",
                                        onkeydown: {
                                            let id = view.id.clone();
                                            move |event: KeyboardEvent| {
                                                let decision = match event.key() {
                                                    Key::Character(c) if c == "y" || c == "Y" => Some(Decision::Accepted),
                                                    Key::Character(c) if c == "n" || c == "N" => Some(Decision::Rejected),
                                                    _ => None,
                                                };
                                                if let Some(d) = decision {
                                                    event.prevent_default();
                                                    decisions.with_mut(|list| list.push((id.clone(), d)));
                                                    if d == Decision::Accepted {
                                                        announce.set("Accepted change".to_string());
                                                        on_accept_change.call(id.clone());
                                                    } else {
                                                        announce.set("Rejected change".to_string());
                                                        on_reject_change.call(id.clone());
                                                    }
                                                }
                                            }
                                        },
                                        span { class: "kit-ptb__change-kind", "{view.kind_word}" }
                                        span { class: "kit-ptb__change-text", "{view.after}" }
                                        span { class: "kit-ptb__change-by", "{view.author}" }
                                    }
                                }
                            }
                        }
                    }

                    section { class: "kit-ptb__card kit-ptb__cases", "aria-labelledby": "{base}-cases",
                        div { class: "kit-ptb__card-head",
                            h2 { id: "{base}-cases", class: "kit-ptb__card-title", "Test cases" }
                            if !is_empty && !is_loading {
                                button { class: "kit-ptb__btn", r#type: "button", onclick: move |_| on_add_case.call(()), "Add case" }
                            }
                        }
                        if is_empty {
                            div { class: "kit-ptb__empty",
                                p { class: "kit-ptb__empty-title", "No test cases yet" }
                                button { class: "kit-ptb__btn kit-ptb__btn--primary", r#type: "button", onclick: move |_| on_add_case.call(()), "Add a test case" }
                            }
                        } else if is_loading {
                            for i in 0..3 {
                                span { key: "{i}", class: "kit-ptb__ph kit-ptb__ph--row" }
                            }
                        } else {
                            div { class: "kit-ptb__tabs", role: "tablist", "aria-label": "Filter test cases",
                                onkeydown: move |event: KeyboardEvent| {
                                    let name = match event.key() {
                                        Key::ArrowRight => "ArrowRight",
                                        Key::ArrowLeft => "ArrowLeft",
                                        Key::Home => "Home",
                                        Key::End => "End",
                                        _ => return,
                                    };
                                    if let Some(next) = filter_after_key(filter(), name) {
                                        event.prevent_default();
                                        filter.set(next);
                                        let at = CaseFilter::ALL.iter().position(|f| *f == next).unwrap_or(0);
                                        focus_after_frame(&format!("{tabs_base}-tab-{at}"));
                                    }
                                },
                                for (i, (f, count)) in filter_tabs.iter().enumerate() {
                                    button { key: "{f:?}", id: "{base}-tab-{i}", class: "kit-ptb__tab", r#type: "button", role: "tab",
                                        "aria-selected": if *f == active_filter { "true" } else { "false" },
                                        tabindex: if *f == active_filter { "0" } else { "-1" },
                                        onclick: {
                                            let f = *f;
                                            move |_| filter.set(f)
                                        },
                                        "{f.label()} "
                                        span { class: "kit-ptb__count", "{count}" }
                                    }
                                }
                            }
                            ul { class: "kit-ptb__rows",
                                for view in case_views.iter() {
                                    li { key: "{view.id}", class: "kit-ptb__row", "data-status": "{view.status_key}",
                                        tabindex: "0", role: "button",
                                        "aria-expanded": if open_id.as_deref() == Some(view.id.as_str()) { "true" } else { "false" },
                                        onclick: {
                                            let id = view.id.clone();
                                            move |_| {
                                                let next = if open_case() == Some(id.clone()) { None } else { Some(id.clone()) };
                                                if next.is_some() {
                                                    on_open_case.call(id.clone());
                                                }
                                                open_case.set(next);
                                            }
                                        },
                                        onkeydown: {
                                            let id = view.id.clone();
                                            move |event: KeyboardEvent| {
                                                match event.key() {
                                                    Key::Enter => {
                                                        event.prevent_default();
                                                        open_case.set(Some(id.clone()));
                                                        on_open_case.call(id.clone());
                                                    }
                                                    Key::Character(c) if c == " " => {
                                                        event.prevent_default();
                                                        open_case.set(Some(id.clone()));
                                                        on_open_case.call(id.clone());
                                                    }
                                                    Key::Escape => open_case.set(None),
                                                    _ => {}
                                                }
                                            }
                                        },
                                        div { class: "kit-ptb__row-main",
                                            span { class: "kit-ptb__input", "{view.input}" }
                                            span { class: "kit-ptb__status", "{view.status_word}" }
                                            span { class: "kit-ptb__score", "{view.score_text}" }
                                            span { class: "kit-ptb__latency", "{view.latency_text}" }
                                        }
                                        if open_id.as_deref() == Some(view.id.as_str()) {
                                            dl { class: "kit-ptb__detail",
                                                dt { "Expected" }
                                                dd { "{view.expected}" }
                                                dt { "Output" }
                                                dd { "{view.output_text}" }
                                                dt { "Score" }
                                                dd { "{view.score_detail}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            p { class: "kit-sr", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}
