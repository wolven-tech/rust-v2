//! Sets how much an agent may do without asking: three levels plus per-capability "always ask" pins.
//!
//! Design choice: the level is `u8` (0, 1 or 2), clamped on entry, because the spec's `0 | 1 | 2`
//! has no Rust literal type. `needs_confirmation` is the single rule the UI and the agent runtime share.
//! Design choice: an out-of-range level falls back to 0 (ask for everything), the safe default.
//! Browser checks: arrow keys wrap across the level radios; pins respond to Space; the group moves
//! animate in sequence.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, next_id};

const CSS: Asset = asset!("/assets/trust-level-selector.css");

pub const LEVEL_NAMES: [&str; 3] = ["Ask first", "Routine", "Autonomous"];
pub const LEVEL_DESCRIPTIONS: [&str; 3] = [
    "Asks before every action.",
    "Runs routine actions. Asks before sensitive and irreversible ones.",
    "Runs every action except those pinned to ask.",
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RiskClass {
    #[default]
    Routine,
    Sensitive,
    Irreversible,
}

impl RiskClass {
    pub fn tag(self) -> Option<&'static str> {
        match self {
            Self::Routine => None,
            Self::Sensitive => Some("Sensitive"),
            Self::Irreversible => Some("Irreversible"),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Routine => "routine",
            Self::Sensitive => "sensitive",
            Self::Irreversible => "irreversible",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Capability {
    pub id: String,
    pub capability_name: String,
    pub capability_note: Option<String>,
    pub risk_class: RiskClass,
    pub capability_mark: Option<String>,
}

/// The one rule: pinned capabilities always ask; level 0 asks for all; level 1 runs only routine
/// capabilities; level 2 runs every unpinned capability.
pub fn needs_confirmation(
    capability: &Capability,
    policy_level: u8,
    always_ask_ids: &[String],
) -> bool {
    if always_ask_ids.contains(&capability.id) {
        return true;
    }
    match policy_level {
        1 => capability.risk_class != RiskClass::Routine,
        2 => false,
        _ => true,
    }
}

/// Footer counts: (runs without confirmation, needs confirmation).
pub fn group_counts(
    capabilities: &[Capability],
    policy_level: u8,
    always_ask_ids: &[String],
) -> (usize, usize) {
    let asks = capabilities
        .iter()
        .filter(|capability| needs_confirmation(capability, policy_level, always_ask_ids))
        .count();
    (capabilities.len() - asks, asks)
}

/// Next level, wrapping at both ends.
pub fn step_level(level: u8, step: i8) -> u8 {
    (i16::from(level) + i16::from(step)).rem_euclid(3) as u8
}

pub fn clamp_level(level: u8) -> u8 {
    if level > 2 { 0 } else { level }
}

pub fn level_announcement(level: u8, runs: usize, asks: usize) -> String {
    format!(
        "Level: {}. {runs} run without confirmation, {asks} need confirmation.",
        LEVEL_NAMES[usize::from(clamp_level(level))]
    )
}

/// Capability list with three levels and per-capability pins.
#[component]
pub fn TrustLevelSelector(
    capabilities: Vec<Capability>,
    #[props(default)] policy_level: Option<u8>,
    #[props(default)] initial_policy_level: u8,
    #[props(default)] on_policy_change: Option<EventHandler<u8>>,
    #[props(default)] always_ask_ids: Option<Vec<String>>,
    #[props(default)] initial_always_ask: Vec<String>,
    #[props(default)] on_always_ask_change: Option<EventHandler<Vec<String>>>,
    #[props(default = "Agent limits".to_string(), into)] section_title: String,
    #[props(default = "Independent actions the agent may take".to_string(), into)] section_subtitle: String,
    #[props(default, into)] root_class: String,
) -> Element {
    let id = use_hook(|| format!("kit-tls-{}", next_id()));
    let mut internal_level = use_signal(|| initial_policy_level);
    let mut internal_pins = use_signal(|| initial_always_ask.clone());

    let level = clamp_level(policy_level.unwrap_or_else(|| internal_level()));
    let pins: Vec<String> = always_ask_ids.clone().unwrap_or_else(|| internal_pins());

    let (runs, asks) = group_counts(&capabilities, level, &pins);
    let announcement = level_announcement(level, runs, asks);
    let level_description = LEVEL_DESCRIPTIONS[usize::from(level)];
    let root_class = if root_class.is_empty() {
        "kit-tls".to_string()
    } else {
        format!("kit-tls {root_class}")
    };
    let count_line = format!("{runs} run without confirmation, {asks} need confirmation");
    let empty = capabilities.is_empty();
    let level_items: Vec<(u8, &'static str)> = (0..3_u8)
        .map(|index| (index, LEVEL_NAMES[usize::from(index)]))
        .collect();

    let mut runs_list: Vec<(Capability, bool)> = Vec::new();
    let mut asks_list: Vec<(Capability, bool)> = Vec::new();
    for capability in &capabilities {
        let pinned = pins.contains(&capability.id);
        if needs_confirmation(capability, level, &pins) {
            asks_list.push((capability.clone(), pinned));
        } else {
            runs_list.push((capability.clone(), pinned));
        }
    }

    let change_level = EventHandler::new(move |next: u8| {
        internal_level.set(next);
        if let Some(handler) = &on_policy_change {
            handler.call(next);
        }
    });
    let pin_current = pins.clone();
    let pin_handler = on_always_ask_change.clone();
    let toggle_pin = EventHandler::new(move |capability_id: String| {
        let mut next = pin_current.clone();
        if let Some(at) = next.iter().position(|known| *known == capability_id) {
            next.remove(at);
        } else {
            next.push(capability_id);
        }
        internal_pins.set(next.clone());
        if let Some(handler) = &pin_handler {
            handler.call(next);
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { id: "{id}", class: "{root_class}", "data-level": "{level}",
            header { class: "kit-tls__head",
                h2 { class: "kit-tls__title", "{section_title}" }
                p { class: "kit-tls__subtitle", "{section_subtitle}" }
            }
            div { class: "kit-tls__levels", role: "radiogroup", aria_label: "Independence level",
                onkeydown: move |event: KeyboardEvent| {
                    let step: i8 = match event.key() {
                        Key::ArrowRight | Key::ArrowDown => 1,
                        Key::ArrowLeft | Key::ArrowUp => -1,
                        _ => return,
                    };
                    event.prevent_default();
                    change_level.call(step_level(level, step));
                },
                for (index, name) in level_items {
                    button {
                        key: "{index}",
                        r#type: "button",
                        role: "radio",
                        class: "kit-tls__level",
                        "aria-checked": if index == level { "true" } else { "false" },
                        tabindex: if index == level { "0" } else { "-1" },
                        "data-level": "{index}",
                        onclick: move |_| change_level.call(index),
                        span { class: "kit-tls__level-name", "{name}" }
                    }
                }
            }
            p { class: "kit-tls__level-note", "{level_description}" }
            if empty {
                p { class: "kit-tls__empty", "No capabilities to configure." }
            } else {
                if !runs_list.is_empty() {
                    CapabilityGroup {
                        group_id: format!("{id}-runs"),
                        group_title: "Runs without confirmation".to_string(),
                        rows: runs_list,
                        on_toggle_pin: toggle_pin,
                    }
                }
                if !asks_list.is_empty() {
                    CapabilityGroup {
                        group_id: format!("{id}-asks"),
                        group_title: "Needs confirmation".to_string(),
                        rows: asks_list,
                        on_toggle_pin: toggle_pin,
                    }
                }
            }
            footer { class: "kit-tls__count", "{count_line}" }
            p { class: "kit-tls__live", role: "status", "aria-live": "polite", "{announcement}" }
        }
    }
}

#[component]
fn CapabilityGroup(
    group_id: String,
    group_title: String,
    rows: Vec<(Capability, bool)>,
    on_toggle_pin: EventHandler<String>,
) -> Element {
    let title_id = format!("{group_id}-title");
    let items: Vec<(usize, String, Capability, bool)> = rows
        .into_iter()
        .enumerate()
        .map(|(index, (capability, pinned))| (index, capability.id.clone(), capability, pinned))
        .collect();
    rsx! {
        section { class: "kit-tls__group", aria_labelledby: "{title_id}",
            h3 { id: "{title_id}", class: "kit-tls__group-title", "{group_title}" }
            ul { class: "kit-tls__list",
                for (index, capability_key, capability, pinned) in items {
                    CapabilityRow {
                        key: "{capability_key}",
                        index: index,
                        capability: capability,
                        pinned: pinned,
                        on_toggle_pin: on_toggle_pin,
                    }
                }
            }
        }
    }
}

#[component]
fn CapabilityRow(
    index: usize,
    capability: Capability,
    pinned: bool,
    on_toggle_pin: EventHandler<String>,
) -> Element {
    let name = capability.capability_name.clone();
    let note = capability.capability_note.clone();
    let mark = capability.capability_mark.clone();
    let tag = capability.risk_class.tag();
    let risk_key = capability.risk_class.as_str();
    let capability_id = capability.id.clone();
    let pin_label = format!("Always ask: {name}");
    let pinned_attr = if pinned { "true" } else { "false" };
    rsx! {
        li { class: "kit-tls__row", "data-pinned": pinned_attr, "data-risk": risk_key, style: "--kit-i: {index}",
            if let Some(glyph) = mark {
                span { class: "kit-tls__mark", aria_hidden: "true", "{glyph}" }
            }
            div { class: "kit-tls__body",
                span { class: "kit-tls__name", "{name}" }
                if let Some(text) = note {
                    span { class: "kit-tls__note", "{text}" }
                }
            }
            if let Some(label) = tag {
                span { class: "kit-tls__tag", "data-risk": risk_key, "{label}" }
            }
            button {
                r#type: "button",
                class: "kit-tls__pin",
                "aria-pressed": pinned_attr,
                aria_label: "{pin_label}",
                onclick: move |_| on_toggle_pin.call(capability_id.clone()),
                if pinned { "Pinned" } else { "Pin" }
            }
        }
    }
}
