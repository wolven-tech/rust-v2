//! Model selector: the model menu of a chat composer, with the options that travel with the model.
//!
//! Design choice: effort scales the first-token estimate only while extended thinking is on
//! (Low 0.6, Medium 1.0, High 1.6). A router option, with no ranks, shows "Varies" instead of bars.
//!
//! Browser checks: hovering a row fills its gauges; `menu_side: Top` opens above the trigger; the
//! row icons and key hints disappear at the narrowest width; focus returns to the trigger on Escape.

use dioxus::prelude::*;
use rv2_affect::switch::Switch;

use crate::internal::{KIT_CSS, class_list, next_id};

const CSS: Asset = asset!("/assets/model-selector.css");

fn focus_dom(id: &str) {
    document::eval(&format!("document.getElementById({id:?})?.focus();"));
}
/// Ranks run from 1 to this value.
pub const MAX_RANK: u8 = 5;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThinkingEffort {
    Low,
    #[default]
    Medium,
    High,
}

impl ThinkingEffort {
    pub const ALL: [ThinkingEffort; 3] = [Self::Low, Self::Medium, Self::High];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
        }
    }

    /// Multiplier on the first-token estimate.
    pub fn factor(self) -> f64 {
        match self {
            Self::Low => 0.6,
            Self::Medium => 1.0,
            Self::High => 1.6,
        }
    }

    /// The effort one step left (-1) or right (1), clamped at the ends.
    pub fn step(self, dir: isize) -> Self {
        let index = Self::ALL.iter().position(|e| *e == self).unwrap_or(1) as isize;
        Self::ALL[(index + dir).clamp(0, 2) as usize]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuAlign {
    #[default]
    Start,
    End,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuSide {
    #[default]
    Bottom,
    Top,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModelOption {
    pub option_id: String,
    pub option_name: String,
    pub option_note: Option<String>,
    pub option_icon: Option<Element>,
    /// 1 to 5, where 5 is fastest.
    pub speed_rank: Option<u8>,
    /// 1 to 5, where 5 is best.
    pub quality_rank: Option<u8>,
    /// 1 to 5, where 5 is most expensive.
    pub cost_rank: Option<u8>,
    /// Seconds to the first token, before thinking.
    pub first_token_time: Option<f64>,
    pub price_per_million: Option<f64>,
}

impl ModelOption {
    pub fn new(option_id: &str, option_name: &str) -> Self {
        Self {
            option_id: option_id.to_string(),
            option_name: option_name.to_string(),
            option_note: None,
            option_icon: None,
            speed_rank: None,
            quality_rank: None,
            cost_rank: None,
            first_token_time: None,
            price_per_million: None,
        }
    }

    /// A router option has no ranks; its gauges read "Varies".
    pub fn is_router(&self) -> bool {
        self.speed_rank.is_none() && self.quality_rank.is_none() && self.cost_rank.is_none()
    }
}

/// Filled gauge segments for a rank; a missing rank fills none.
pub fn bars_filled(rank: Option<u8>) -> u8 {
    rank.map_or(0, |rank| rank.clamp(1, MAX_RANK))
}

/// `1.2s`, scaled by the effort factor while thinking is on.
pub fn first_token_text(seconds: Option<f64>, effort: ThinkingEffort, thinking: bool) -> String {
    match seconds {
        None => "—".to_string(),
        Some(base) => {
            let factor = if thinking { effort.factor() } else { 1.0 };
            format!("{:.1}s", base * factor)
        }
    }
}

pub fn price_text(price: Option<f64>) -> String {
    match price {
        Some(price) => format!("${price:.2} / M tokens"),
        None => "Varies".to_string(),
    }
}

pub fn trigger_label(model_name: &str, thinking: bool) -> String {
    let state = if thinking { "on" } else { "off" };
    format!("Model: {model_name}, extended thinking {state}")
}

#[component]
fn RankBars(label: String, rank: Option<u8>) -> Element {
    let filled = bars_filled(rank) as usize;
    rsx! {
        span { class: "kit-ms__gauge", role: "img", "aria-label": "{label}",
            for index in 0..MAX_RANK as usize {
                span { key: "{index}", class: "kit-ms__bar", "data-on": (index < filled).then_some("true") }
            }
        }
    }
}

#[component]
pub fn ModelSelector(
    #[props(default)] model_options: Vec<ModelOption>,
    #[props(default)] selected_model_id: Option<String>,
    #[props(default)] initial_model_id: Option<String>,
    #[props(default)] on_model_change: EventHandler<String>,
    #[props(default)] thinking_on: Option<bool>,
    #[props(default)] initial_thinking: bool,
    #[props(default)] on_thinking_change: EventHandler<bool>,
    #[props(default)] web_search_on: Option<bool>,
    #[props(default)] initial_web_search: bool,
    #[props(default)] on_web_search_change: EventHandler<bool>,
    #[props(default)] thinking_effort: Option<ThinkingEffort>,
    #[props(default)] initial_effort: ThinkingEffort,
    #[props(default)] on_effort_change: EventHandler<ThinkingEffort>,
    #[props(default)] is_open: Option<bool>,
    #[props(default)] initial_open: bool,
    #[props(default)] on_open_change: EventHandler<bool>,
    #[props(default)] menu_align: MenuAlign,
    #[props(default)] menu_side: MenuSide,
    #[props(default, into)] extra_class: String,
) -> Element {
    let id = use_hook(|| format!("kit-ms-{}", next_id()));
    let mut model_state = use_signal(|| initial_model_id.clone());
    let mut thinking_state = use_signal(|| initial_thinking);
    let mut web_state = use_signal(|| initial_web_search);
    let mut effort_state = use_signal(|| initial_effort);
    let mut open_state = use_signal(|| initial_open);
    let mut highlighted = use_signal(|| None::<usize>);

    let first_id = model_options.first().map(|m| m.option_id.clone());
    let current_id = selected_model_id
        .clone()
        .or(model_state())
        .or(first_id)
        .unwrap_or_default();
    let thinking = thinking_on.unwrap_or(thinking_state());
    let web_search = web_search_on.unwrap_or(web_state());
    let effort = thinking_effort.unwrap_or(effort_state());
    let open = is_open.unwrap_or(open_state());
    let current_index = model_options.iter().position(|m| m.option_id == current_id);
    let shown_index = highlighted()
        .or(current_index)
        .filter(|i| *i < model_options.len());
    let shown = shown_index.map(|i| model_options[i].clone());
    let shown_seconds = shown.as_ref().and_then(|m| m.first_token_time);
    let shown_price = shown.as_ref().and_then(|m| m.price_per_million);
    let name = model_options
        .get(current_index.unwrap_or(0))
        .map(|m| m.option_name.clone())
        .unwrap_or_default();
    let trigger = trigger_label(&name, thinking);
    let align = if menu_align == MenuAlign::End {
        "end"
    } else {
        "start"
    };
    let side = if menu_side == MenuSide::Top {
        "top"
    } else {
        "bottom"
    };
    let footer_first = first_token_text(shown_seconds, effort, thinking);
    let footer_price = price_text(shown_price);
    let options_for_keys = model_options.clone();
    let trigger_id = format!("{id}-trigger");
    let mut set_open = move |next: bool| {
        open_state.set(next);
        on_open_change.call(next);
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class: class_list("kit-ms", &extra_class), "data-open": open.then_some("true"),
            button {
                id: "{id}-trigger",
                r#type: "button",
                class: "kit-ms__trigger",
                "aria-haspopup": "listbox",
                "aria-expanded": "{open}",
                "aria-controls": "{id}-menu",
                "aria-label": "{trigger}",
                onclick: move |_| {
                    highlighted.set(None);
                    set_open(!open);
                },
                onkeydown: move |event: KeyboardEvent| {
                    if matches!(event.key(), Key::ArrowDown) {
                        event.prevent_default();
                        highlighted.set(current_index.or(Some(0)));
                        set_open(true);
                    }
                },
                span { class: "kit-ms__name", "{name}" }
                if thinking {
                    span { class: "kit-ms__thinking", "aria-hidden": "true", "✦" }
                }
            }
            if open {
                div {
                    id: "{id}-menu",
                    class: "kit-ms__menu",
                    "data-align": "{align}",
                    "data-side": "{side}",
                    div {
                        id: "{id}-list",
                        class: "kit-ms__list",
                        role: "listbox",
                        "aria-label": "Models",
                        tabindex: "0",
                        onmounted: {
                            let list_id = format!("{id}-list");
                            move |_: MountedEvent| focus_dom(&list_id)
                        },
                        onkeydown: move |event: KeyboardEvent| {
                            let len = options_for_keys.len();
                            let key = event.key();
                            if key == Key::Enter || key == Key::Character(" ".into()) {
                                event.prevent_default();
                                if let Some(option) = highlighted().and_then(|i| options_for_keys.get(i)) {
                                    model_state.set(Some(option.option_id.clone()));
                                    on_model_change.call(option.option_id.clone());
                                    set_open(false);
                                    focus_dom(&trigger_id);
                                }
                                return;
                            }
                            match key {
                                Key::ArrowDown | Key::ArrowUp => {
                                    event.prevent_default();
                                    let dir = if event.key() == Key::ArrowDown { 1 } else { -1 };
                                    let from = highlighted().or(current_index);
                                    if len > 0 {
                                        highlighted.set(Some(match from {
                                            None if dir > 0 => 0,
                                            None => len - 1,
                                            Some(i) => (i as isize + dir).clamp(0, len as isize - 1) as usize,
                                        }));
                                    }
                                }
                                Key::Home => {
                                    event.prevent_default();
                                    highlighted.set(Some(0));
                                }
                                Key::End => {
                                    event.prevent_default();
                                    highlighted.set(len.checked_sub(1));
                                }
                                Key::Escape => {
                                    event.prevent_default();
                                    set_open(false);
                                    focus_dom(&trigger_id);
                                }
                                _ => {}
                            }
                        },
                        for (index, option) in model_options.iter().enumerate() {
                            {
                                let picked = option.option_id == current_id;
                                let is_router = option.is_router();
                                let speed_label = format!("Speed {} of {}", bars_filled(option.speed_rank), MAX_RANK);
                                let quality_label = format!("Quality {} of {}", bars_filled(option.quality_rank), MAX_RANK);
                                let cost_label = format!("Cost {} of {}", bars_filled(option.cost_rank), MAX_RANK);
                                let option_id = option.option_id.clone();
                                let option_name = option.option_name.clone();
                                let note = option.option_note.clone();
                                let icon = option.option_icon.clone();
                                let row_id = format!("{id}-row-{index}");
                                rsx! {
                                    div {
                                        key: "{option.option_id}",
                                        id: "{row_id}",
                                        class: "kit-ms__row",
                                        role: "option",
                                        "aria-selected": "{picked}",
                                        "data-highlighted": (highlighted() == Some(index)).then_some("true"),
                                        onmouseenter: move |_| highlighted.set(Some(index)),
                                        onclick: move |_| {
                                            model_state.set(Some(option_id.clone()));
                                            on_model_change.call(option_id.clone());
                                            set_open(false);
                                        },
                                        if let Some(icon) = icon {
                                            span { class: "kit-ms__icon", "aria-hidden": "true", {icon} }
                                        }
                                        span { class: "kit-ms__text",
                                            span { class: "kit-ms__title", "{option_name}" }
                                            if let Some(note) = note {
                                                span { class: "kit-ms__note", "{note}" }
                                            }
                                        }
                                        if is_router {
                                            span { class: "kit-ms__varies", "Varies" }
                                        } else {
                                            span { class: "kit-ms__gauges",
                                                RankBars { label: speed_label, rank: option.speed_rank }
                                                RankBars { label: quality_label, rank: option.quality_rank }
                                                RankBars { label: cost_label, rank: option.cost_rank }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "kit-ms__divider" }
                    div { class: "kit-ms__settings",
                        Switch {
                            label: rsx! { "Extended thinking" },
                            checked: thinking,
                            onchange: move |_| {
                                let next = !thinking;
                                thinking_state.set(next);
                                on_thinking_change.call(next);
                            },
                        }
                        if thinking {
                            div { class: "kit-ms__effort", role: "radiogroup", "aria-label": "Thinking effort",
                                onkeydown: move |event: KeyboardEvent| {
                                    let dir = match event.key() {
                                        Key::ArrowLeft => -1,
                                        Key::ArrowRight => 1,
                                        _ => 0,
                                    };
                                    if dir != 0 {
                                        event.prevent_default();
                                        let next = effort.step(dir);
                                        effort_state.set(next);
                                        on_effort_change.call(next);
                                    }
                                },
                                for level in ThinkingEffort::ALL {
                                    {
                                        let checked = effort == level;
                                        let label = level.label();
                                        rsx! {
                                            button {
                                                key: "{level:?}",
                                                r#type: "button",
                                                role: "radio",
                                                class: "kit-ms__level",
                                                "aria-checked": "{checked}",
                                                tabindex: if checked { "0" } else { "-1" },
                                                onclick: move |_| {
                                                    effort_state.set(level);
                                                    on_effort_change.call(level);
                                                },
                                                "{label}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Switch {
                            label: rsx! { "Web search" },
                            checked: web_search,
                            onchange: move |_| {
                                let next = !web_search;
                                web_state.set(next);
                                on_web_search_change.call(next);
                            },
                        }
                    }
                    div { class: "kit-ms__tray", "aria-live": "off",
                        span { class: "kit-ms__figure", "First token {footer_first}" }
                        span { class: "kit-ms__figure", "{footer_price}" }
                    }
                }
            }
        }
    }
}
