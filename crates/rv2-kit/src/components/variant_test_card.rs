//! Variant test card: a compact summary of an A/B or multi-arm test. Folded it shows the
//! name, status and leader; unfolded it sets every arm against the control for a chosen
//! metric, and lets a person pick a winner and act on that choice.
//!
//! Design choice: the card does not compute statistical confidence. `confidence` is
//! carried from the caller and not drawn.
//! Design choice: hand over raw numbers; each metric's `format` does the display work.
//! Design choice: `reveal: Hover` opens on pointer enter and closes on leave; a click
//! toggles in either mode. Hover is ignored in `Tap` mode.
//! Design choice: `chosen_arm` and `is_open` are `Option` controlled props. `chosen_arm`
//! is `Option<Option<_>>`, so the host can hold "no winner".
//! Not implemented: the roll of values between metrics; values re-render. Arm detail
//! beyond the value and delta is not shown.
//! Browser checks: the detail fade, the hover reveal and its leave-close, the touch tap
//! outside to fold, the arm emphasis in the head chart.

use std::collections::BTreeMap;

use dioxus::prelude::*;

use crate::components::series_trend_chart::{
    SeriesTone, finite_or_zero, focus_element, signed_percent, wrap_index,
};
use crate::internal::{KIT_CSS, class_list, next_id};
use rv2_affect::button::{Button, ButtonVariant};

const CSS: Asset = asset!("/assets/variant-test-card.css");

