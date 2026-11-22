//! Quota usage panel: consumption against a plan total, what it was spent on, open and
//! capped allowances, a daily trend, and an action that changes as the plan runs low.
//!
//! Design choice: `resets_at_ms` and `now_ms` are epoch milliseconds supplied by the
//! host. The countdown is computed from them, so rendering stays deterministic. Without
//! `now_ms` only the calendar date is shown.
//! Design choice: `shown_period` is the host's; without it the first period is kept in
//! local state. Either way the host supplies the matching breakdown, caps and trend.
//! Design choice: the action button is `rv2_affect::button::Button`. Its variant is
//! primary at or past `alert_ratio`, secondary below.
//! Design choice: the trend is a row of bars, one per day, each focusable. Days with
//! `None` draw a flat stub.
//! Not implemented: the count-up of figures and the fill from zero on first view; values
//! render at their final figure.
//! Browser checks: the meter fill and segment glide, the warning treatment at
//! `alert_ratio`, the action label swap at the cap, the countdown and date reveal, the
//! narrow layout stacking the caps.

use dioxus::prelude::*;

use crate::components::series_trend_chart::{
    IndexMove, SeriesTone, compact_number, focus_element, group_digits, non_negative, share,
    share_label, step_index, wrap_index,
};
use crate::internal::{KIT_CSS, class_list, next_id};
use rv2_affect::button::{Button, ButtonVariant};

const CSS: Asset = asset!("/assets/quota-usage-panel.css");

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

#[derive(Clone, Debug, PartialEq)]
pub struct SpendPart {
    pub spend_label: String,
    pub spend_value: f64,
    pub spend_emphasis: Option<SeriesTone>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CapMeter {
    pub cap_label: String,
    pub cap_used: f64,
    /// `None` marks an open-ended allowance.
    pub cap_total: Option<f64>,
    pub unit: Option<String>,
    pub decimals: Option<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PeriodChoice {
    pub period_id: String,
    pub period_label: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DayPoint {
    pub day_label: String,
    /// `None` is a day not yet reached.
    pub day_value: Option<f64>,
}

/// State handed to `on_action`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionState {
    Ok,
    Warn,
    Limit,
}

impl ActionState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warn => "warn",
            Self::Limit => "limit",
        }
    }
}

/// Sum of the non-negative spend values.
pub fn consumed(spend: &[SpendPart]) -> f64 {
    spend.iter().map(|p| non_negative(p.spend_value)).sum()
}

/// `Limit` at or past the plan total, `Warn` at or past `alert_ratio`, otherwise `Ok`.
pub fn meter_state(consumed: f64, plan_total: f64, alert_ratio: f64) -> ActionState {
    if plan_total <= 0.0 {
        return if consumed > 0.0 {
            ActionState::Limit
        } else {
            ActionState::Ok
        };
    }
    let ratio = share(consumed, plan_total);
    if ratio >= 1.0 {
        ActionState::Limit
    } else if ratio >= alert_ratio {
        ActionState::Warn
    } else {
        ActionState::Ok
    }
}

/// Used fraction of a capped allowance; `None` when open-ended.
pub fn cap_ratio(used: f64, total: Option<f64>) -> Option<f64> {
    match total {
        Some(total) if total > 0.0 => Some(share(used, total)),
        _ => None,
    }
}

/// Days and hours, hours and minutes, or minutes until a refresh: `3d 4h`, `5h 12m`,
/// `42m`, or `now`.
pub fn countdown_text(remaining_ms: i64) -> String {
    if remaining_ms <= 0 {
        return "now".to_string();
    }
    let minutes = remaining_ms / 60_000;
    let days = minutes / (60 * 24);
    let hours = (minutes / 60) % 24;
    let mins = minutes % 60;
    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {mins:02}m")
    } else {
        format!("{}m", mins.max(1))
    }
}

/// Calendar day of an epoch-millisecond instant (UTC) as `Sep 1`.
pub fn date_label(epoch_ms: i64) -> String {
    let days = epoch_ms.div_euclid(86_400_000);
    let (_, month, day) = civil_from_days(days);
    format!("{} {day}", MONTHS[(month - 1) as usize])
}

/// Year, month (1-12) and day of a day count since 1970-01-01.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    let year = yoe as i64 + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

fn format_decimals(value: f64, decimals: usize) -> String {
    let value = non_negative(value);
    if decimals == 0 {
        group_digits(value)
    } else {
        format!("{value:.decimals$}")
    }
}

/// Whole totals print without decimals, so `8 GB` stays `8 GB` beside a `2.4` used figure.
fn format_total(value: f64, decimals: usize) -> String {
    if non_negative(value).fract() == 0.0 {
        group_digits(non_negative(value))
    } else {
        format_decimals(value, decimals)
    }
}

