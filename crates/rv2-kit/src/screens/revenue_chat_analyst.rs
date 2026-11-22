//! Revenue chat analyst: a chat that answers revenue questions with figures and a chart. A person can pick a part of
//! a chart and ask about only that part.
//!
//! Design choice: trend and forecast charts are `SeriesTrendChart`. A forecast's continuation is the chart's
//! `is_earlier` series, which the chart draws dashed. Split charts are drawn here as stacked columns, since the chart
//! kit has no such form; each column's parts add to its total.
//! Design choice: range picking lives here, over the chart's controlled `active` point. On a line chart Enter starts a
//! pick, the arrows widen it, and a second Enter opens the pick field. On a split chart Enter on a column picks it.
//! Design choice: the sidebar is drawn inline, folded to icons by default, so the screen stands alone.

use dioxus::prelude::*;

use crate::components::series_trend_chart::{
    AxisMode, ChartVariant, PLOT_WIDTH, SeriesTone, SeriesTrendChart, TrendSeries, compact_number,
    group_digits, project_x,
};
use crate::internal::*;

const CSS: Asset = asset!("/assets/revenue-chat-analyst.css");

/// Plot span of a line chart with its y axis drawn, in the chart's viewBox units.
const PLOT_LEFT: f64 = 44.0;
const PLOT_RIGHT: f64 = PLOT_WIDTH - 8.0;

