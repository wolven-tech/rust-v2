//! Inspected composer scene: a chat composer shown as if a designer were inspecting it in a layout
//! tool. It is tilted back, its three rows are framed, spacing values sit beside them, and a guide
//! walks down the rows, pausing on each.
//!
//! Design choice: the play rules, guide clock, keyboard arrows and reduced-motion view are the card
//! scene's (`inspected_agent_card_scene`); only the geometry and the content differ.
//! Design choice: the composer has no header, so its first row starts at the top of the scene and
//! the rows use their own pitch (`COMPOSER_ROW_PX`).
//! Design choice: connected-app icons are supplied by the caller as `connected_apps`; the scene draws
//! none of its own.
//! Browser checks: the guide visits the three rows in order and pauses on each; hovering a row pins
//! the guide there; reduced motion shows the static composer with all spacing; arrows move the guide a
//! row and Escape resumes; no sideways scroll at 390 or 320 with the markers hidden.

use dioxus::prelude::*;

use crate::illustrations::inspected_agent_card_scene::{
    GuideClock, InspectSpeed, InspectStep, is_playing, step_by, use_guide_clock,
};
use crate::internal::{KIT_CSS, class_list, next_id};

const CSS: Asset = asset!("/assets/inspected-composer-scene.css");

/// Height of one composer row, px.
pub const COMPOSER_ROW_PX: u32 = 56;
/// Spacing value shown beside each row, in design units.
pub const COMPOSER_GAP_UNITS: u32 = 8;
/// Rows in the composer: mode and apps, text, attach and send.
pub const COMPOSER_ROWS: usize = 3;

/// Top of a composer row, px from the top of the scene.
pub fn composer_row_top(row: usize) -> u32 {
    row as u32 * COMPOSER_ROW_PX
}

/// Centre of a composer row, px from the top of the scene; the guide sits here.
pub fn composer_row_centre(row: usize) -> u32 {
    composer_row_top(row) + COMPOSER_ROW_PX / 2
}

#[derive(Clone, PartialEq)]
pub struct Composer {
    pub placeholder_text: String,
    pub mode_label: String,
    pub tools: Vec<String>,
    pub send_label: String,
}