fn with_unit(text: String, unit: &str) -> String {
    if unit.is_empty() {
        text
    } else {
        format!("{text} {unit}")
    }
}

/// Series tones for parts with no emphasis, in order. Adjacent entries are the most distinct
/// pairs in the series palette, which has no orange or red.
const CATEGORY_TONES: [SeriesTone; 4] = [
    SeriesTone::Ink,
    SeriesTone::Ok,
    SeriesTone::Fault,
    SeriesTone::Muted,
];

fn category_tone(position: usize) -> &'static str {
    CATEGORY_TONES[position % CATEGORY_TONES.len()].as_str()
}

/// A usage and billing panel for a metered plan.
#[component]
pub fn QuotaUsagePanel(
    plan_total: f64,
    spend_breakdown: Vec<SpendPart>,
    #[props(default)] cap_list: Vec<CapMeter>,
    #[props(default)] period_choices: Vec<PeriodChoice>,
    #[props(default)] shown_period: Option<String>,
    #[props(default)] first_period: Option<String>,
    #[props(default)] on_period_pick: Option<EventHandler<String>>,
    #[props(default)] resets_at_ms: Option<i64>,
    #[props(default)] now_ms: Option<i64>,
    #[props(default)] daily_trend: Vec<DayPoint>,
    #[props(default)] on_action: Option<EventHandler<ActionState>>,
    #[props(default = "Usage".to_string(), into)] panel_heading: String,
    #[props(default, into)] plan_label: String,
    #[props(default = "credits".to_string(), into)] meter_unit: String,
    #[props(default = 0.85)] alert_ratio: f64,
    #[props(default = "Manage plan".to_string(), into)] headroom_action: String,
    #[props(default = "Top up".to_string(), into)] ceiling_action: String,
    #[props(default, into)] extra_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-qup-{}", next_id()));
    let mut local_period = use_signal(|| None::<String>);
    let mut active_part = use_signal(|| None::<usize>);
    let mut active_day = use_signal(|| None::<usize>);

    let period_ids: Vec<String> = period_choices.iter().map(|p| p.period_id.clone()).collect();
    let controlled = shown_period.is_some();
    let selected = shown_period
        .clone()
        .or_else(|| local_period())
        .or_else(|| first_period.clone())
        .and_then(|id| period_ids.iter().position(|p| *p == id))
        .unwrap_or(0);
    let selected_id = period_ids.get(selected).cloned();

    let used = consumed(&spend_breakdown);
    let plan = non_negative(plan_total);
    let ratio = share(used, plan);
    let state = meter_state(used, plan, alert_ratio);
    let fill = format!("{:.2}", ratio.min(1.0) * 100.0);
    let percent_text = format!("{:.0}%", ratio * 100.0);
    let used_text = group_digits(used);
    let plan_figure = group_digits(plan);
    let notice = match state {
        ActionState::Ok => String::new(),
        ActionState::Warn => format!("Approaching the cap: {percent_text} of {meter_unit} used"),
        ActionState::Limit => format!("Cap reached: all {meter_unit} used"),
    };
    let value_text = format!("{used_text} of {plan_figure} {meter_unit}");
    let left_text = format!("{} {meter_unit} left", group_digits((plan - used).max(0.0)));
    let of_text = format!("of {plan_figure} {meter_unit}");
    let action_label = match state {
        ActionState::Ok => headroom_action.clone(),
        _ => ceiling_action.clone(),
    };
    let action_variant = match state {
        ActionState::Ok => ButtonVariant::Secondary,
        _ => ButtonVariant::Primary,
    };

    let unshaded: Vec<usize> = spend_breakdown
        .iter()
        .enumerate()
        .filter(|(_, part)| part.spend_emphasis.is_none())
        .map(|(i, _)| i)
        .collect();
    let segments: Vec<(String, f64, String, &'static str, String)> = spend_breakdown
        .iter()
        .enumerate()
        .map(|(i, part)| {
            let amount = non_negative(part.spend_value);
            let tone = match part.spend_emphasis {
                Some(tone) => tone.as_str(),
                None => {
                    let position = unshaded.iter().position(|&u| u == i).unwrap_or(0);
                    category_tone(position)
                }
            };
            let ratio = share(amount, used);
            (
                part.spend_label.clone(),
                amount,
                format!("{} · {}", group_digits(amount), share_label(ratio)),
                tone,
                format!("width: {:.2}%", ratio * 100.0),
            )
        })
        .collect();
    let part_active = active_part().filter(|i| *i < segments.len());
    let readout = part_active.map(|i| format!("{}: {}", segments[i].0, segments[i].2));

    let caps: Vec<(String, String, Option<f64>, String, String)> = cap_list
        .iter()
        .map(|cap| {
            let decimals = cap.decimals.unwrap_or(0);
            let unit = cap.unit.clone().unwrap_or_default();
            let consumed_text = format_decimals(cap.cap_used, decimals);
            let (value, words) = match cap.cap_total {
                Some(total) => {
                    let total_text = format_total(total, decimals);
                    let text = with_unit(format!("{consumed_text} of {total_text}"), &unit);
                    (text.clone(), text)
                }
                None => {
                    let used_text = with_unit(consumed_text.clone(), &unit);
                    (
                        format!("{used_text} · Open-ended"),
                        format!("{used_text}, open-ended"),
                    )
                }
            };
            let ratio = cap_ratio(cap.cap_used, cap.cap_total);
            let fill_style = format!("width: {:.1}%", ratio.unwrap_or(0.0).min(1.0) * 100.0);
            (cap.cap_label.clone(), value, ratio, words, fill_style)
        })
        .collect();

    let peak = daily_trend
        .iter()
        .filter_map(|d| d.day_value)
        .filter(|v| v.is_finite())
        .fold(0.0_f64, f64::max);
    let days: Vec<(usize, Option<f64>, String, String)> = daily_trend
        .iter()
        .enumerate()
        .map(|(i, day)| {
            let value = day.day_value.filter(|v| v.is_finite()).map(non_negative);
            let height = match value {
                Some(v) if peak > 0.0 => (v / peak * 100.0).max(2.0),
                _ => 0.0,
            };
            let text = match value {
                Some(v) => format!("{}: {}", day.day_label, compact_number(v)),
                None => format!("{}: not yet", day.day_label),
            };
            (i, value, format!("height: {height:.1}%"), text)
        })
        .collect();
    let day_readout = active_day().and_then(|i| days.get(i)).map(|d| d.3.clone());
    let refresh = resets_at_ms.map(|at| match now_ms {
        Some(now) => format!("Refreshes in {}", countdown_text(at - now)),
        None => format!("Refreshes {}", date_label(at)),
    });
    let refresh_date = resets_at_ms.map(date_label).unwrap_or_default();
    let class = class_list("kit-qup", &extra_class);
    let period_count = period_ids.len();
    let heading = panel_heading.clone();
    let plan_text = plan_label.clone();
    let key_base = base.clone();
    let state_name = state.as_str();
    let days_len = days.len();
    let segment_count = segments.len();

    let mut pick = move |id: String| {
        if !controlled {
            local_period.set(Some(id.clone()));
        }
        if let Some(handler) = &on_period_pick {
            handler.call(id);
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: class, "aria-label": heading.clone(), "data-state": state_name,
            header { class: "kit-qup__head",
                div { class: "kit-qup__titles",
                    h3 { class: "kit-qup__title", "{heading}" }
                    if !plan_text.is_empty() {
                        span { class: "kit-qup__plan", "{plan_text}" }
                    }
                }
                if let Some(text) = refresh.clone() {
                    span { class: "kit-qup__refresh", title: "{refresh_date}", "{text}" }
                }
            }
            if period_count > 0 {
                div {
                    class: "kit-qup__periods",
                    role: "radiogroup",
                    "aria-label": "Period",
                    onkeydown: {
                        let key_base = key_base.clone();
                        let ids = period_ids.clone();
                        move |event: KeyboardEvent| {
                            let next = match event.key() {
                                Key::ArrowRight => Some(wrap_index(selected, period_count, true)),
                                Key::ArrowLeft => Some(wrap_index(selected, period_count, false)),
                                Key::Home => Some(0),
                                Key::End => Some(period_count - 1),
                                _ => None,
                            };
                            if let Some(index) = next {
                                event.prevent_default();
                                pick(ids[index].clone());
                                focus_element(&format!("{key_base}-period-{index}"));
                            }
                        }
                    },
                    for (index, choice) in period_choices.iter().enumerate() {
                        button {
                            key: "{choice.period_id}",
                            id: "{key_base}-period-{index}",
                            r#type: "button",
                            role: "radio",
                            class: "kit-qup__period",
                            "aria-checked": if Some(&choice.period_id) == selected_id.as_ref() { "true" } else { "false" },
                            tabindex: if index == selected { "0" } else { "-1" },
                            onclick: {
                                let id = choice.period_id.clone();
                                move |_| pick(id.clone())
                            },
                            "{choice.period_label}"
                        }
                    }
                }
            }
            div { class: "kit-qup__main",
                div { class: "kit-qup__figures",
                    span { class: "kit-qup__used", "{used_text}" }
                    span { class: "kit-qup__of", "{of_text}" }
                    span { class: "kit-qup__pct", "data-state": state_name, "{percent_text}" }
                }
                div {
                    class: "kit-qup__meter",
                    role: "meter",
                    tabindex: "0",
                    "aria-label": "{meter_unit} used",
                    "aria-valuemin": "0",
                    "aria-valuemax": "{plan}",
                    "aria-valuenow": "{used}",
                    "aria-valuetext": "{value_text}",
                    onkeydown: move |event: KeyboardEvent| {
                        let movement = match event.key() {
                            Key::ArrowRight => Some(IndexMove::Next),
                            Key::ArrowLeft => Some(IndexMove::Prev),
                            Key::Escape => {
                                active_part.set(None);
                                None
                            }
                            _ => None,
                        };
                        if let Some(movement) = movement {
                            event.prevent_default();
                            active_part.set(step_index(part_active, segment_count, movement));
                        }
                    },
                    onmouseleave: move |_| active_part.set(None),
                    div { class: "kit-qup__track",
                        div { class: "kit-qup__fill", style: "width: {fill}%",
                            for (i, (label, _amount, _detail, tone, width)) in segments.iter().cloned().enumerate() {
                                span {
                                    key: "{i}",
                                    class: "kit-qup__segment",
                                    "data-tone": tone,
                                    "data-dim": part_active.is_some_and(|a| a != i).then_some("true"),
                                    title: "{label}",
                                    style: width,
                                    onmouseenter: move |_| active_part.set(Some(i)),
                                }
                            }
                        }
                    }
                    p { class: "kit-qup__readout", role: "status", "aria-live": "polite", "{readout.clone().unwrap_or_default()}" }
                }
                p { class: "kit-qup__notice", role: "status", "aria-live": "polite", "{notice}" }
                ol { class: "kit-qup__legend",
                    for (i, (label, amount, detail, tone, _width)) in segments.iter().cloned().enumerate() {
                        li {
                            key: "{i}",
                            class: "kit-qup__legend-row",
                            "data-tone": tone,
                            "data-lit": (part_active == Some(i)).then_some("true"),
                            "data-hidden": (amount <= 0.0).then_some("true"),
                            onmouseenter: move |_| active_part.set(Some(i)),
                            onmouseleave: move |_| active_part.set(None),
                            span { class: "kit-qup__swatch", "aria-hidden": "true" }
                            span { class: "kit-qup__legend-name", "{label}" }
                            span { class: "kit-qup__legend-detail", "{detail}" }
                        }
                    }
                }
            }
            if !days.is_empty() {
                div { class: "kit-qup__trend", role: "group", tabindex: "0", "aria-label": "Daily usage",
                    onkeydown: move |event: KeyboardEvent| {
                        let movement = match event.key() {
                            Key::ArrowRight => Some(IndexMove::Next),
                            Key::ArrowLeft => Some(IndexMove::Prev),
                            Key::Escape => {
                                active_day.set(None);
                                None
                            }
                            _ => None,
                        };
                        if let Some(movement) = movement {
                            event.prevent_default();
                            active_day.set(step_index(active_day(), days_len, movement));
                        }
                    },
                    onmouseleave: move |_| active_day.set(None),
                    div { class: "kit-qup__bars",
                        for (i, value, height, _text) in days.iter().cloned() {
                            span {
                                key: "{i}",
                                class: "kit-qup__day",
                                "data-stub": value.is_none().then_some("true"),
                                "data-active": (active_day() == Some(i)).then_some("true"),
                                style: height,
                                onmouseenter: move |_| active_day.set(Some(i)),
                            }
                        }
                    }
                    p { class: "kit-qup__readout", role: "status", "aria-live": "polite", "{day_readout.clone().unwrap_or_default()}" }
                }
            }
            footer { class: "kit-qup__foot",
                if !caps.is_empty() {
                    ul { class: "kit-qup__caps",
                        for (i, (label, value, ratio, words, fill_style)) in caps.iter().cloned().enumerate() {
                            li { key: "{i}", class: "kit-qup__cap",
                                div { class: "kit-qup__cap-head",
                                    span { class: "kit-qup__cap-label", "{label}" }
                                    span { class: "kit-qup__cap-value", "{value}" }
                                }
                                div {
                                    class: "kit-qup__cap-track",
                                    role: if ratio.is_some() { "meter" } else { "img" },
                                    "aria-label": "{label}",
                                    "aria-valuetext": "{words}",
                                    "data-open": ratio.is_none().then_some("true"),
                                    span { class: "kit-qup__cap-fill", style: fill_style }
                                }
                            }
                        }
                    }
                }
                div { class: "kit-qup__action",
                    span { class: "kit-qup__remaining", "{left_text}" }
                    Button {
                        variant: action_variant,
                        label: action_label,
                        onclick: move |_| {
                            if let Some(handler) = &on_action {
                                handler.call(state);
                            }
                        },
                    }
                }
            }
        }
    }
}
