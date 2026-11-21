//! Asks a person whether an agent may carry out one action; folds to a one-line outcome with time left.
//!
//! Design choice: the spec's "ask again" is the control shown after an expiry; it calls
//! `on_decision_change(None)` like Revoke and Change do.
//! Design choice: expiry is detected during render. The host callback fires once from an effect,
//! and the expired line stays until the person or host acts.
//! Design choice: `grant_outcome` is controlled when `Some` (`Some(None)` means awaiting an answer);
//! `default_outcome` is the uncontrolled start.
//! Design choice: `window_choice` is controlled through `on_window_change`, since the spec lists no callback for it.
//! Browser checks: focus lands in the card on mount; the countdown bar drains in real time;
//! Enter and Escape need a hardware keyboard; touch target sizes follow `pointer: coarse`.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, next_id};

const CSS: Asset = asset!("/assets/access-grant-prompt.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RiskLevel {
    Safe,
    #[default]
    Risky,
    Destructive,
}

impl RiskLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Risky => "risky",
            Self::Destructive => "destructive",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PermissionWindow {
    #[default]
    Once,
    Timed,
    Always,
}

impl PermissionWindow {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Once => "once",
            Self::Timed => "timed",
            Self::Always => "always",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Requester {
    pub name: String,
    pub model_name: Option<String>,
    /// Image URL; initials are shown when absent.
    pub avatar: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ImpactItem {
    pub item_id: String,
    pub item_label: String,
    pub item_meta: Option<String>,
    /// A short glyph, shown before the label.
    pub item_mark: Option<String>,
}

/// A decision on a request. Epoch seconds, supplied by the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Decision {
    pub allowed: bool,
    pub window: PermissionWindow,
    pub decided_at: i64,
    pub valid_until: Option<i64>,
}

const ALL_WINDOWS: [PermissionWindow; 3] = [
    PermissionWindow::Once,
    PermissionWindow::Timed,
    PermissionWindow::Always,
];

/// Windows on offer for a risk level, in fixed order. `Always` is withheld for destructive actions.
/// Falls back to `Once` when nothing else survives.
pub fn offered_windows(risk: RiskLevel, allowed: &[PermissionWindow]) -> Vec<PermissionWindow> {
    let out: Vec<PermissionWindow> = ALL_WINDOWS
        .into_iter()
        .filter(|window| allowed.contains(window))
        .filter(|window| !(risk == RiskLevel::Destructive && *window == PermissionWindow::Always))
        .collect();
    if out.is_empty() {
        vec![PermissionWindow::Once]
    } else {
        out
    }
}

pub fn allow_label(window: PermissionWindow, timed_minutes: u32) -> String {
    match window {
        PermissionWindow::Once => "Allow once".to_string(),
        PermissionWindow::Always => "Allow always".to_string(),
        PermissionWindow::Timed if timed_minutes == 60 => "Allow for one hour".to_string(),
        PermissionWindow::Timed => format!("Allow for {timed_minutes} minutes"),
    }
}

pub fn window_label(window: PermissionWindow, timed_minutes: u32) -> String {
    match window {
        PermissionWindow::Once => "Once".to_string(),
        PermissionWindow::Always => "Always".to_string(),
        PermissionWindow::Timed if timed_minutes == 60 => "One hour".to_string(),
        PermissionWindow::Timed => format!("{timed_minutes} min"),
    }
}

/// The expiry instant for a grant; only timed grants expire.
pub fn valid_until_for(
    window: PermissionWindow,
    decided_at: i64,
    timed_minutes: u32,
) -> Option<i64> {
    match window {
        PermissionWindow::Timed => Some(decided_at + i64::from(timed_minutes) * 60),
        _ => None,
    }
}

pub fn remaining_secs(valid_until: i64, now: i64) -> i64 {
    (valid_until - now).max(0)
}

/// Strictly after `valid_until`: the grant is still valid at the instant itself.
pub fn is_expired(decision: &Decision, now: i64) -> bool {
    decision.valid_until.is_some_and(|until| now > until)
}

/// Share of a timed grant still left, 1.0 at the decision and 0.0 at expiry.
pub fn remaining_fraction(decided_at: i64, valid_until: i64, now: i64) -> f64 {
    let span = (valid_until - decided_at).max(1) as f64;
    (remaining_secs(valid_until, now) as f64 / span).clamp(0.0, 1.0)
}

pub fn format_remaining(secs: i64) -> String {
    let secs = secs.max(0);
    if secs >= 3600 {
        format!("{}h {:02}m left", secs / 3600, (secs % 3600) / 60)
    } else if secs >= 60 {
        format!("{}m left", secs / 60)
    } else {
        format!("{secs}s left")
    }
}

/// Rows shown and rows hidden behind the "more" line.
pub fn split_impact(total: usize, visible: usize) -> (usize, usize) {
    let shown = total.min(visible);
    (shown, total - shown)
}

pub fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|word| word.chars().next())
        .take(2)
        .flat_map(char::to_uppercase)
        .collect()
}

