//! Inspected composer scene: one composer column tilted back in a layout tool. The message row is in
//! front and at full strength; the toolbar and attach rows sit faded above it and repeat below it. The
//! selected row is framed with its tag, spacing values and width, and a guide walks the first three
//! rows, pausing on each. The faded rows never take the guide.
//!
//! Design choice: the play rules, guide clock, keyboard arrows and reduced-motion view are the card
//! scene's (`inspected_agent_card_scene`); only the geometry and the content differ.
//! Design choice: the rows have fixed heights and gaps (`COMPOSER_ROW_HEIGHTS_PX`), so the frame, guide
//! and markers are placed from pure geometry, as in the card scene. The repeated rows below the message
//! row take the same heights in CSS and are never framed.
//! Design choice: connected-app marks are supplied by the caller as `connected_apps`; the scene shows
//! the first three, on the attach row only, and draws none of its own.
//! Design choice: the width label is the frame's layout width, read after each step (`offsetWidth`,
//! since the tilt transform changes the bounding box).
//! Browser checks: the guide visits the three framed rows in order and pauses on each; the tag changes
//! as the guide passes; reduced motion holds the message row with all spacing; arrows move the guide a
//! row and Escape resumes; no sideways scroll at 390 or 320 with the markers hidden.

use dioxus::prelude::*;

use crate::illustrations::inspected_agent_card_scene::{
    GuideClock, InspectSpeed, InspectStep, is_playing, step_by, use_guide_clock,
};
use crate::internal::{KIT_CSS, class_list, next_id};

const CSS: Asset = asset!("/assets/inspected-composer-scene.css");

/// Height of each framed composer row (toolbar, attach, message), px.
pub const COMPOSER_ROW_HEIGHTS_PX: [u32; 3] = [48, 44, 64];
/// Space between two rows, px; the spacing marker sits in it.
pub const COMPOSER_ROW_GAP_PX: u32 = 12;
/// Padding inside the surface above the first row and below the last, px.
pub const COMPOSER_PAD_PX: u32 = 12;
/// Spacing value shown in the gap beside each row, in design units.
pub const COMPOSER_GAP_UNITS: u32 = 12;
/// Spacing value shown on the frame's top edge and left inset, in design units.
pub const COMPOSER_INSET_UNITS: u32 = 8;
/// Rows the guide visits.
pub const COMPOSER_ROWS: usize = 3;
/// Tag shown beside each framed row.
pub const COMPOSER_TAGS: [&str; COMPOSER_ROWS] = ["Toolbar", "Attach", "Message"];
/// App marks shown in the attach row.
pub const APP_MARKS_SHOWN: usize = 3;

/// Top of a composer row, px from the top of the surface.
pub fn composer_row_top(row: usize) -> u32 {
    COMPOSER_PAD_PX
        + COMPOSER_ROW_HEIGHTS_PX[..row.min(COMPOSER_ROWS)]
            .iter()
            .map(|h| h + COMPOSER_ROW_GAP_PX)
            .sum::<u32>()
}

/// Height of a composer row, px.
pub fn composer_row_height(row: usize) -> u32 {
    COMPOSER_ROW_HEIGHTS_PX[row.min(COMPOSER_ROWS - 1)]
}

/// Centre of a composer row, px from the top of the surface; the guide sits here.
pub fn composer_row_centre(row: usize) -> u32 {
    composer_row_top(row) + composer_row_height(row) / 2
}

#[derive(Clone, PartialEq)]
pub struct Composer {
    pub message: String,
    pub mode_label: String,
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
    let rest_step = steps_len.saturating_sub(1);
    let mut active = use_signal(|| rest_step);
    let mut pointer_scene = use_signal(|| false);
    let mut pinned = use_signal(|| None::<usize>);
    let mut focused = use_signal(|| false);
    let mut resumed = use_signal(|| false);
    let visible = use_signal(|| true);
    let motion_on = use_signal(|| true);
    let mut playing = use_signal(|| false);
    let mut frame_width = use_signal(|| None::<u32>);

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
            rest_step,
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

    let measure_id = uid.clone();
    use_effect(use_reactive((&current_row,), move |(row,)| {
        let script = format!(
            "const el = document.querySelector('#{measure_id} .kit-composer__row[data-row=\"{row}\"]');
             dioxus.send(el ? el.offsetWidth : 0);"
        );
        let mut channel = document::eval(&script);
        spawn(async move {
            if let Ok(width) = channel.recv::<u32>().await {
                frame_width.set((width > 0).then_some(width));
            }
        });
    }));