#[component]
pub fn InspectedComposerScene(
    composer: Composer,
    #[props(default)] connected_apps: Vec<Element>,
    #[props(default)] inspect_steps: Vec<InspectStep>,
    #[props(default = true)] auto_play: bool,
    #[props(default)] speed: InspectSpeed,
    #[props(default)] interactive: bool,
    #[props(default)] class: String,
) -> Element {
    let uid = use_hook(|| format!("kit-composer-{}", next_id()));
    let steps: Vec<InspectStep> = if inspect_steps.is_empty() {
        (0..COMPOSER_ROWS)
            .map(|row_index| InspectStep {
                row_index,
                note: String::new(),
            })
            .collect()
    } else {
        inspect_steps.clone()
    };
    let steps_len = steps.len();
    let mut active = use_signal(|| 0usize);
    let mut pointer_scene = use_signal(|| false);
    let mut pinned = use_signal(|| None::<usize>);
    let mut focused = use_signal(|| false);
    let mut resumed = use_signal(|| false);
    let visible = use_signal(|| true);
    let motion_on = use_signal(|| true);
    let mut playing = use_signal(|| false);

    let focus_held = focused() && !resumed();
    let want = is_playing(
        auto_play,
        pointer_scene(),
        focus_held,
        pinned().is_some(),
        visible(),
        motion_on(),
    );
    if *playing.peek() != want {
        playing.set(want);
    }
    use_guide_clock(
        uid.clone(),
        GuideClock {
            active,
            playing,
            visible,
            motion_on,
            steps_len,
            speed,
        },
    );

    let step_index = pinned()
        .unwrap_or_else(|| active())
        .min(steps_len.saturating_sub(1));
    let current_row = steps
        .get(step_index)
        .map(|s| s.row_index)
        .unwrap_or(0)
        .min(COMPOSER_ROWS - 1);
    let note = steps
        .get(step_index)
        .map(|s| s.note.clone())
        .unwrap_or_default();
    let guide_y = composer_row_centre(current_row) as i32 - 1;
    let guide_style = format!("transform: translateY({guide_y}px);");
    let frame_style = format!("transform: translateY({}px);", guide_y - 28);
    let tag_style = guide_style.clone();
    let pointer_style = guide_style.clone();
    let row_label = ["Mode and apps", "Message", "Attach and send"][current_row];
    let row_pins: Vec<Option<usize>> = (0..COMPOSER_ROWS)
        .map(|row| steps.iter().position(|s| s.row_index == row))
        .collect();
    let [pin0, pin1, pin2] = [row_pins[0], row_pins[1], row_pins[2]];
    let class = class_list("kit-composer", &class);
    let motion_attr = if motion_on() { "on" } else { "off" };
    let root_id = uid.clone();
    let mode_label = composer.mode_label.clone();
    let placeholder_text = composer.placeholder_text.clone();
    let send_label = composer.send_label.clone();
    let tools = composer.tools.clone();
    let scene_aria_hidden = (!interactive).then_some("true");
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class, id: "{root_id}", "aria-hidden": scene_aria_hidden, "data-motion": motion_attr,
            "data-playing": playing().then_some("true"),
            tabindex: interactive.then_some("0"),
            onmouseenter: move |_| pointer_scene.set(true),
            onmouseleave: move |_| {
                pointer_scene.set(false);
                pinned.set(None);
            },
            onfocus: move |_| focused.set(true),
            onblur: move |_| {
                focused.set(false);
                resumed.set(false);
            },
            onkeydown: move |event: KeyboardEvent| {
                if !interactive {
                    return;
                }
                let delta = match event.key() {
                    Key::ArrowLeft | Key::ArrowUp => Some(-1),
                    Key::ArrowRight | Key::ArrowDown => Some(1),
                    _ => None,
                };
                if let Some(delta) = delta {
                    event.prevent_default();
                    active.set(step_by(step_index, steps_len, delta));
                } else if event.key() == Key::Escape {
                    event.prevent_default();
                    resumed.set(true);
                }
            },
            div { class: "kit-composer__stage",
                div { class: "kit-composer__ghost", "data-pos": "above" }
                div { class: "kit-composer__ghost", "data-pos": "below" }
                div { class: "kit-composer__surface",
                    div { class: "kit-composer__row", "data-row": "0",
                        onmouseenter: move |_| {
                            if let Some(position) = pin0 {
                                active.set(position);
                                pinned.set(Some(position));
                            }
                        },
                        onmouseleave: move |_| pinned.set(None),
                        span { class: "kit-composer__mode", {mode_label} }
                        span { class: "kit-composer__apps",
                            for app in connected_apps.iter() {
                                span { class: "kit-composer__app", {app.clone()} }
                            }
                        }
                    }
                    div { class: "kit-composer__row kit-composer__row--text", "data-row": "1",
                        onmouseenter: move |_| {
                            if let Some(position) = pin1 {
                                active.set(position);
                                pinned.set(Some(position));
                            }
                        },
                        onmouseleave: move |_| pinned.set(None),
                        span { class: "kit-composer__placeholder", {placeholder_text} }
                    }
                    div { class: "kit-composer__row", "data-row": "2",
                        onmouseenter: move |_| {
                            if let Some(position) = pin2 {
                                active.set(position);
                                pinned.set(Some(position));
                            }
                        },
                        onmouseleave: move |_| pinned.set(None),
                        span { class: "kit-composer__attach", "aria-hidden": "true", "+" }
                        span { class: "kit-composer__tools",
                            for tool in tools.iter() {
                                span { class: "kit-composer__tool", {tool.clone()} }
                            }
                        }
                        span { class: "kit-composer__send", {send_label} }
                    }
                    for row in 0..COMPOSER_ROWS {
                        {
                            let top = composer_row_top(row);
                            rsx! {
                                span { class: "kit-composer__marker", "data-row": "{row}",
                                    "data-active": (row == current_row).then_some("true"),
                                    style: "top: {top}px;",
                                    "{COMPOSER_GAP_UNITS}"
                                }
                            }
                        }
                    }
                    span { class: "kit-composer__frame", style: "{frame_style}" }
                    span { class: "kit-composer__guide", style: "{guide_style}" }
                    span { class: "kit-composer__pointer", style: "{pointer_style}" }
                    span { class: "kit-composer__tag", style: "{tag_style}", {row_label} }
                }
            }
            p { class: "kit-composer__note", {note} }
        }
    }
}
