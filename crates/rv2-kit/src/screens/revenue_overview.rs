//! Revenue overview: headline figures, a multi-line daily chart, a ledger of components, ranked items,
//! a sessions comparison and one average with its comparison line, under a page header with a period menu.
//!
//! Design choice: the day chart and the sessions chart are `SeriesTrendChart`, which carries the keyboard day cursor.
//! Design choice: the sidebar is drawn inline as page navigation so the screen stands alone, without the workspace frame.
//! Design choice: key-figure tone is `change_sign` against `good_when`, not `MetricTile`, because the spec's change
//! line is free text with a printed sign word, which `MetricTile` cannot carry.
//! Design choice: the average's comparison line is the mean of the period's daily values, drawn dashed.

use dioxus::prelude::*;

use crate::components::series_trend_chart::{
    ChartVariant, SeriesTone, SeriesTrendChart, TrendSeries,
};
use crate::internal::*;

const CSS: Asset = asset!("/assets/revenue-overview.css");

/// Sidebar entries, in order. The third is active until the person picks another.
pub const NAV_ITEMS: [&str; 6] = [
    "Home",
    "Inbox",
    "Revenue",
    "Customers",
    "Reports",
    "Settings",
];

/// Periods the selector offers when the host gives none.
pub const DEFAULT_PERIODS: [&str; 4] = ["Last 7 days", "Last 30 days", "Last 90 days", "This year"];

/// Direction of a figure, and of a change against the previous period.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChangeSign {
    Up,
    Down,
    #[default]
    Flat,
}

impl ChangeSign {
    /// The printed sign word; a visual cue is never the only signal.
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

/// Positive, negative or neutral reading of a change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Positive,
    Negative,
    Neutral,
}

impl Tone {
    pub fn key(self) -> &'static str {
        match self {
            Self::Positive => "positive",
            Self::Negative => "negative",
            Self::Neutral => "neutral",
        }
    }
}

/// A change is positive when its sign equals `good_when`, negative when it does not, neutral when flat.
pub fn change_tone(change_sign: ChangeSign, good_when: ChangeSign) -> Tone {
    match change_sign {
        ChangeSign::Flat => Tone::Neutral,
        sign if sign == good_when => Tone::Positive,
        _ => Tone::Negative,
    }
}

/// Bar heights as a percentage of the largest value; all zero when no value is positive.
pub fn bar_percentages(values: &[f64]) -> Vec<f64> {
    let max = values
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .fold(0.0_f64, f64::max);
    values
        .iter()
        .map(|v| {
            if max <= 0.0 || !v.is_finite() {
                0.0
            } else {
                (v.max(0.0) / max * 100.0).clamp(0.0, 100.0)
            }
        })
        .collect()
}

/// A ranked row's share bar width, clamped to the track.
pub fn share_width(share_pct: f64) -> f64 {
    if share_pct.is_finite() {
        share_pct.clamp(0.0, 100.0)
    } else {
        0.0
    }
}

/// Points of a sparkline as `x,y` pairs in a `width` by `height` box; a flat line when the values are constant.
pub fn sparkline_points(values: &[f64], width: f64, height: f64) -> String {
    if values.is_empty() {
        return String::new();
    }
    let lo = values.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = hi - lo;
    let step = if values.len() > 1 {
        width / (values.len() - 1) as f64
    } else {
        0.0
    };
    values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let y = if span <= 0.0 {
                height / 2.0
            } else {
                height - (v - lo) / span * height
            };
            format!("{:.1},{:.1}", i as f64 * step, y)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Sum of a day series; non-finite values count as zero.
pub fn period_total(daily: &[f64]) -> f64 {
    daily.iter().filter(|v| v.is_finite()).sum()
}

/// Mean of a day series, or 0 for an empty one.
pub fn daily_mean(daily: &[f64]) -> f64 {
    if daily.is_empty() {
        0.0
    } else {
        period_total(daily) / daily.len() as f64
    }
}

/// Y position of the mean inside the 80-unit average box, matching `sparkline_points`.
pub fn mean_line_y(daily: &[f64]) -> f64 {
    let lo = daily.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = daily.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = hi - lo;
    if daily.is_empty() || span <= 0.0 {
        return 40.0;
    }
    80.0 - (daily_mean(daily) - lo) / span * 80.0
}

/// Whole-unit total with grouped digits, e.g. `48,210`.
pub fn format_total(value: f64) -> String {
    let rounded = value.round() as i64;
    let digits = rounded.abs().to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    if rounded < 0 { format!("-{out}") } else { out }
}

/// The period a sessions line stands for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionPeriod {
    This,
    Previous,
}