    let guide_y = composer_row_top(current_row) as i32;
    let guide_h = composer_row_height(current_row);
    let frame_y = composer_row_top(current_row) as i32 - 4;
    let frame_h = composer_row_height(current_row) + 8;
    let guide_style = format!("transform: translateY({guide_y}px); height: {guide_h}px;");
    let frame_style = format!("transform: translateY({frame_y}px); height: {frame_h}px;");
    let tag_y = composer_row_centre(current_row) as i32 - 9;
    let tag_style = format!("transform: translateY({tag_y}px);");
    let width_label = frame_width().map(|w| format!("{w}px")).unwrap_or_default();
    let row_label = COMPOSER_TAGS[current_row];
    let row_pins: Vec<Option<usize>> = (0..COMPOSER_ROWS)
        .map(|row| steps.iter().position(|s| s.row_index == row))
        .collect();
    let [pin0, pin1, pin2] = [row_pins[0], row_pins[1], row_pins[2]];
    let class = class_list("kit-composer", &class);
    let motion_attr = if motion_on() { "on" } else { "off" };
    let root_id = uid.clone();
    let mode_label = composer.mode_label.clone();
    let message = composer.message.clone();
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
                div { class: "kit-composer__column",
                    div { class: "kit-composer__surface",
                        div { class: "kit-composer__row kit-composer__row--faded", "data-row": "0",
                            onmouseenter: move |_| {
                                if let Some(position) = pin0 {
                                    active.set(position);
                                    pinned.set(Some(position));
                                }
                            },
                            onmouseleave: move |_| pinned.set(None),
                            span { class: "kit-composer__round", "aria-hidden": "true", "+" }
                            span { class: "kit-composer__mode",
                                span { class: "kit-composer__mode-label", "{mode_label}" }
                                span { class: "kit-composer__chevron", "aria-hidden": "true" }
                            }
                            span { class: "kit-composer__spacer" }
                            span { class: "kit-composer__round kit-composer__round--quiet", "aria-hidden": "true", "!" }
                            span { class: "kit-composer__round kit-composer__round--send", "aria-hidden": "true", "↑" }
                        }
                        div { class: "kit-composer__row kit-composer__row--attach kit-composer__row--faded", "data-row": "1",
                            onmouseenter: move |_| {
                                if let Some(position) = pin1 {
                                    active.set(position);
                                    pinned.set(Some(position));
                                }
                            },
                            onmouseleave: move |_| pinned.set(None),
                            span { class: "kit-composer__chip",
                                span { class: "kit-composer__folder", "aria-hidden": "true" }
                                span { "Work in a folder" }
                            }
                            span { class: "kit-composer__chip kit-composer__chip--apps",
                                span { class: "kit-composer__apps", "aria-hidden": "true",
                                    for app in connected_apps.iter().take(APP_MARKS_SHOWN) {
                                        span { class: "kit-composer__app", {app.clone()} }
                                    }
                                }
                                span { "Connect apps" }
                            }
                        }
                        div { class: "kit-composer__row kit-composer__row--message", "data-row": "2",
                            onmouseenter: move |_| {
                                if let Some(position) = pin2 {
                                    active.set(position);
                                    pinned.set(Some(position));
                                }
                            },
                            onmouseleave: move |_| pinned.set(None),
                            span { class: "kit-composer__message", "{message}" }
                            span { class: "kit-composer__caret", "aria-hidden": "true" }
                        }
                        div { class: "kit-composer__row kit-composer__row--faded", "aria-hidden": "true",
                            span { class: "kit-composer__round", "+" }
                            span { class: "kit-composer__mode",
                                span { class: "kit-composer__mode-label", "{mode_label}" }
                                span { class: "kit-composer__chevron" }
                            }
                            span { class: "kit-composer__spacer" }
                            span { class: "kit-composer__round kit-composer__round--quiet", "!" }
                            span { class: "kit-composer__round kit-composer__round--send", "↑" }
                        }
                        div { class: "kit-composer__row kit-composer__row--attach kit-composer__row--faded", "aria-hidden": "true",
                            span { class: "kit-composer__chip",
                                span { class: "kit-composer__folder" }
                                span { "Work in a folder" }
                            }
                            span { class: "kit-composer__chip", "Connect apps" }
                        }
                        for row in 0..COMPOSER_ROWS {
                            {
                                let is_current = row == current_row;
                                let top_y = composer_row_top(row) as i32 - 4;
                                let row_mid = composer_row_centre(row) as i32 - 6;
                                let gap_y = (composer_row_top(row) + composer_row_height(row) + COMPOSER_ROW_GAP_PX / 2) as i32 - 6;
                                let top_style = format!("transform: translateY({}px);", top_y - 18);
                                let inset_style = format!("transform: translateY({row_mid}px);");
                                let gap_style = format!("transform: translateY({gap_y}px);");
                                rsx! {
                                    span { class: "kit-composer__marker", "data-row": "{row}",
                                        "data-active": is_current.then_some("true"),
                                        style: "{top_style}",
                                        "{COMPOSER_INSET_UNITS}"
                                    }
                                    span { class: "kit-composer__inset", "data-row": "{row}",
                                        "data-active": is_current.then_some("true"),
                                        style: "{inset_style}",
                                        "{COMPOSER_INSET_UNITS}"
                                    }
                                    span { class: "kit-composer__gap", "data-row": "{row}",
                                        "data-active": is_current.then_some("true"),
                                        style: "{gap_style}",
                                        "{COMPOSER_GAP_UNITS}"
                                    }
                                }
                            }
                        }
                        span { class: "kit-composer__rail" }
                        span { class: "kit-composer__guide", style: "{guide_style}" }
                        span { class: "kit-composer__frame", key: "frame-{current_row}", style: "{frame_style}",
                            span { class: "kit-composer__width", "{width_label}" }
                        }
                        span { class: "kit-composer__tag", style: "{tag_style}", "{row_label}" }
                    }
                }
            }
            p { class: "kit-composer__note", {note} }
        }
    }
}