#[derive(Clone, Debug, PartialEq)]
pub struct Arm {
    pub arm_id: String,
    pub arm_letter: char,
    pub arm_name: String,
    pub readings: BTreeMap<String, f64>,
    pub is_control: bool,
    pub sample_size: Option<u64>,
    pub confidence: BTreeMap<String, f64>,
    pub badge_tone: Option<SeriesTone>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Metric {
    pub metric_id: String,
    pub metric_label: String,
    pub format: Option<fn(f64) -> String>,
    pub lower_wins: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StatusChip {
    pub chip_text: String,
    pub chip_tone: SeriesTone,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ActionLabel {
    Static(String),
    /// Receives the winning arm.
    ForArm(fn(&Arm) -> String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResultAction {
    pub action_text: ActionLabel,
    pub on_run: EventHandler<Arm>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Reveal {
    #[default]
    Hover,
    Tap,
}

/// Index of the control arm: the one flagged `is_control`, else the first.
pub fn control_index(arms: &[Arm]) -> Option<usize> {
    arms.iter()
        .position(|a| a.is_control)
        .or(if arms.is_empty() { None } else { Some(0) })
}

/// Percent change of `value` against `control`. `None` when the control is zero.
pub fn delta_percent(value: f64, control: f64) -> Option<f64> {
    if control == 0.0 || !control.is_finite() || !value.is_finite() {
        None
    } else {
        Some((value - control) / control.abs() * 100.0)
    }
}

/// Whether a delta leads. With `lower_wins`, a decrease leads.
pub fn is_favourable(delta: f64, lower_wins: bool) -> bool {
    if lower_wins { delta < 0.0 } else { delta > 0.0 }
}

/// Index of the leading arm for `metric`. Arms without a reading do not lead; ties keep
/// the first.
pub fn leader_index(arms: &[Arm], metric: &Metric) -> Option<usize> {
    let mut best: Option<(usize, f64)> = None;
    for (i, arm) in arms.iter().enumerate() {
        let Some(value) = arm
            .readings
            .get(&metric.metric_id)
            .copied()
            .filter(|v| v.is_finite())
        else {
            continue;
        };
        let better = match best {
            None => true,
            Some((_, b)) => {
                if metric.lower_wins {
                    value < b
                } else {
                    value > b
                }
            }
        };
        if better {
            best = Some((i, value));
        }
    }
    best.map(|(i, _)| i)
}

fn default_format(value: f64) -> String {
    format!("{:.1}", finite_or_zero(value))
}

/// A compact test summary that unfolds to every arm against the control.
#[component]
pub fn VariantTestCard(
    #[props(into)] test_name: String,
    arms: Vec<Arm>,
    metric_options: Vec<Metric>,
    #[props(default)] active_metric: Option<String>,
    #[props(default)] starting_metric: Option<String>,
    #[props(default)] on_metric_change: Option<EventHandler<String>>,
    #[props(default)] chosen_arm: Option<Option<String>>,
    #[props(default)] starting_choice: Option<String>,
    #[props(default)] on_choose: Option<EventHandler<Option<String>>>,
    #[props(default)] status_chip: Option<StatusChip>,
    #[props(default)] status_aside: Option<String>,
    #[props(default)] result_action: Option<ResultAction>,
    #[props(default = "Pick a winner".to_string(), into)] empty_prompt: String,
    #[props(default)] reveal: Reveal,
    #[props(default)] is_open: Option<bool>,
    #[props(default)] starting_open: bool,
    #[props(default)] on_toggle: Option<EventHandler<bool>>,
    #[props(default, into)] extra_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-variant-{}", next_id()));
    let mut metric_local = use_signal(|| starting_metric.clone());
    let mut choice_local = use_signal(|| starting_choice.clone());
    let mut open_local = use_signal(|| starting_open);
    let mut hovered_arm = use_signal(|| None::<usize>);

    let metric_controlled = active_metric.is_some();
    let choice_controlled = chosen_arm.is_some();
    let open_controlled = is_open.is_some();

    let metric_id = active_metric
        .clone()
        .or_else(|| metric_local())
        .filter(|id| metric_options.iter().any(|m| m.metric_id == *id))
        .or_else(|| metric_options.first().map(|m| m.metric_id.clone()));
    let metric = metric_id
        .as_ref()
        .and_then(|id| metric_options.iter().find(|m| m.metric_id == *id))
        .cloned();
    let chosen: Option<String> = match &chosen_arm {
        Some(value) => value.clone(),
        None => choice_local(),
    };
    let open = is_open.unwrap_or(open_local());
    let class = class_list("kit-variant", &extra_class);
    let head_id = format!("{base}-head");
    let escape_head = head_id.clone();

    let control = control_index(&arms);
    let leader = metric.as_ref().and_then(|m| leader_index(&arms, m));
    let leader_name = leader
        .map(|i| arms[i].arm_name.clone())
        .unwrap_or_else(|| "none yet".to_string());
    let fmt: fn(f64) -> String = metric
        .as_ref()
        .and_then(|m| m.format)
        .unwrap_or(default_format);
    let lower = metric.as_ref().map(|m| m.lower_wins).unwrap_or(false);
    let max_value = match &metric {
        Some(m) => arms
            .iter()
            .filter_map(|a| a.readings.get(&m.metric_id).copied())
            .filter(|v| v.is_finite())
            .fold(0.0_f64, |acc, v| acc.max(v.abs())),
        None => 0.0,
    };
    let status_text = status_chip
        .as_ref()
        .map(|c| c.chip_text.clone())
        .unwrap_or_default();
    let head_name = format!("{test_name}, {status_text}, leader {leader_name}");
    let chosen_index = chosen
        .as_ref()
        .and_then(|id| arms.iter().position(|a| a.arm_id == *id));
    let chosen_arm_value = chosen_index.map(|i| arms[i].clone());
    let reveal_hover = reveal == Reveal::Hover;

    let mut set_open = move |next: bool| {
        if !open_controlled {
            open_local.set(next);
        }
        if let Some(handler) = &on_toggle {
            handler.call(next);
        }
    };
    let choose = move |next: Option<String>| {
        if !choice_controlled {
            choice_local.set(next.clone());
        }
        if let Some(handler) = &on_choose {
            handler.call(next);
        }
    };
    let mut pick_metric = move |id: String| {
        if !metric_controlled {
            metric_local.set(Some(id.clone()));
        }
        if let Some(handler) = &on_metric_change {
            handler.call(id);
        }
    };

    let mut arm_rows: Vec<Element> = Vec::new();
    for (idx, arm) in arms.iter().enumerate() {
        let value = metric
            .as_ref()
            .and_then(|m| arm.readings.get(&m.metric_id).copied())
            .filter(|v| v.is_finite());
        let value_text = match value {
            Some(v) => fmt(v),
            None => "–".to_string(),
        };
        let delta = match (value, control.and_then(|c| arms.get(c))) {
            (Some(v), Some(base_arm)) if !std::ptr::eq(arm, base_arm) => metric
                .as_ref()
                .and_then(|m| base_arm.readings.get(&m.metric_id).copied())
                .and_then(|c| delta_percent(v, c)),
            _ => None,
        };
        let is_base = control == Some(idx);
        let delta_text = if is_base {
            "Base".to_string()
        } else {
            delta.map(signed_percent).unwrap_or_else(|| "–".to_string())
        };
        let favourable = delta.map(|d| is_favourable(d, lower)).unwrap_or(false);
        let tone = if is_base {
            "muted"
        } else if favourable {
            "ok"
        } else if delta.is_some_and(|d| d != 0.0) {
            "fault"
        } else {
            "muted"
        };
        let width = match value {
            Some(v) if max_value > 0.0 => format!("width: {:.1}%", v.abs() / max_value * 100.0),
            _ => "width: 0%".to_string(),
        };
        let is_chosen = chosen.as_deref() == Some(arm.arm_id.as_str());
        let letter = arm.arm_letter.to_string();
        let badge_letter = if is_chosen {
            "✓".to_string()
        } else {
            letter.clone()
        };
        let name = arm.arm_name.clone();
        let aria = format!("{name}, {value_text}, {delta_text}");
        let row_id = format!("{base}-arm-{idx}");
        let click_id = arm.arm_id.clone();
        let key_id = arm.arm_id.clone();
        let mut choose_click = choose;
        let mut choose_key = choose;
        let is_focus_row = is_chosen || (chosen.is_none() && idx == 0);
        arm_rows.push(rsx! {
            div {
                key: "{row_id}",
                id: "{row_id}",
                role: "radio",
                class: "kit-variant__arm",
                "aria-checked": if is_chosen { "true" } else { "false" },
                "aria-label": "{aria}",
                tabindex: if is_focus_row { "0" } else { "-1" },
                "data-tone": tone,
                "data-chosen": is_chosen.then_some("true"),
                "data-base": is_base.then_some("true"),
                onmouseenter: move |_| hovered_arm.set(Some(idx)),
                onmouseleave: move |_| hovered_arm.set(None),
                onclick: move |_| choose_click(if is_chosen { None } else { Some(click_id.clone()) }),
                onkeydown: move |event: KeyboardEvent| {
                    if matches!(&event.key(), Key::Character(c) if c == " ") || event.key() == Key::Enter {
                        event.prevent_default();
                        choose_key(if is_chosen { None } else { Some(key_id.clone()) });
                    }
                },
                span { class: "kit-variant__badge", "aria-hidden": "true", "{badge_letter}" }
                span { class: "kit-variant__name", "{name}" }
                span { class: "kit-variant__value", "{value_text}" }
                span { class: "kit-variant__delta", "{delta_text}" }
                span { class: "kit-variant__bar-track", "aria-hidden": "true",
                    span { class: "kit-variant__bar", style: width }
                }
            }
        });
    }

    let action_element = match (chosen_arm_value.clone(), result_action.clone()) {
        (Some(winner), Some(action)) => {
            let label = match &action.action_text {
                ActionLabel::Static(text) => text.clone(),
                ActionLabel::ForArm(f) => f(&winner),
            };
            let on_run = action.on_run;
            Some(rsx! {
                Button {
                    variant: ButtonVariant::Primary,
                    label: label,
                    onclick: move |_| on_run.call(winner.clone()),
                }
            })
        }
        _ => None,
    };
    let prompt_visible = action_element.is_none();
    let metric_count = metric_options.len();
    let metric_choices: Vec<(String, String)> = metric_options
        .iter()
        .map(|m| (m.metric_id.clone(), m.metric_label.clone()))
        .collect();
    let selected_metric_index = metric_id
        .as_ref()
        .and_then(|id| metric_choices.iter().position(|(mid, _)| mid == id))
        .unwrap_or(0);
    let metric_key_base = base.clone();
    let chart_bars: Vec<(String, String, &'static str, bool)> = arms
        .iter()
        .enumerate()
        .map(|(idx, arm)| {
            let value = metric
                .as_ref()
                .and_then(|m| arm.readings.get(&m.metric_id).copied())
                .filter(|v| v.is_finite())
                .unwrap_or(0.0);
            let height = if max_value > 0.0 {
                value.abs() / max_value * 100.0
            } else {
                0.0
            };
            let tone = match arm.badge_tone {
                Some(t) => t.as_str(),
                None => "muted",
            };
            (
                format!("height: {height:.1}%"),
                arm.arm_letter.to_string(),
                tone,
                hovered_arm() == Some(idx),
            )
        })
        .collect();
    let leader_share = leader
        .and_then(|i| arms[i].sample_size)
        .map(|n| format!("n={n}"))
        .unwrap_or_default();
    let chip_tone = status_chip
        .as_ref()
        .map(|c| c.chip_tone.as_str())
        .unwrap_or("muted");
    let aside = status_aside.clone();
    let prompt_text = empty_prompt.clone();
    let chosen_name = chosen_arm_value
        .as_ref()
        .map(|a| a.arm_name.clone())
        .unwrap_or_default();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section {
            class: class,
            "data-open": open.then_some("true"),
            "data-reveal": if reveal_hover { "hover" } else { "tap" },
            onmouseenter: move |_| {
                if reveal_hover {
                    set_open(true);
                }
            },
            onmouseleave: move |_| {
                if reveal_hover {
                    set_open(false);
                }
            },
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Escape && open {
                    event.prevent_default();
                    set_open(false);
                    focus_element(&escape_head);
                }
            },
            button {
                id: "{head_id}",
                r#type: "button",
                class: "kit-variant__head",
                "aria-expanded": if open { "true" } else { "false" },
                "aria-label": "{head_name}",
                onclick: move |_| set_open(!open),
                div { class: "kit-variant__titles",
                    span { class: "kit-variant__title", "{test_name}" }
                    if let Some(chip) = status_chip.clone() {
                        span { class: "kit-variant__chip", "data-tone": chip_tone, "{chip.chip_text}" }
                    }
                    if let Some(text) = aside.clone() {
                        span { class: "kit-variant__aside", "{text}" }
                    }
                }
                span { class: "kit-variant__leader", "Leader: {leader_name}" }
                span { class: "kit-variant__mini", "aria-hidden": "true",
                    for (idx, (style, letter, tone, lit)) in chart_bars.iter().cloned().enumerate() {
                        span { key: "{idx}", class: "kit-variant__mini-bar", style: style, "data-tone": tone, "data-lit": lit.then_some("true"), "{letter}" }
                    }
                }
            }
            if open {
                div { class: "kit-variant__detail",
                    if metric_count > 1 {
                        div {
                            class: "kit-variant__metrics",
                            role: "radiogroup",
                            "aria-label": "Metric",
                            onkeydown: {
                                let ids = metric_choices.clone();
                                let key_base = metric_key_base.clone();
                                move |event: KeyboardEvent| {
                                    let next = match event.key() {
                                        Key::ArrowRight => Some(wrap_index(selected_metric_index, metric_count, true)),
                                        Key::ArrowLeft => Some(wrap_index(selected_metric_index, metric_count, false)),
                                        _ => None,
                                    };
                                    if let Some(index) = next {
                                        event.prevent_default();
                                        pick_metric(ids[index].0.clone());
                                        focus_element(&format!("{key_base}-metric-{index}"));
                                    }
                                }
                            },
                            for (idx, (mid, label)) in metric_choices.iter().cloned().enumerate() {
                                button {
                                    key: "{mid}",
                                    id: "{metric_key_base}-metric-{idx}",
                                    r#type: "button",
                                    role: "radio",
                                    class: "kit-variant__metric",
                                    "aria-checked": if idx == selected_metric_index { "true" } else { "false" },
                                    tabindex: if idx == selected_metric_index { "0" } else { "-1" },
                                    onclick: {
                                        let id = mid.clone();
                                        move |_| pick_metric(id.clone())
                                    },
                                    "{label}"
                                }
                            }
                        }
                    }
                    div { class: "kit-variant__arms", role: "radiogroup", "aria-label": "Arms",
                        {arm_rows.into_iter()}
                    }
                    div { class: "kit-variant__foot",
                        p { class: "kit-variant__prompt", role: "status", "aria-live": "polite",
                            if prompt_visible { "{prompt_text}" }
                        }
                        if let Some(element) = action_element {
                            {element}
                        }
                    }
                    span { class: "kit-variant__sr", role: "status", "aria-live": "polite",
                        if !chosen_name.is_empty() { "Winner: {chosen_name}" }
                    }
                    if !leader_share.is_empty() {
                        span { class: "kit-variant__sample", "{leader_share}" }
                    }
                }
            }
        }
    }
}
