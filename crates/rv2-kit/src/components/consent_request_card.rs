//! Asks a person to allow one specific action an agent is about to take; expires when nobody answers.
//!
//! Design choice: a promise-returning `on_grant` cannot be expressed as a Rust callback. The host
//! drives the busy state with `processing`, and sets `decision_state` once its work settles.
//! Design choice: the countdown is kept in seconds and advanced by a one-second task. Hover and
//! focus pause it. The task reads the live state through a signal, so a host reset restarts it.
//! Design choice: the decided time is shown as UTC `HH:MM`; a host wanting local time passes an
//! offset-adjusted `current_time`.
//! Browser checks: the countdown freezes on hover and focus; Ctrl or Cmd+Enter grants and Escape
//! declines; the resolved row takes focus when focus was inside the card.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, next_id, sleep};

const CSS: Asset = asset!("/assets/consent-request-card.css");
const IS_MAC: &str = "const nav = navigator; return /mac|iphone|ipad|ipod/i.test(nav.userAgentData?.platform || nav.platform || nav.userAgent || '');";
/// Seconds left at which the countdown switches to the warning tint.
pub const WARNING_SECS: u32 = 10;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ConsentRisk {
    #[default]
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DecisionState {
    #[default]
    Pending,
    Granted,
    Declined,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeclineCause {
    User,
    Timeout,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ControlLayout {
    #[default]
    Inline,
    Footer,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GrantOptions {
    pub keep_for_action: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub struct DetailRow {
    pub key_label: String,
    pub value: String,
}

/// Button and label wording.
#[derive(Clone, PartialEq, Debug)]
pub struct ConsentText {
    pub grant: String,
    pub decline: String,
    pub edit: String,
    pub remember: String,
    pub group_label: String,
}

impl Default for ConsentText {
    fn default() -> Self {
        Self {
            grant: "Allow".into(),
            decline: "Deny".into(),
            edit: "Edit".into(),
            remember: "Keep for this action type".into(),
            group_label: "Agent request".into(),
        }
    }
}

pub fn risk_label(risk: ConsentRisk) -> &'static str {
    match risk {
        ConsentRisk::Low => "Low risk",
        ConsentRisk::Medium => "Medium risk",
        ConsentRisk::High => "High risk",
    }
}

pub fn risk_attr(risk: ConsentRisk) -> &'static str {
    match risk {
        ConsentRisk::Low => "low",
        ConsentRisk::Medium => "medium",
        ConsentRisk::High => "high",
    }
}

/// Outcome word for a resolved row. A timeout decline reads "Expired".
pub fn outcome_word(state: DecisionState, cause: DeclineCause) -> &'static str {
    match (state, cause) {
        (DecisionState::Granted, _) => "Granted",
        (DecisionState::Declined, DeclineCause::Timeout) => "Expired",
        (DecisionState::Declined, DeclineCause::User) => "Declined",
        (DecisionState::Pending, _) => "Pending",
    }
}

/// Path data for the outcome glyph: check for a grant, cross for a decline, clock for an expiry.
fn outcome_glyph(key: &str) -> &'static str {
    match key {
        "granted" => "M3.2 8.4l3.1 3.1 6.5-6.6",
        "declined" => "M4 4l8 8M12 4l-8 8",
        _ => "M8 1.8a6.2 6.2 0 1 0 0 12.4a6.2 6.2 0 1 0 0-12.4M8 4.6V8l2.3 1.5",
    }
}

/// `m:ss` for a countdown.
pub fn format_countdown(secs: u32) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

/// One tick of the countdown: the next remaining value, and whether the window has run out.
pub fn tick(remaining: u32, paused: bool) -> (u32, bool) {
    if paused {
        return (remaining, false);
    }
    let next = remaining.saturating_sub(1);
    (next, next == 0)
}

pub fn is_warning(remaining: u32) -> bool {
    remaining <= WARNING_SECS
}

/// UTC wall clock `HH:MM` for an epoch-seconds value.
pub fn format_clock(epoch_secs: i64) -> String {
    let day_secs = epoch_secs.rem_euclid(86_400);
    format!("{:02}:{:02}", day_secs / 3600, (day_secs % 3600) / 60)
}

/// Consent card: the request, its risk, the detail rows, and allow or deny.
#[component]
pub fn ConsentRequestCard(
    #[props(into)] request_title: String,
    #[props(default, into)] context_line: String,
    #[props(default)] risk_level: Option<ConsentRisk>,
    #[props(default)] header_mark: Option<String>,
    #[props(default)] detail_rows: Vec<DetailRow>,
    #[props(default)] preview_content: Option<Element>,
    #[props(default)] body_content: Option<Element>,
    #[props(default)] decision_state: Option<DecisionState>,
    #[props(default)] initial_decision: DecisionState,
    #[props(default)] on_grant: Option<EventHandler<GrantOptions>>,
    #[props(default)] on_decline: Option<EventHandler<DeclineCause>>,
    #[props(default)] on_amend: Option<EventHandler<()>>,
    #[props(default)] timeout_seconds: Option<u32>,
    #[props(default)] on_timeout: Option<EventHandler<()>>,
    #[props(default = true)] show_remember_option: bool,
    #[props(default)] text_overrides: ConsentText,
    #[props(default = true)] show_key_hints: bool,
    #[props(default)] band_label: Option<Element>,
    #[props(default)] control_layout: ControlLayout,
    #[props(default)] processing: bool,
    #[props(default)] current_time: i64,
    #[props(default, into)] root_class: String,
) -> Element {
    let id = use_hook(|| format!("kit-ccr-{}", next_id()));
    let title_id = format!("{id}-title");
    let context_id = format!("{id}-context");
    let mut internal = use_signal(|| initial_decision);
    let mut cause_sig = use_signal(|| DeclineCause::User);
    let mut decided_at = use_signal(|| 0_i64);
    let mut remember = use_signal(|| false);
    let mut paused = use_signal(|| false);
    let mut remaining = use_signal(|| timeout_seconds.unwrap_or(0));
    let mut cycle = use_signal(|| 0_u32);
    let mut live_state = use_signal(|| initial_decision);
    let mut refocus = use_signal(|| false);
    let mut mac = use_signal(|| false);
    use_effect(move || {
        spawn(async move {
            if let Ok(value) = document::eval(IS_MAC).await {
                mac.set(value.as_bool() == Some(true));
            }
        });
    });

    let state = decision_state.unwrap_or_else(|| internal());
    let pending = state == DecisionState::Pending;

    use_effect(use_reactive((&state,), move |(state,)| {
        live_state.set(state)
    }));
    use_effect(use_reactive(
        (&state, &timeout_seconds),
        move |(state, timeout)| {
            if state == DecisionState::Pending {
                remaining.set(timeout.unwrap_or(0));
                cycle.with_mut(|value| *value = value.wrapping_add(1));
            }
        },
    ));
    {
        let on_timeout = on_timeout.clone();
        let on_decline = on_decline.clone();
        use_effect(use_reactive((&cycle(),), move |(run,)| {
            if run == 0 || timeout_seconds.is_none() {
                return;
            }
            let on_timeout = on_timeout.clone();
            let on_decline = on_decline.clone();
            spawn(async move {
                loop {
                    sleep(1000).await;
                    if *live_state.peek() != DecisionState::Pending {
                        break;
                    }
                    let (next, expired) = tick(remaining(), paused());
                    remaining.set(next);
                    if expired {
                        if let Some(handler) = &on_timeout {
                            handler.call(());
                        }
                        cause_sig.set(DeclineCause::Timeout);
                        decided_at.set(current_time);
                        internal.set(DecisionState::Declined);
                        if let Some(handler) = &on_decline {
                            handler.call(DeclineCause::Timeout);
                        }
                        break;
                    }
                }
            });
        }));
    }

    let grant = {
        let handler = on_grant.clone();
        move |keep: bool| {
            if !pending || processing {
                return;
            }
            refocus.set(true);
            decided_at.set(current_time);
            internal.set(DecisionState::Granted);
            if let Some(h) = &handler {
                h.call(GrantOptions {
                    keep_for_action: keep,
                });
            }
        }
    };
    let decline = {
        let handler = on_decline.clone();
        move |cause: DeclineCause| {
            if !pending || processing {
                return;
            }
            refocus.set(true);
            cause_sig.set(cause);
            decided_at.set(current_time);
            internal.set(DecisionState::Declined);
            if let Some(h) = &handler {
                h.call(cause);
            }
        }
    };

    let title = request_title.clone();
    let grant_text = text_overrides.grant.clone();
    let decline_text = text_overrides.decline.clone();
    let edit_text = text_overrides.edit.clone();
    let remember_text = text_overrides.remember.clone();
    let group_text = text_overrides.group_label.clone();
    let risk_text = risk_level.map(risk_label);
    let risk_key = risk_level.map(risk_attr).unwrap_or("none");
    let countdown_on = pending && timeout_seconds.is_some();
    let warning = countdown_on && is_warning(remaining());
    let countdown = countdown_on.then(|| format_countdown(remaining()));
    let expiry_text = format!("Expires in {} seconds", remaining());
    let expiry_id = format!("{id}-expiry");
    let described_by = [
        (!context_line.is_empty()).then(|| context_id.clone()),
        countdown_on.then(|| expiry_id.clone()),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ");
    let resolved_word = outcome_word(state, cause_sig());
    let resolved_key = match resolved_word {
        "Granted" => "granted",
        "Declined" => "declined",
        _ => "expired",
    };
    let resolved_glyph = outcome_glyph(resolved_key);
    let risk_glyph = if risk_level == Some(ConsentRisk::Low) {
        "M3.2 8.4l3.1 3.1 6.5-6.6"
    } else {
        "M8 2.2L14.2 13H1.8zM8 6.6v2.8M8 11.2v.2"
    };
    let resolved_time = format_clock(decided_at());
    let grant_hint = match (show_key_hints, mac()) {
        (false, _) => "",
        (true, true) => "Cmd+Enter",
        (true, false) => "Ctrl+Enter",
    };
    let decline_hint = if show_key_hints { "Esc" } else { "" };
    let locked = processing || !pending;
    let footer = control_layout == ControlLayout::Footer;
    let root_class = if root_class.is_empty() {
        "kit-ccr".to_string()
    } else {
        format!("kit-ccr {root_class}")
    };
    let state_attr = match state {
        DecisionState::Pending => "pending",
        DecisionState::Granted => "granted",
        DecisionState::Declined => "declined",
    };
    let paused_attr = paused().then_some("true");
    let group_label = group_text;
    let edit_handler = on_amend.clone();
    let detail_pairs: Vec<(String, String)> = detail_rows
        .iter()
        .map(|row| (row.key_label.clone(), row.value.clone()))
        .collect();

    if !pending {
        return rsx! {
            document::Link { rel: "stylesheet", href: KIT_CSS }
            document::Link { rel: "stylesheet", href: CSS }
            div { id: "{id}", class: "{root_class}", "data-state": state_attr,
                div { class: "kit-ccr__resolved", role: "status", tabindex: "-1",
                    onmounted: move |event: MountedEvent| {
                        if refocus() {
                            spawn(async move {
                                let _ = event.set_focus(true).await;
                            });
                        }
                    },
                    span { class: "kit-ccr__resolved-word", "data-outcome": "{resolved_key}",
                        svg { view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.6", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
                            path { d: "{resolved_glyph}" }
                        }
                        "{resolved_word}"
                    }
                    span { class: "kit-ccr__resolved-title", "{title}" }
                    span { class: "kit-ccr__resolved-time", "{resolved_time}" }
                }
            }
        };
    }

    let mut grant_click = grant.clone();
    let mut grant_key = grant.clone();
    let mut decline_click = decline.clone();
    let mut decline_key = decline.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section {
            id: "{id}",
            class: "{root_class}",
            role: "group",
            aria_labelledby: "{title_id}",
            aria_describedby: (!described_by.is_empty()).then_some(described_by),
            "data-state": state_attr,
            "data-risk": risk_key,
            "data-layout": if footer { "footer" } else { "inline" },
            "data-paused": paused_attr,
            "aria-busy": processing.then_some("true"),
            onkeydown: move |event: KeyboardEvent| {
                let mods = event.modifiers();
                let grant_mod = if mac() { mods.meta() } else { mods.ctrl() };
                if event.key() == Key::Enter && grant_mod {
                    event.prevent_default();
                    grant_key(remember());
                } else if event.key() == Key::Escape {
                    event.prevent_default();
                    decline_key(DeclineCause::User);
                }
            },
            onmouseenter: move |_| paused.set(true),
            onmouseleave: move |_| paused.set(false),
            onfocus: move |_| paused.set(true),
            onblur: move |_| paused.set(false),
            if let Some(band) = band_label {
                div { class: "kit-ccr__band", {band} }
            }
            header { class: "kit-ccr__head",
                span { class: "kit-ccr__mark", aria_hidden: "true",
                    if let Some(mark) = header_mark {
                        "{mark}"
                    } else {
                        svg { view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round",
                            path { d: "M8 2.2l4.8 1.8v3.9c0 3-2 5.2-4.8 6.2-2.8-1-4.8-3.2-4.8-6.2V4z" }
                        }
                    }
                }
                div { class: "kit-ccr__titles",
                    h3 { id: "{title_id}", class: "kit-ccr__title", "{title}" }
                    if let Some(label) = risk_text {
                        span { class: "kit-ccr__risk", "data-risk": risk_key,
                            svg { class: "kit-ccr__glyph", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
                                path { d: "{risk_glyph}" }
                            }
                            "{label}"
                        }
                    }
                }
            }
            if !context_line.is_empty() {
                p { id: "{context_id}", class: "kit-ccr__context", "{context_line}" }
            }
            if preview_content.is_some() || body_content.is_some() {
                div { class: "kit-ccr__preview",
                    if let Some(preview) = preview_content {
                        {preview}
                    } else if let Some(body) = body_content {
                        {body}
                    }
                }
            }
            if !detail_pairs.is_empty() {
                dl { class: "kit-ccr__details",
                    for (key_label, value) in detail_pairs {
                        div { key: "{key_label}", class: "kit-ccr__row",
                            dt { "{key_label}" }
                            dd { title: "{value}", "{value}" }
                        }
                    }
                }
            }
            footer { class: "kit-ccr__foot",
                if show_remember_option {
                    label { class: "kit-ccr__remember",
                        input {
                            r#type: "checkbox",
                            checked: remember(),
                            disabled: locked,
                            onchange: move |event: FormEvent| remember.set(event.checked()),
                        }
                        span { "{remember_text}" }
                    }
                }
                if let Some(label) = countdown {
                    div { id: "{expiry_id}", class: "kit-ccr__expiry",
                        span { class: "kit-ccr__timer", "data-warning": warning.then_some("true"), aria_hidden: "true",
                            svg { view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round",
                                path { d: "M8 1.8a6.2 6.2 0 1 0 0 12.4a6.2 6.2 0 1 0 0-12.4M8 4.6V8l2.3 1.5" }
                            }
                            "{label}"
                        }
                        span { class: "kit-ccr__expiry-text", "{expiry_text}" }
                    }
                }
                div { class: "kit-ccr__actions", role: "group", aria_label: "{group_label}",
                    if let Some(edit) = edit_handler {
                        button {
                            r#type: "button",
                            class: "kit-ccr__edit",
                            disabled: locked,
                            onclick: move |_| edit.call(()),
                            "{edit_text}"
                        }
                    }
                    button {
                        r#type: "button",
                        class: "kit-ccr__deny",
                        disabled: locked,
                        onclick: move |_| decline_click(DeclineCause::User),
                        "{decline_text}"
                        if !decline_hint.is_empty() {
                            span { class: "kit-ccr__hint", aria_hidden: "true", "{decline_hint}" }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "kit-ccr__allow",
                        disabled: locked,
                        onclick: move |_| grant_click(remember()),
                        "{grant_text}"
                        if !grant_hint.is_empty() {
                            span { class: "kit-ccr__hint", aria_hidden: "true", "{grant_hint}" }
                        }
                    }
                }
            }
        }
    }
}