fn focus_element(id: &str) {
    document::eval(&format!("document.getElementById({id:?})?.focus();"));
}

fn emit(
    mut internal: Signal<Option<Decision>>,
    on_change: &Option<EventHandler<Option<Decision>>>,
    next: Option<Decision>,
) {
    internal.set(next);
    if let Some(handler) = on_change {
        handler.call(next);
    }
}

/// Permission card: waiting for a choice, or folded to a one-line outcome.
#[component]
pub fn AccessGrantPrompt(
    requester: Requester,
    #[props(into)] requested_action: String,
    #[props(default, into)] target_label: Option<String>,
    #[props(default)] target_mark: Option<String>,
    #[props(default)] risk_level: RiskLevel,
    #[props(default, into)] agent_rationale: Option<String>,
    #[props(default)] impact_items: Vec<ImpactItem>,
    #[props(default = "What will happen".to_string(), into)] impact_heading: String,
    #[props(default = 3)] impact_visible_count: usize,
    #[props(default = ALL_WINDOWS.to_vec())] allowed_windows: Vec<PermissionWindow>,
    #[props(default)] window_choice: Option<PermissionWindow>,
    #[props(default)] default_window: PermissionWindow,
    #[props(default)] on_window_change: Option<EventHandler<PermissionWindow>>,
    #[props(default = 90)] timed_minutes: u32,
    #[props(default)] grant_outcome: Option<Option<Decision>>,
    #[props(default)] default_outcome: Option<Decision>,
    #[props(default)] on_decision_change: Option<EventHandler<Option<Decision>>>,
    #[props(default)] current_time: i64,
) -> Element {
    let id = use_hook(|| format!("kit-agp-{}", next_id()));
    let title_id = format!("{id}-title");
    let rationale_id = format!("{id}-why");
    let internal = use_signal(|| default_outcome);
    let mut internal_window = use_signal(|| default_window);

    let outcome: Option<Decision> = match grant_outcome {
        Some(value) => value,
        None => internal(),
    };
    let offered = offered_windows(risk_level, &allowed_windows);
    let chosen = window_choice.unwrap_or_else(|| internal_window());
    let window = if offered.contains(&chosen) {
        chosen
    } else {
        offered[0]
    };
    let expired = outcome.is_some_and(|decision| is_expired(&decision, current_time));

    {
        let on_change = on_decision_change.clone();
        use_effect(use_reactive((&expired,), move |(expired,)| {
            if expired {
                if let Some(handler) = &on_change {
                    handler.call(None);
                }
            }
        }));
    }
    let focus_id = id.clone();
    use_effect(move || focus_element(&focus_id));

    let title = format!("{} wants to {}", requester.name, requested_action);
    let risk_word = match risk_level {
        RiskLevel::Safe => None,
        RiskLevel::Risky => Some("Risky"),
        RiskLevel::Destructive => Some("Destructive"),
    };
    let allow_text = allow_label(window, timed_minutes);
    let initials_text = initials(&requester.name);
    let avatar_src = requester.avatar.clone();
    let model_name = requester.model_name.clone();
    let (shown, hidden) = split_impact(impact_items.len(), impact_visible_count);
    let shown_items: Vec<ImpactItem> = impact_items.iter().take(shown).cloned().collect();
    let hidden_text = format!("{hidden} more");
    let hint_text = "Enter allows, Escape denies";
    let root_id = id.clone();
    let risk_attr = risk_level.as_str();
    let destructive = risk_level == RiskLevel::Destructive;
    let impact_empty = shown_items.is_empty();

    if let Some(decision) = outcome {
        let (outcome_text, outcome_key, action_text) = if expired {
            ("Expired".to_string(), "expired", "Ask again")
        } else if decision.allowed {
            let text = match decision.window {
                PermissionWindow::Once => "Allowed once".to_string(),
                PermissionWindow::Always => "Allowed always".to_string(),
                PermissionWindow::Timed => format!(
                    "Allowed for {}",
                    window_label(PermissionWindow::Timed, timed_minutes).to_lowercase()
                ),
            };
            (text, "allowed", "Revoke")
        } else {
            ("Denied".to_string(), "denied", "Change")
        };
        let countdown = decision.valid_until.filter(|_| !expired).map(|until| {
            (
                format_remaining(remaining_secs(until, current_time)),
                remaining_fraction(decision.decided_at, until, current_time),
            )
        });
        let bar_style = countdown
            .as_ref()
            .map(|(_, fraction)| format!("transform: scaleX({fraction:.4})"));
        let countdown_text = countdown.map(|(text, _)| text);
        let action_handler = on_decision_change.clone();
        return rsx! {
            document::Link { rel: "stylesheet", href: KIT_CSS }
            document::Link { rel: "stylesheet", href: CSS }
            div { id: "{id}", class: "kit-agp kit-agp--decided", "data-outcome": "{outcome_key}",
                div { class: "kit-agp__line", role: "status",
                    span { class: "kit-agp__dot", aria_hidden: "true" }
                    span { class: "kit-agp__outcome", "{outcome_text}" }
                    if let Some(text) = countdown_text {
                        span { class: "kit-agp__countdown", "{text}" }
                    }
                    button {
                        r#type: "button",
                        class: "kit-agp__link",
                        "data-action": "{outcome_key}",
                        onclick: move |_| emit(internal, &action_handler, None),
                        "{action_text}"
                    }
                }
                if let Some(style) = bar_style {
                    div { class: "kit-agp__bar", aria_hidden: "true",
                        span { class: "kit-agp__bar-fill", style: "{style}" }
                    }
                }
            }
        };
    }

    let choices = if offered.len() > 1 {
        let choice_items: Vec<(PermissionWindow, String, &'static str)> = offered
            .iter()
            .map(|option| {
                (
                    *option,
                    window_label(*option, timed_minutes),
                    option.as_str(),
                )
            })
            .collect();
        let key_offered = offered.clone();
        let on_window_keys = on_window_change.clone();
        let on_window_clicks = on_window_change.clone();
        let key_root = root_id.clone();
        Some(rsx! {
            div { class: "kit-agp__choices", role: "radiogroup", aria_label: "Duration",
                onkeydown: move |event: KeyboardEvent| {
                    let step: isize = match event.key() {
                        Key::ArrowRight | Key::ArrowDown => 1,
                        Key::ArrowLeft | Key::ArrowUp => -1,
                        _ => return,
                    };
                    event.prevent_default();
                    let at = key_offered.iter().position(|w| *w == window).unwrap_or(0) as isize;
                    let len = key_offered.len() as isize;
                    let next = key_offered[(at + step).rem_euclid(len) as usize];
                    internal_window.set(next);
                    if let Some(handler) = &on_window_keys {
                        handler.call(next);
                    }
                    focus_element(&format!("{key_root}-w-{}", next.as_str()));
                },
                for (option, label, key) in choice_items {
                    button {
                        key: "{key}",
                        r#type: "button",
                        role: "radio",
                        id: "{id}-w-{key}",
                        class: "kit-agp__choice",
                        "aria-checked": if option == window { "true" } else { "false" },
                        tabindex: if option == window { "0" } else { "-1" },
                        onclick: {
                            let handler = on_window_clicks.clone();
                            move |_| {
                                internal_window.set(option);
                                if let Some(handler) = &handler {
                                    handler.call(option);
                                }
                            }
                        },
                        "{label}"
                    }
                }
            }
        })
    } else {
        None
    };

    let deny_handler = on_decision_change.clone();
    let allow_handler = on_decision_change.clone();
    let key_allow_handler = on_decision_change.clone();
    let key_deny_handler = on_decision_change.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div {
            id: "{id}",
            class: "kit-agp",
            role: "dialog",
            aria_labelledby: "{title_id}",
            aria_describedby: "{rationale_id}",
            tabindex: "-1",
            "data-risk": risk_attr,
            onkeydown: move |event: KeyboardEvent| {
                match event.key() {
                    Key::Enter => {
                        event.prevent_default();
                        emit(internal, &key_allow_handler, Some(Decision {
                            allowed: true,
                            window,
                            decided_at: current_time,
                            valid_until: valid_until_for(window, current_time, timed_minutes),
                        }));
                    }
                    Key::Escape => {
                        event.prevent_default();
                        emit(internal, &key_deny_handler, Some(Decision {
                            allowed: false,
                            window,
                            decided_at: current_time,
                            valid_until: None,
                        }));
                    }
                    _ => {}
                }
            },
            header { class: "kit-agp__head",
                span { class: "kit-agp__avatar", aria_hidden: "true",
                    if let Some(src) = avatar_src {
                        img { src: "{src}", alt: "" }
                    } else {
                        span { "{initials_text}" }
                    }
                }
                div { class: "kit-agp__heading",
                    h3 { id: "{title_id}", class: "kit-agp__title", "{title}" }
                    if let Some(model) = model_name {
                        span { class: "kit-agp__model", "{model}" }
                    }
                }
                if let Some(label) = risk_word {
                    span { class: "kit-agp__risk", "data-risk": risk_attr, "{label}" }
                }
            }
            if let Some(label) = target_label {
                p { class: "kit-agp__target",
                    if let Some(mark) = target_mark {
                        span { class: "kit-agp__mark", aria_hidden: "true", "{mark}" }
                    }
                    span { "{label}" }
                }
            }
            if let Some(why) = agent_rationale {
                p { id: "{rationale_id}", class: "kit-agp__rationale", "{why}" }
            }
            if !impact_empty {
                section { class: "kit-agp__impact",
                    h4 { class: "kit-agp__impact-title", "{impact_heading}" }
                    ul { class: "kit-agp__impact-list",
                        for (index, item) in shown_items.iter().enumerate() {
                            li { key: "{item.item_id}", class: "kit-agp__impact-row", style: "--kit-i: {index}",
                                if let Some(mark) = item.item_mark.clone() {
                                    span { class: "kit-agp__mark", aria_hidden: "true", "{mark}" }
                                }
                                span { class: "kit-agp__impact-label", "{item.item_label}" }
                                if let Some(meta) = item.item_meta.clone() {
                                    span { class: "kit-agp__meta", "{meta}" }
                                }
                            }
                        }
                    }
                    if hidden > 0 {
                        p { class: "kit-agp__more", "{hidden_text}" }
                    }
                }
            }
            if let Some(group) = choices {
                {group}
            }
            div { class: "kit-agp__actions",
                button {
                    r#type: "button",
                    class: "kit-agp__deny",
                    onclick: move |_| emit(internal, &deny_handler, Some(Decision {
                        allowed: false,
                        window,
                        decided_at: current_time,
                        valid_until: None,
                    })),
                    "Deny"
                }
                button {
                    r#type: "button",
                    class: "kit-agp__allow",
                    "data-risk": risk_attr,
                    "data-destructive": destructive.then_some("true"),
                    aria_label: "{allow_text}",
                    onclick: move |_| emit(internal, &allow_handler, Some(Decision {
                        allowed: true,
                        window,
                        decided_at: current_time,
                        valid_until: valid_until_for(window, current_time, timed_minutes),
                    })),
                    span { key: "{allow_text}", class: "kit-agp__allow-label", "{allow_text}" }
                }
            }
            p { class: "kit-agp__hints", aria_hidden: "true", "{hint_text}" }
        }
    }
}