#[derive(Clone, Debug, PartialEq)]
pub struct FocusRange {
    pub chart_kind: String,
    pub from_point: usize,
    pub to_point: usize,
    pub label_text: String,
    pub detail_text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MetricCard {
    pub entry_label: String,
    pub value: f64,
    pub change_text: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SplitPart {
    pub entry_name: String,
    pub entry_value: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SplitGroup {
    pub entry_name: String,
    pub parts: Vec<SplitPart>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ChartBlock {
    Trend {
        category_labels: Vec<String>,
        current_values: Vec<f64>,
        previous_values: Vec<f64>,
        marker: Option<String>,
    },
    Forecast {
        category_labels: Vec<String>,
        actual_values: Vec<f64>,
        forecast_values: Vec<f64>,
        target_value: Option<f64>,
    },
    Split {
        title_text: String,
        previous_total: Option<f64>,
        groups: Vec<SplitGroup>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnswerBody {
    pub text: String,
    pub metric_cards: Vec<MetricCard>,
    pub chart_block: Option<ChartBlock>,
    pub caption_text: Option<String>,
    pub source_list: Vec<String>,
    pub follow_up_list: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Turn {
    Question {
        id: String,
        text: String,
        quoted_focus: Option<FocusRange>,
    },
    Answer {
        id: String,
        body: AnswerBody,
    },
}

/// Metric card figure: large values shorten with a magnitude suffix.
pub fn short_value(value: f64) -> String {
    compact_number(value)
}

/// The total a split column's parts add to. Negative and non-finite parts count as zero.
pub fn split_total(group: &SplitGroup) -> f64 {
    group
        .parts
        .iter()
        .map(|p| p.entry_value)
        .filter(|v| v.is_finite() && *v >= 0.0)
        .sum()
}

/// Forecast series: actuals, then the forecast from the last actual point on, so the dashed line joins them.
pub fn forecast_series(actual: &[f64], forecast: &[f64]) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    let mut actual_line: Vec<Option<f64>> = actual.iter().map(|v| Some(*v)).collect();
    let mut continuation: Vec<Option<f64>> = vec![None; actual.len()];
    if let (Some(join), Some(slot)) = (actual.last(), continuation.last_mut()) {
        *slot = Some(*join);
    }
    continuation.extend(forecast.iter().map(|v| Some(*v)));
    actual_line.extend(std::iter::repeat_n(None, forecast.len()));
    (actual_line, continuation)
}

/// The focus range for a line pick, from one point to another, labelled with the chart's own labels.
pub fn line_focus(chart_kind: &str, from: usize, to: usize, labels: &[String]) -> FocusRange {
    let (lo, hi) = if from <= to { (from, to) } else { (to, from) };
    let first = labels
        .get(lo)
        .cloned()
        .unwrap_or_else(|| format!("point {}", lo + 1));
    let last = labels
        .get(hi)
        .cloned()
        .unwrap_or_else(|| format!("point {}", hi + 1));
    FocusRange {
        chart_kind: chart_kind.to_string(),
        from_point: lo,
        to_point: hi,
        label_text: format!("{first} to {last}"),
        detail_text: format!("{} points", hi - lo + 1),
    }
}

/// Keeps a pick's ends inside `count` points.
pub fn clamp_range(from: usize, to: usize, count: usize) -> (usize, usize) {
    let last = count.saturating_sub(1);
    (from.min(last), to.min(last))
}

fn initial_of(text: &str) -> String {
    text.chars()
        .next()
        .map(|c| c.to_string())
        .unwrap_or_default()
}

fn part_tone(index: usize) -> &'static str {
    ["ink", "ok", "warn", "muted"][index % 4]
}

/// CSS left and width of a band over points `from..=to` of a line chart with `count` points.
fn band_style(from: usize, to: usize, count: usize) -> String {
    let (lo, hi) = clamp_range(from, to, count);
    let x = |i: usize| project_x(i, count, PLOT_LEFT, PLOT_RIGHT) / PLOT_WIDTH * 100.0;
    let (a, b) = (x(lo), x(hi));
    let width = (b - a).max(1.5);
    format!("left: {a:.2}%; width: {width:.2}%;")
}

/// Horizontal centre of a pick, as a percentage of the chart width.
fn pick_center_pct(range: &FocusRange, block: &ChartBlock) -> f64 {
    match block {
        ChartBlock::Split { groups, .. } => {
            (range.from_point as f64 + 0.5) / groups.len().max(1) as f64 * 100.0
        }
        ChartBlock::Forecast {
            category_labels, ..
        } => {
            let mid = (range.from_point + range.to_point) as f64 / 2.0;
            let steps = category_labels.len().max(2) - 1;
            (PLOT_LEFT + (PLOT_RIGHT - PLOT_LEFT) * mid / steps as f64) / PLOT_WIDTH * 100.0
        }
        ChartBlock::Trend { .. } => 50.0,
    }
}

#[derive(Clone, PartialEq)]
struct MetricView {
    label: String,
    short: String,
    full: String,
    change: Option<String>,
}

fn metric_views(cards: &[MetricCard]) -> Vec<MetricView> {
    cards
        .iter()
        .map(|c| MetricView {
            label: c.entry_label.clone(),
            short: short_value(c.value),
            full: group_digits(c.value),
            change: c.change_text.clone(),
        })
        .collect()
}

/// Revenue analyst page: chat list, the thread of questions and answers, and a composer with a picked-range field.
#[component]
pub fn RevenueChatAnalyst(
    #[props(default)] chat_list: Vec<String>,
    #[props(default)] turn_list: Vec<Turn>,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] starter_questions: Vec<String>,
    #[props(default)] on_ask: EventHandler<(String, Option<FocusRange>)>,
    #[props(default)] on_navigate: EventHandler<String>,
) -> Element {
    let base = use_hook(|| format!("kit-rca-{}", next_id()));
    let mut turns = use_signal(|| turn_list.clone());
    let mut draft = use_signal(String::new);
    let mut pick = use_signal(|| None::<FocusRange>);
    let mut pick_open = use_signal(|| false);
    let pick_start = use_signal(|| None::<usize>);
    let pick_end = use_signal(|| 0_usize);
    let mut chart_focus = use_signal(String::new);
    let active = use_signal(|| None::<usize>);
    let mut announce = use_signal(String::new);
    let mut sidebar_open = use_signal(|| false);
    let mut working = use_signal(|| false);

    let turns_now = turns();
    let starters = if starter_questions.is_empty() {
        vec![
            "How did revenue change this quarter?".to_string(),
            "Which region grew fastest?".to_string(),
            "What will next month look like?".to_string(),
        ]
    } else {
        starter_questions
    };
    let sidebar_class = if sidebar_open() {
        "kit-rca__side is-open"
    } else {
        "kit-rca__side"
    };
    let pick_now = pick();
    let picked_text = pick_now
        .as_ref()
        .map(|r| format!("{} · {}", r.label_text, r.detail_text))
        .unwrap_or_default();
    let thread_id = format!("{base}-thread");
    let pick_input_id = format!("{base}-pickinput");

    let mut send = move |text: String, focus: Option<FocusRange>| {
        let question = text.trim().to_string();
        if question.is_empty() {
            return;
        }
        let id = format!("q-{}", turns().len() + 1);
        turns.with_mut(|list| {
            list.push(Turn::Question {
                id,
                text: question.clone(),
                quoted_focus: focus.clone(),
            });
        });
        draft.set(String::new());
        pick.set(None);
        pick_open.set(false);
        working.set(true);
        announce.set("Reading sources".to_string());
        on_ask.call((question, focus));
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "kit-rca", "aria-labelledby": "{base}-title",
            aside { class: "{sidebar_class}", "aria-label": "Chats",
                button { class: "kit-rca__toggle", r#type: "button", "aria-expanded": "{sidebar_open()}",
                    onclick: move |_| sidebar_open.toggle(),
                    span { class: "kit-sr", "Show chats" }
                    span { "aria-hidden": "true", "≡" }
                }
                ul { class: "kit-rca__chats",
                    for (i, chat) in chat_list.iter().enumerate() {
                        li { key: "{i}",
                            button { class: "kit-rca__chat", r#type: "button", title: "{chat}",
                                onclick: {
                                    let label = chat.clone();
                                    move |_| on_navigate.call(label.clone())
                                },
                                span { class: "kit-rca__chat-mark", "aria-hidden": "true", "{initial_of(chat)}" }
                                span { class: "kit-rca__chat-name", "{chat}" }
                            }
                        }
                    }
                }
                button { class: "kit-rca__chat", r#type: "button", title: "Revenue", onclick: move |_| on_navigate.call("Revenue".to_string()),
                    span { class: "kit-rca__chat-mark", "aria-hidden": "true", "R" }
                    span { class: "kit-rca__chat-name", "Revenue" }
                }
            }

            div { class: "kit-rca__main",
                h1 { id: "{base}-title", class: "kit-sr", "Revenue analyst" }
                div { id: "{thread_id}", class: "kit-rca__thread", role: "log", "aria-busy": "{working()}",
                    if is_loading {
                        span { class: "kit-rca__ph kit-rca__ph--line" }
                        span { class: "kit-rca__ph kit-rca__ph--block" }
                    } else if is_empty || turns_now.is_empty() {
                        div { class: "kit-rca__empty",
                            p { class: "kit-rca__empty-title", "Ask about revenue" }
                            p { class: "kit-rca__empty-text", "Answers come with figures and a chart you can question." }
                            ul { class: "kit-rca__starters",
                                for (i, s) in starters.iter().enumerate() {
                                    li { key: "{i}",
                                        button { class: "kit-rca__chip", r#type: "button",
                                            onclick: {
                                                let s = s.clone();
                                                move |_| send(s.clone(), None)
                                            },
                                            "{s}"
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        for turn in turns_now.iter() {
                            {
                                match turn.clone() {
                                    Turn::Question { id, text, quoted_focus } => rsx! {
                                        div { key: "{id}", class: "kit-rca__question",
                                            if let Some(q) = quoted_focus.clone() {
                                                p { class: "kit-rca__quote", "{q.label_text}" }
                                            }
                                            p { class: "kit-rca__bubble", "{text}" }
                                        }
                                    },
                                    Turn::Answer { id, body } => {
                                        let metrics = metric_views(&body.metric_cards);
                                        let sources = body.source_list.join(", ");
                                        let chart_id_here = format!("{id}-chart");
                                        let field_here = pick_open() && chart_focus() == chart_id_here;
                                        let picked_here = if field_here { pick_now.clone() } else { None };
                                        let field_css = match (picked_here.as_ref(), body.chart_block.as_ref()) {
                                            (Some(range), Some(block)) => format!(
                                                "margin-left: clamp(0px, calc({:.1}% - 120px), calc(100% - 240px));",
                                                pick_center_pct(range, block)
                                            ),
                                            _ => String::new(),
                                        };
                                        let field_input_id = pick_input_id.clone();
                                        let focus_input_id = pick_input_id.clone();
                                        rsx! {
                                            article { key: "{id}", class: "kit-rca__answer", "aria-label": "Answer",
                                                p { class: "kit-rca__text", "{body.text}" }
                                                if !metrics.is_empty() {
                                                    ul { class: "kit-rca__metrics",
                                                        for (m, card) in metrics.iter().enumerate() {
                                                            li { key: "{m}", class: "kit-rca__metric", title: "{card.full}",
                                                                span { class: "kit-rca__metric-label", "{card.label}" }
                                                                span { class: "kit-rca__metric-value", "{card.short}" }
                                                                if let Some(change) = card.change.clone() {
                                                                    span { class: "kit-rca__metric-change", "{change}" }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                if let Some(block) = body.chart_block.clone() {
                                                    ChartPanel {
                                                        block,
                                                        chart_key: format!("{id}"),
                                                        active,
                                                        pick_start,
                                                        pick_end,
                                                        picked: picked_here.clone(),
                                                        on_pick_ready: move |(range, chart_id): (FocusRange, String)| {
                                                            pick.set(Some(range));
                                                            chart_focus.set(chart_id);
                                                            pick_open.set(true);
                                                            focus_after_frame(&focus_input_id);
                                                        },
                                                    }
                                                }
                                                if field_here {
                                                    if let Some(range) = picked_here.clone() {
                                                        div { class: "kit-rca__pickwrap",
                                                            div { class: "kit-rca__pickfield", role: "group", "aria-label": "Picked range: {range.label_text}", style: "{field_css}",
                                                                p { class: "kit-rca__pickinfo", "{picked_text}" }
                                                                input { id: "{field_input_id}", class: "kit-rca__input", r#type: "text", "aria-label": "Question about the picked range", placeholder: "Ask about this part",
                                                                    value: "{draft()}",
                                                                    oninput: move |e| draft.set(e.value()),
                                                                    onkeydown: {
                                                                        let range = range.clone();
                                                                        move |event: KeyboardEvent| {
                                                                            event.stop_propagation();
                                                                            match event.key() {
                                                                                Key::Enter => {
                                                                                    event.prevent_default();
                                                                                    send(draft(), Some(range.clone()));
                                                                                }
                                                                                Key::Escape => {
                                                                                    event.prevent_default();
                                                                                    pick.set(None);
                                                                                    pick_open.set(false);
                                                                                    focus_after_frame(&chart_focus());
                                                                                }
                                                                                _ => {}
                                                                            }
                                                                        }
                                                                    },
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                if let Some(caption) = body.caption_text.clone() {
                                                    p { class: "kit-rca__caption", "{caption}" }
                                                }
                                                if !sources.is_empty() {
                                                    p { class: "kit-rca__sources", "Sources: {sources}" }
                                                }
                                                if !body.follow_up_list.is_empty() {
                                                    ul { class: "kit-rca__followups",
                                                        for (f, follow) in body.follow_up_list.iter().enumerate() {
                                                            li { key: "{f}",
                                                                button { class: "kit-rca__chip", r#type: "button",
                                                                    onclick: {
                                                                        let follow = follow.clone();
                                                                        move |_| draft.set(follow.clone())
                                                                    },
                                                                    "{follow}"
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
                        if working() {
                            p { class: "kit-rca__working",
                                span { class: "kit-rca__pulse", "aria-hidden": "true" }
                                "Reading sources…"
                            }
                        }
                    }
                }

                form { class: "kit-rca__composer",
                    onsubmit: move |event: FormEvent| {
                        event.prevent_default();
                        send(draft(), None);
                    },
                    textarea { class: "kit-rca__input kit-rca__input--area", "aria-label": "Ask a revenue question", rows: "2", value: "{draft()}",
                        oninput: move |e| draft.set(e.value()),
                        onkeydown: move |event: KeyboardEvent| {
                            if matches!(event.key(), Key::Enter) && !event.modifiers().shift() {
                                event.prevent_default();
                                send(draft(), None);
                            }
                        },
                    }
                    button { class: "kit-rca__btn kit-rca__btn--primary", r#type: "submit", disabled: draft().trim().is_empty(), "Send" }
                }
            }
            p { class: "kit-sr", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}

/// A chart in an answer. Line charts take the pick keys; a split column takes Enter to pick it.
#[component]
fn ChartPanel(
    block: ChartBlock,
    chart_key: String,
    mut active: Signal<Option<usize>>,
    mut pick_start: Signal<Option<usize>>,
    mut pick_end: Signal<usize>,
    picked: Option<FocusRange>,
    on_pick_ready: EventHandler<(FocusRange, String)>,
) -> Element {
    let chart_id = format!("{chart_key}-chart");
    match block {
        ChartBlock::Trend {
            category_labels,
            current_values,
            previous_values,
            marker,
        } => {
            let series = vec![
                TrendSeries {
                    name: "This period".into(),
                    values: current_values.iter().map(|v| Some(*v)).collect(),
                    tone: SeriesTone::Ink,
                    emphasis: true,
                    is_earlier: false,
                },
                TrendSeries {
                    name: "Last year".into(),
                    values: previous_values.iter().map(|v| Some(*v)).collect(),
                    tone: SeriesTone::Muted,
                    emphasis: false,
                    is_earlier: true,
                },
            ];
            rsx! {
                div { class: "kit-rca__chart", id: "{chart_id}",
                    if let Some(m) = marker.clone() {
                        p { class: "kit-rca__marker", "{m}" }
                    }
                    SeriesTrendChart {
                        x_labels: category_labels,
                        series,
                        variant: ChartVariant::Area,
                        height: 180.0,
                        label: "Revenue trend against last year",
                        fmt_value: group_digits,
                        axis: AxisMode::On,
                        active: Some(*active.read()),
                        on_active_change: move |next: Option<usize>| active.set(next),
                    }
                }
            }
        }
        ChartBlock::Forecast {
            category_labels,
            actual_values,
            forecast_values,
            target_value,
        } => {
            let (actual, continuation) = forecast_series(&actual_values, &forecast_values);
            let mut series = vec![
                TrendSeries {
                    name: "Actual".into(),
                    values: actual,
                    tone: SeriesTone::Ink,
                    emphasis: true,
                    is_earlier: false,
                },
                TrendSeries {
                    name: "Forecast".into(),
                    values: continuation,
                    tone: SeriesTone::Ok,
                    emphasis: false,
                    is_earlier: true,
                },
            ];
            let target = target_value.map(|t| format!("Target {}", group_digits(t)));
            if let Some(t) = target_value {
                series.push(TrendSeries {
                    name: "Target".into(),
                    values: vec![Some(t); category_labels.len()],
                    tone: SeriesTone::Warn,
                    emphasis: false,
                    is_earlier: true,
                });
            }
            let labels_for_keys = category_labels.clone();
            let point_count = category_labels.len();
            let band = picked
                .clone()
                .map(|r| (r.from_point, r.to_point))
                .or_else(|| pick_start().map(|s| (s, pick_end())));
            let band_css = band
                .filter(|_| point_count > 0)
                .map(|(from, to)| band_style(from, to, point_count));
            let chart_id_for_keys = chart_id.clone();
            rsx! {
                div { class: "kit-rca__chart kit-rca__chart--line", id: "{chart_id}", tabindex: "0",
                    onkeydown: move |event: KeyboardEvent| {
                        let count = labels_for_keys.len();
                        match event.key() {
                            Key::Enter => {
                                event.prevent_default();
                                match pick_start() {
                                    None => {
                                        let at = active().unwrap_or(0);
                                        pick_start.set(Some(at));
                                        pick_end.set(at);
                                    }
                                    Some(start) => {
                                        let (lo, hi) = clamp_range(start, pick_end(), count);
                                        on_pick_ready.call((line_focus("forecast", lo, hi, &labels_for_keys), chart_id_for_keys.clone()));
                                        pick_start.set(None);
                                    }
                                }
                            }
                            Key::ArrowRight if pick_start().is_some() => {
                                event.prevent_default();
                                pick_end.set((pick_end() + 1).min(count.saturating_sub(1)));
                            }
                            Key::ArrowLeft if pick_start().is_some() => {
                                event.prevent_default();
                                pick_end.set(pick_end().saturating_sub(1));
                            }
                            _ => {}
                        }
                    },
                    if let Some(t) = target.clone() {
                        p { class: "kit-rca__marker", "{t}" }
                    }
                    if let Some(css) = band_css.clone() {
                        div { class: "kit-rca__band", "aria-hidden": "true", style: "{css}" }
                    }
                    SeriesTrendChart {
                        x_labels: category_labels,
                        series,
                        variant: ChartVariant::Line,
                        height: 180.0,
                        label: "Forecast with likely range",
                        fmt_value: compact_number,
                        axis: AxisMode::On,
                        active: Some(*active.read()),
                        on_active_change: move |next: Option<usize>| active.set(next),
                    }
                    ul { class: "kit-rca__legend", "aria-hidden": "true",
                        li { span { class: "kit-rca__swatch", "data-tone": "ink" } "Actual" }
                        li { span { class: "kit-rca__swatch", "data-tone": "ok", "data-dashed": "true" } "Forecast" }
                        if target.is_some() {
                            li { span { class: "kit-rca__swatch", "data-tone": "warn", "data-dashed": "true" } "Target" }
                        }
                    }
                }
            }
        }
        ChartBlock::Split {
            title_text,
            previous_total,
            groups,
        } => {
            let max_total = groups
                .iter()
                .map(split_total)
                .fold(0.0_f64, f64::max)
                .max(1.0);
            let previous = previous_total.map(|t| format!("Previous {}", group_digits(t)));
            let mut part_names: Vec<String> = Vec::new();
            for group in &groups {
                for part in &group.parts {
                    if !part_names.contains(&part.entry_name) {
                        part_names.push(part.entry_name.clone());
                    }
                }
            }
            rsx! {
                figure { class: "kit-rca__split", id: "{chart_id}", "aria-label": "{title_text}",
                    figcaption { class: "kit-rca__split-title", "{title_text}" }
                    ul { class: "kit-rca__legend", "aria-hidden": "true",
                        for (pi, name) in part_names.iter().enumerate() {
                            li { key: "{pi}", span { class: "kit-rca__swatch", "data-tone": part_tone(pi) } "{name}" }
                        }
                    }
                    if let Some(p) = previous.clone() {
                        p { class: "kit-rca__marker", "{p}" }
                    }
                    div { class: "kit-rca__columns",
                        for (gi, group) in groups.iter().enumerate() {
                            {
                                let total = split_total(group);
                                let height = total / max_total * 100.0;
                                let name = group.entry_name.clone();
                                let total_text = group_digits(total);
                                let parts_text = group
                                    .parts
                                    .iter()
                                    .map(|p| format!("{} {}", p.entry_name, group_digits(p.entry_value)))
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                let aria = format!("{name}: {total_text} total, {parts_text}");
                                let chart_id_for_pick = chart_id.clone();
                                rsx! {
                                    div { key: "{gi}", class: if picked.as_ref().is_some_and(|r| r.from_point == gi) { "kit-rca__col is-picked" } else { "kit-rca__col" },
                                        tabindex: "0", role: "button", "aria-label": "{aria}",
                                        onkeydown: {
                                            let name = name.clone();
                                            let total_text = total_text.clone();
                                            move |event: KeyboardEvent| {
                                                if matches!(event.key(), Key::Enter) {
                                                    event.prevent_default();
                                                    let focus = FocusRange {
                                                        chart_kind: "split".into(),
                                                        from_point: gi,
                                                        to_point: gi,
                                                        label_text: name.clone(),
                                                        detail_text: format!("{total_text} total"),
                                                    };
                                                    on_pick_ready.call((focus, chart_id_for_pick.clone()));
                                                }
                                            }
                                        },
                                        span { class: "kit-rca__col-total", "{total_text}" }
                                        div { class: "kit-rca__stack", style: "height: {height:.1}%;",
                                            for (pi, part) in group.parts.iter().enumerate() {
                                                span { key: "{pi}", class: "kit-rca__seg", "data-tone": part_tone(pi),
                                                    style: "flex: {part.entry_value.max(0.0)} 1 0;",
                                                    title: "{part.entry_name}: {group_digits(part.entry_value)}",
                                                }
                                            }
                                        }
                                        span { class: "kit-rca__col-name", "{name}" }
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