impl SessionPeriod {
    pub fn label(self) -> &'static str {
        match self {
            Self::This => "This period",
            Self::Previous => "Previous period",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Account {
    pub name: String,
    pub avatar_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct KeyFigure {
    pub item_name: String,
    pub item_value: String,
    pub change_text: String,
    pub change_sign: ChangeSign,
    pub good_when: ChangeSign,
    pub trend_bars: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChannelSeries {
    pub name: String,
    pub daily: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BreakdownRow {
    pub item_name: String,
    pub item_value: String,
    pub trend_points: Vec<f64>,
    pub trend_direction: ChangeSign,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RankedItem {
    pub item_name: String,
    pub item_value: String,
    pub share_pct: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SessionsSeries {
    pub item_name: SessionPeriod,
    pub daily: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AverageValue {
    pub current: String,
    pub previous: String,
    pub daily: Vec<f64>,
}

fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

fn tone_for(index: usize) -> SeriesTone {
    [
        SeriesTone::Ink,
        SeriesTone::Ok,
        SeriesTone::Warn,
        SeriesTone::Muted,
    ][index % 4]
}

/// Revenue overview page: header, figures, charts and lists, with loading and empty states.
#[component]
pub fn RevenueOverview(
    #[props(default = "Juniper Works".to_string(), into)] workspace_name: String,
    account: Account,
    #[props(default = "Last 7 days".to_string(), into)] period_label: String,
    #[props(default)] period_options: Vec<String>,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] key_figures: Vec<KeyFigure>,
    #[props(default)] day_labels: Vec<String>,
    #[props(default)] channel_series: Vec<ChannelSeries>,
    #[props(default, into)] chart_total: String,
    #[props(default, into)] chart_change_text: String,
    #[props(default = ChangeSign::Flat)] chart_change_sign: ChangeSign,
    #[props(default)] breakdown_rows: Vec<BreakdownRow>,
    #[props(default)] ranked_items: Vec<RankedItem>,
    #[props(default)] sessions_series: Vec<SessionsSeries>,
    #[props(default)] average_value: Option<AverageValue>,
    #[props(default)] on_connect: EventHandler<()>,
    #[props(default)] on_navigate: EventHandler<String>,
    #[props(default)] on_export: EventHandler<()>,
    #[props(default)] on_save_report: EventHandler<()>,
    #[props(default)] on_period_change: EventHandler<String>,
) -> Element {
    let base = use_hook(|| format!("kit-rev-{}", next_id()));
    let options: Vec<String> = if period_options.is_empty() {
        DEFAULT_PERIODS.iter().map(|p| p.to_string()).collect()
    } else {
        period_options
    };
    let mut period = use_signal(|| period_label.clone());
    let mut menu_open = use_signal(|| false);
    let mut menu_index = use_signal(|| 0_usize);
    let mut nav_open = use_signal(|| false);
    let mut active_nav = use_signal(|| 2_usize);
    let mut announce = use_signal(String::new);

    let trigger_id = format!("{base}-period");
    let list_id = format!("{base}-periods");
    let day_count = channel_series
        .iter()
        .map(|c| c.daily.len())
        .max()
        .unwrap_or(0);
    let labels: Vec<String> = if day_labels.len() == day_count {
        day_labels.clone()
    } else {
        (1..=day_count).map(|d| format!("Day {d}")).collect()
    };
    let chart_series: Vec<TrendSeries> = channel_series
        .iter()
        .enumerate()
        .map(|(i, c)| TrendSeries {
            name: c.name.clone(),
            values: c.daily.iter().map(|v| Some(*v)).collect(),
            tone: tone_for(i),
            emphasis: i == 0,
            is_earlier: false,
        })
        .collect();
    let session_chart: Vec<TrendSeries> = sessions_series
        .iter()
        .map(|s| TrendSeries {
            name: s.item_name.label().to_string(),
            values: s.daily.iter().map(|v| Some(*v)).collect(),
            tone: if s.item_name == SessionPeriod::This {
                SeriesTone::Ink
            } else {
                SeriesTone::Muted
            },
            emphasis: s.item_name == SessionPeriod::This,
            is_earlier: s.item_name == SessionPeriod::Previous,
        })
        .collect();
    let session_days = sessions_series
        .iter()
        .map(|s| s.daily.len())
        .max()
        .unwrap_or(0);
    let session_labels: Vec<String> = (1..=session_days).map(|d| format!("Day {d}")).collect();
    let figure_views: Vec<(KeyFigure, Tone, Vec<f64>)> = key_figures
        .iter()
        .map(|f| {
            (
                f.clone(),
                change_tone(f.change_sign, f.good_when),
                bar_percentages(&f.trend_bars),
            )
        })
        .collect();
    let chart_total_text = if chart_total.is_empty() {
        format_total(channel_series.iter().map(|c| period_total(&c.daily)).sum())
    } else {
        chart_total
    };
    let nav_class = if nav_open() {
        "kit-rev__nav is-open"
    } else {
        "kit-rev__nav"
    };
    let active_index = active_nav();
    let period_now = period();

    let on_trigger = {
        let list_id = list_id.clone();
        let options = options.clone();
        move |_| {
            if menu_open() {
                menu_open.set(false);
            } else {
                let current = options.iter().position(|o| *o == period()).unwrap_or(0);
                menu_index.set(current);
                menu_open.set(true);
                focus_after_frame(&list_id);
            }
        }
    };
    let on_menu_keys = {
        let trigger_id = trigger_id.clone();
        let options = options.clone();
        move |event: KeyboardEvent| {
            let last = options.len().saturating_sub(1);
            match event.key() {
                Key::ArrowDown => {
                    event.prevent_default();
                    menu_index.set((menu_index() + 1).min(last));
                }
                Key::ArrowUp => {
                    event.prevent_default();
                    menu_index.set(menu_index().saturating_sub(1));
                }
                Key::Home => {
                    event.prevent_default();
                    menu_index.set(0);
                }
                Key::End => {
                    event.prevent_default();
                    menu_index.set(last);
                }
                Key::Enter => {
                    event.prevent_default();
                    let picked = options[menu_index()].clone();
                    announce.set(format!("Period: {picked}"));
                    period.set(picked.clone());
                    on_period_change.call(picked);
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
        section { class: "kit-rev", "aria-labelledby": "{base}-title",
            div { class: "kit-rev__frame",
                button { class: "kit-rev__menu-btn", r#type: "button", "aria-expanded": "{nav_open()}",
                    "aria-controls": "{base}-nav",
                    onclick: move |_| nav_open.toggle(),
                    "Menu"
                }
                nav { id: "{base}-nav", class: "{nav_class}", "aria-label": "Sidebar",
                    p { class: "kit-rev__ws", "{workspace_name}" }
                    ul { class: "kit-rev__nav-list",
                        for (i, item) in NAV_ITEMS.iter().enumerate() {
                            li { key: "{item}",
                                button { class: "kit-rev__nav-item", r#type: "button",
                                    "aria-current": if i == active_index { "page" } else { "false" },
                                    onclick: {
                                        let label = item.to_string();
                                        move |_| {
                                            active_nav.set(i);
                                            nav_open.set(false);
                                            on_navigate.call(label.clone());
                                        }
                                    },
                                    "{item}"
                                }
                            }
                        }
                    }
                    div { class: "kit-rev__account",
                        span { class: "kit-rev__avatar", "aria-hidden": "true",
                            if let Some(src) = account.avatar_url.clone() {
                                img { src: "{src}", alt: "" }
                            } else {
                                "{initials(&account.name)}"
                            }
                        }
                        span { class: "kit-rev__account-name", "{account.name}" }
                    }
                }
                div { class: "kit-rev__page",
                    header { class: "kit-rev__head",
                        h1 { id: "{base}-title", class: "kit-rev__title", "Revenue overview" }
                        div { class: "kit-rev__actions",
                            div { class: "kit-rev__period",
                                button { id: "{trigger_id}", class: "kit-rev__btn kit-rev__period-btn", r#type: "button",
                                    "aria-haspopup": "listbox", "aria-expanded": "{menu_open()}",
                                    "aria-controls": "{list_id}",
                                    onclick: on_trigger,
                                    span { class: "kit-rev__period-label", "{period_now}" }
                                    span { class: "kit-rev__caret", "aria-hidden": "true", "▾" }
                                }
                                if menu_open() {
                                    ul { id: "{list_id}", class: "kit-rev__menu", role: "listbox", tabindex: "-1",
                                        "aria-label": "Period",
                                        "aria-activedescendant": "{list_id}-{menu_index()}",
                                        onkeydown: on_menu_keys,
                                        for (i, option) in options.iter().enumerate() {
                                            li { id: "{list_id}-{i}", key: "{option}", role: "option",
                                                "aria-selected": if i == menu_index() { "true" } else { "false" },
                                                class: if i == menu_index() { "kit-rev__opt is-active" } else { "kit-rev__opt" },
                                                onclick: {
                                                    let option = option.clone();
                                                    move |_| {
                                                        announce.set(format!("Period: {option}"));
                                                        period.set(option.clone());
                                                        on_period_change.call(option.clone());
                                                        menu_open.set(false);
                                                    }
                                                },
                                                "{option}"
                                            }
                                        }
                                    }
                                }
                            }
                            button { class: "kit-rev__btn", r#type: "button", onclick: move |_| on_export.call(()), "Export" }
                            button { class: "kit-rev__btn kit-rev__btn--primary", r#type: "button", onclick: move |_| on_save_report.call(()), "Save as report" }
                        }
                    }

                    if is_empty {
                        div { class: "kit-rev__empty",
                            h2 { class: "kit-rev__empty-title", "No revenue yet" }
                            p { class: "kit-rev__empty-text", "Figures, charts and rankings appear here once the first payment arrives." }
                            button { class: "kit-rev__btn kit-rev__btn--primary", r#type: "button", onclick: move |_| on_connect.call(()), "Connect a payment account" }
                        }
                    } else if is_loading {
                        div { class: "kit-rev__grid", "aria-busy": "true",
                            div { class: "kit-rev__figures",
                                for i in 0..4 {
                                    div { key: "{i}", class: "kit-rev__card kit-rev__card--figure",
                                        span { class: "kit-rev__ph kit-rev__ph--line" }
                                        span { class: "kit-rev__ph kit-rev__ph--big" }
                                        span { class: "kit-rev__ph kit-rev__ph--line" }
                                    }
                                }
                            }
                            section { class: "kit-rev__card kit-rev__card--chart", "aria-label": "Revenue by channel",
                                h2 { class: "kit-rev__card-title", "Revenue by channel" }
                                span { class: "kit-rev__ph kit-rev__ph--plot" }
                            }
                            section { class: "kit-rev__card", "aria-label": "Breakdown",
                                h2 { class: "kit-rev__card-title", "Breakdown" }
                                for i in 0..3 {
                                    span { key: "{i}", class: "kit-rev__ph kit-rev__ph--row" }
                                }
                            }
                            section { class: "kit-rev__card", "aria-label": "Top items",
                                h2 { class: "kit-rev__card-title", "Top items" }
                                for i in 0..3 {
                                    span { key: "{i}", class: "kit-rev__ph kit-rev__ph--row" }
                                }
                            }
                        }
                    } else {
                        div { class: "kit-rev__grid",
                            div { class: "kit-rev__figures",
                                for (i, (figure, tone, bars)) in figure_views.iter().enumerate() {
                                    section { key: "{i}", class: "kit-rev__card kit-rev__card--figure", role: "group",
                                        "aria-label": "{figure.item_name}",
                                        "data-tone": tone.key(),
                                        span { class: "kit-rev__figure-label", "{figure.item_name}" }
                                        span { class: "kit-rev__figure-value", "{figure.item_value}" }
                                        span { class: "kit-rev__change", "data-sign": figure.change_sign.key(),
                                            span { class: "kit-rev__sign", "{figure.change_sign.word()}" }
                                            " {figure.change_text}"
                                        }
                                        span { class: "kit-rev__bars", "aria-hidden": "true",
                                            for (b, pct) in bars.iter().enumerate() {
                                                span { key: "{b}", class: "kit-rev__bar", style: "height: {pct:.1}%;" }
                                            }
                                        }
                                    }
                                }
                            }

                            section { class: "kit-rev__card kit-rev__card--chart", "aria-labelledby": "{base}-chart",
                                div { class: "kit-rev__card-head",
                                    div {
                                        h2 { id: "{base}-chart", class: "kit-rev__card-title", "Revenue by channel" }
                                        p { class: "kit-rev__total",
                                            span { class: "kit-rev__total-value", "{chart_total_text}" }
                                            " this period, "
                                            span { class: "kit-rev__change", "data-sign": chart_change_sign.key(),
                                                span { class: "kit-rev__sign", "{chart_change_sign.word()}" }
                                                " {chart_change_text}"
                                            }
                                        }
                                    }
                                    ul { class: "kit-rev__legend",
                                        for (i, c) in channel_series.iter().enumerate() {
                                            li { key: "{c.name}",
                                                span { class: "kit-rev__swatch", "data-tone": tone_for(i).as_str(), "aria-hidden": "true" }
                                                "{c.name}"
                                            }
                                        }
                                    }
                                }
                                if day_count > 0 {
                                    SeriesTrendChart {
                                        x_labels: labels,
                                        series: chart_series,
                                        variant: ChartVariant::Line,
                                        height: 240.0,
                                        label: "Revenue by channel per day",
                                    }
                                }
                            }

                            section { class: "kit-rev__card", "aria-labelledby": "{base}-breakdown",
                                h2 { id: "{base}-breakdown", class: "kit-rev__card-title", "Breakdown" }
                                ul { class: "kit-rev__ledger",
                                    for (i, row) in breakdown_rows.iter().enumerate() {
                                        li { key: "{i}", class: "kit-rev__ledger-row",
                                            span { class: "kit-rev__ledger-name", "{row.item_name}" }
                                            span { class: "kit-rev__ledger-value", "{row.item_value}" }
                                            svg { class: "kit-rev__spark", view_box: "0 0 60 18", "aria-hidden": "true", preserve_aspect_ratio: "none",
                                                polyline { points: "{sparkline_points(&row.trend_points, 60.0, 18.0)}", "data-direction": row.trend_direction.key() }
                                            }
                                        }
                                    }
                                }
                            }

                            section { class: "kit-rev__card", "aria-labelledby": "{base}-ranked",
                                h2 { id: "{base}-ranked", class: "kit-rev__card-title", "Top items" }
                                ol { class: "kit-rev__ranked",
                                    for (i, item) in ranked_items.iter().enumerate() {
                                        li { key: "{i}", class: "kit-rev__ranked-row",
                                            span { class: "kit-rev__ranked-name", "{item.item_name}" }
                                            span { class: "kit-rev__ranked-value", "{item.item_value}" }
                                            span { class: "kit-rev__track", "aria-hidden": "true",
                                                span { class: "kit-rev__fill", style: "width: {share_width(item.share_pct):.1}%;" }
                                            }
                                        }
                                    }
                                }
                            }

                            section { class: "kit-rev__card", "aria-labelledby": "{base}-sessions",
                                div { class: "kit-rev__card-head",
                                    h2 { id: "{base}-sessions", class: "kit-rev__card-title", "Sessions" }
                                    p { class: "kit-rev__total",
                                        for (i, s) in sessions_series.iter().enumerate() {
                                            span { key: "{i}", class: "kit-rev__session-total",
                                                "{s.item_name.label()}: "
                                                strong { "{format_total(period_total(&s.daily))}" }
                                            }
                                        }
                                    }
                                }
                                if !session_chart.is_empty() {
                                    SeriesTrendChart {
                                        x_labels: session_labels,
                                        series: session_chart,
                                        variant: ChartVariant::Line,
                                        height: 180.0,
                                        label: "Sessions compared with the previous period",
                                    }
                                }
                            }

                            if let Some(avg) = average_value {
                                section { class: "kit-rev__card", "aria-labelledby": "{base}-average",
                                    h2 { id: "{base}-average", class: "kit-rev__card-title", "Average order" }
                                    p { class: "kit-rev__avg",
                                        span { class: "kit-rev__total-value", "{avg.current}" }
                                        span { class: "kit-rev__avg-prev", "Previous {avg.previous}" }
                                    }
                                    svg { class: "kit-rev__avg-chart", view_box: "0 0 300 80", preserve_aspect_ratio: "none", "aria-hidden": "true",
                                        polyline { class: "kit-rev__avg-line", points: "{sparkline_points(&avg.daily, 300.0, 80.0)}" }
                                        line { class: "kit-rev__avg-ref", x1: "0", x2: "300", y1: "{mean_line_y(&avg.daily):.1}", y2: "{mean_line_y(&avg.daily):.1}" }
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
