//! Inspected agent card scene: one focused agent card tilted back in a layout tool, with a faded card
//! above it. The selected row is framed, its tag and spacing values sit beside it, and a guide walks
//! down the rows, pausing on each. The faded card never takes the guide.
//!
//! Design choice: the rows have fixed heights and a fixed gap (`ROW_HEIGHTS_PX`, `ROW_GAP_PX`), so the
//! guide, frame, tags and spacing markers are placed from pure geometry (`row_top`, `row_centre`).
//! Design choice: the width label is the frame's layout width, read after each step, so it matches the
//! row it sits on. `offsetWidth` is used because the tilt transform changes the bounding box.
//! Design choice: the guide's play state is one function of hover, focus, pin, visibility and motion
//! preference (`is_playing`); the timer only advances the step while that holds.
//! Design choice: the reduced-motion view is the static card with every spacing marker shown and the
//! guide on the first row.
//! Design choice: the keyboard pauses the scene on focus; Escape resumes play until focus leaves.
//! Browser checks: the guide visits each row in order and pauses on each; hovering a row pins the guide
//! there and leaving resumes; the frame's width label equals the row's width; reduced motion shows all
//! markers with no movement; Right moves forward a row and Escape resumes; no sideways scroll at 390 or
//! 320 with the markers hidden.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id, offset_width, prefers_motion_script, sleep};

const CSS: Asset = asset!("/assets/inspected-agent-card-scene.css");

/// Height of each row (header, task line, footer), px.
pub const ROW_HEIGHTS_PX: [u32; 3] = [84, 56, 48];
/// Space between two rows, px; the gap marker sits in it.
pub const ROW_GAP_PX: u32 = 16;
/// Padding inside the card above the first row and below the last, px.
pub const CARD_PAD_PX: u32 = 12;
/// The value shown in the gap between the header and the task line, in design units. The rows sit
/// flush on the frame, so this reads 0 at rest.
pub const GAP_UNITS: u32 = 0;
/// The value shown on the frame's top edge and left inset, in design units.
pub const INSET_UNITS: u32 = 8;
/// Tag shown beside each row.
pub const ROW_TAGS: [&str; 3] = ["Header", "Task", "Footer"];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InspectSpeed {
    Slow,
    #[default]
    Normal,
}

/// Milliseconds the guide rests on each step.
pub fn tick_ms(speed: InspectSpeed) -> u32 {
    match speed {
        InspectSpeed::Slow => 2600,
        InspectSpeed::Normal => 1600,
    }
}

/// Top of a row's box, px from the top of the card.
pub fn row_top(row: usize) -> u32 {
    CARD_PAD_PX
        + ROW_HEIGHTS_PX[..row.min(ROW_HEIGHTS_PX.len())]
            .iter()
            .map(|h| h + ROW_GAP_PX)
            .sum::<u32>()
}

/// Height of a row's box, px.
pub fn row_height(row: usize) -> u32 {
    ROW_HEIGHTS_PX[row.min(ROW_HEIGHTS_PX.len() - 1)]
}

/// Centre of a row, px from the top of the card; the guide sits here.
pub fn row_centre(row: usize) -> u32 {
    row_top(row) + row_height(row) / 2
}

/// The step after `current`, wrapping at `len`.
pub fn next_step(current: usize, len: usize) -> usize {
    if len == 0 { 0 } else { (current + 1) % len }
}

/// The step `delta` away from `current`, clamped to the steps.
pub fn step_by(current: usize, len: usize, delta: isize) -> usize {
    if len == 0 {
        return 0;
    }
    (current as isize + delta).clamp(0, len as isize - 1) as usize
}

/// The guide plays only when nothing holds it: auto play on, no pointer or focus hold, no pin, the
/// scene on screen and motion allowed.
pub fn is_playing(
    auto_play: bool,
    pointer_held: bool,
    focus_held: bool,
    pinned: bool,
    visible: bool,
    motion_on: bool,
) -> bool {
    auto_play && !pointer_held && !focus_held && !pinned && visible && motion_on
}

/// The signals the guide's clock reads and writes; shared by every inspect scene.
#[derive(Clone, Copy)]
pub(crate) struct GuideClock {
    pub active: Signal<usize>,
    pub playing: Signal<bool>,
    pub visible: Signal<bool>,
    pub motion_on: Signal<bool>,
    pub steps_len: usize,
    pub speed: InspectSpeed,
    /// The step the guide rests on at mount and under reduced motion.
    pub rest_step: usize,
}

/// Starts the guide's timer and its on-screen observer. With reduced motion the timer never starts
/// and the guide stays on its rest step.
pub(crate) fn use_guide_clock(root_id: String, clock: GuideClock) {
    let GuideClock {
        mut active,
        playing,
        mut visible,
        mut motion_on,
        steps_len,
        speed,
        rest_step,
    } = clock;
    use_hook(move || {
        spawn(async move {
            let script = format!(
                "let el = null;\
                 for (let i = 0; i < 600 && !(el = document.getElementById({root_id:?})); i++) \
                     await new Promise((resolve) => requestAnimationFrame(resolve));\
                 if (!el) return;\
                 const io = new IntersectionObserver((entries) => dioxus.send(entries[0].isIntersecting));\
                 io.observe(el);\
                 await new Promise(() => {{}});"
            );
            let mut eval = document::eval(&script);
            while let Ok(on_screen) = eval.recv::<bool>().await {
                visible.set(on_screen);
            }
        });
    });
    use_hook(move || {
        spawn(async move {
            let allowed = document::eval(&format!("return {};", prefers_motion_script()))
                .await
                .ok()
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !allowed {
                motion_on.set(false);
                active.set(rest_step);
                return;
            }
            loop {
                sleep(tick_ms(speed)).await;
                if *playing.peek() {
                    let next = next_step(*active.peek(), steps_len);
                    active.set(next);
                }
            }
        });
    });
}

#[derive(Clone, PartialEq)]
pub struct CardFooter {
    pub cost_label: String,
    pub time_label: String,
}

#[derive(Clone, PartialEq)]
pub struct AgentCard {
    pub title: String,
    pub model_label: String,
    pub status_label: String,
    pub task_line: String,
    pub footer: CardFooter,
}

/// The faded card above the focused one: header, task line and footer, a different agent.
#[derive(Clone, PartialEq)]
pub struct GhostCard {
    pub title: String,
    pub model_label: String,
    pub status_label: String,
    pub task_line: String,
    pub footer: CardFooter,
}

#[derive(Clone, PartialEq)]
pub struct InspectStep {
    pub row_index: usize,
    pub note: String,
}

/// Capital letters of up to the first two words of a name, for the avatar.
pub fn avatar_letters(name: &str) -> String {
    name.split_whitespace()
        .take(2)
        .filter_map(|word| word.chars().next())
        .flat_map(char::to_uppercase)
        .collect()
}

#[component]
pub fn InspectedAgentCardScene(
    card: AgentCard,
    #[props(default)] ghost: Option<GhostCard>,
    #[props(default)] inspect_steps: Vec<InspectStep>,
    #[props(default = true)] auto_play: bool,
    #[props(default)] speed: InspectSpeed,
    #[props(default)] interactive: bool,
    #[props(default)] class: String,
) -> Element {
    let uid = use_hook(|| format!("kit-inspect-{}", next_id()));
    let steps: Vec<InspectStep> = if inspect_steps.is_empty() {
        (0..ROW_TAGS.len())
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
            rest_step: 0,
        },
    );

    let step_index = pinned()
        .unwrap_or_else(|| active())
        .min(steps_len.saturating_sub(1));
    let current_row = steps
        .get(step_index)
        .map(|s| s.row_index)
        .unwrap_or(0)
        .min(ROW_TAGS.len() - 1);
    let note = steps
        .get(step_index)
        .map(|s| s.note.clone())
        .unwrap_or_default();

    let measure_id = uid.clone();
    use_effect(use_reactive((&current_row,), move |(row,)| {
        let selector = format!("#{measure_id} .kit-inspect__row[data-row=\"{row}\"]");
        if let Some(width) = offset_width(&selector) {
            frame_width.set((width > 0).then_some(width));
        }
    }));

    let guide_y = row_top(current_row) as i32;
    let guide_h = row_height(current_row);
    let frame_y = row_top(current_row) as i32 - 4;
    let frame_h = row_height(current_row) + 8;
    let frame_style = format!("transform: translateY({frame_y}px); height: {frame_h}px;");
    let guide_style = format!("transform: translateY({guide_y}px); height: {guide_h}px;");
    let pointer_y = row_centre(current_row) as i32 - 5;
    let pointer_style = format!("transform: translateY({pointer_y}px);");
    let width_label = frame_width().map(|w| format!("{w}px")).unwrap_or_default();
    let class = class_list("kit-inspect", &class);
    let motion_attr = if motion_on() { "on" } else { "off" };
    let root_id = uid.clone();
    let row_pins: Vec<Option<usize>> = (0..ROW_TAGS.len())
        .map(|row| steps.iter().position(|s| s.row_index == row))
        .collect();
    let [pin0, pin1, pin2] = [row_pins[0], row_pins[1], row_pins[2]];
    let title = card.title.clone();
    let status_label = card.status_label.clone();
    let model_label = card.model_label.clone();
    let task_line = card.task_line.clone();
    let cost_label = card.footer.cost_label.clone();
    let time_label = card.footer.time_label.clone();
    let avatar = avatar_letters(&card.title);
    let scene_aria_hidden = (!interactive).then_some("true");
    let frame_key = format!("frame-{current_row}");
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
            div { class: "kit-inspect__stage",
                if let Some(ghost) = ghost {
                    div { class: "kit-inspect__ghost", "aria-hidden": "true",
                        span { class: "kit-inspect__ghost-head",
                            span { class: "kit-inspect__ghost-title", "{ghost.title}" }
                            span { class: "kit-inspect__ghost-status", "{ghost.status_label}" }
                        }
                        span { class: "kit-inspect__ghost-model", "{ghost.model_label}" }
                        span { class: "kit-inspect__ghost-task", "{ghost.task_line}" }
                        span { class: "kit-inspect__ghost-foot",
                            span { "{ghost.footer.cost_label}" }
                            span { "{ghost.footer.time_label}" }
                        }
                    }
                }
                div { class: "kit-inspect__card",
                    div { class: "kit-inspect__row", "data-row": "0",
                        onmouseenter: move |_| {
                            if let Some(position) = pin0 {
                                active.set(position);
                                pinned.set(Some(position));
                            }
                        },
                        onmouseleave: move |_| pinned.set(None),
                        span { class: "kit-inspect__avatar", "aria-hidden": "true", "{avatar}" }
                        span { class: "kit-inspect__who",
                            span { class: "kit-inspect__title", "{title}" }
                            span { class: "kit-inspect__model", "{model_label}" }
                        }
                        span { class: "kit-inspect__status",
                            span { class: "kit-inspect__spin", "aria-hidden": "true" }
                            "{status_label}"
                        }
                    }
                    div { class: "kit-inspect__row kit-inspect__row--task", "data-row": "1",
                        onmouseenter: move |_| {
                            if let Some(position) = pin1 {
                                active.set(position);
                                pinned.set(Some(position));
                            }
                        },
                        onmouseleave: move |_| pinned.set(None),
                        span { class: "kit-inspect__task", "{task_line}" }
                    }
                    div { class: "kit-inspect__row kit-inspect__row--foot", "data-row": "2",
                        onmouseenter: move |_| {
                            if let Some(position) = pin2 {
                                active.set(position);
                                pinned.set(Some(position));
                            }
                        },
                        onmouseleave: move |_| pinned.set(None),
                        span { class: "kit-inspect__cost", "{cost_label}" }
                        span { class: "kit-inspect__time", "{time_label}" }
                    }
                    for row in 0..ROW_TAGS.len() {
                        {
                            let is_current = row == current_row;
                            let top_y = row_top(row) as i32 - 4;
                            let row_mid = row_centre(row) as i32 - 6;
                            let gap_y = (row_top(row) + row_height(row) + ROW_GAP_PX / 2) as i32 - 6;
                            let top_style = format!("transform: translateY({}px);", top_y - 18);
                            let inset_style = format!("transform: translateY({row_mid}px);");
                            let gap_style = format!("transform: translateY({gap_y}px);");
                            let tag_style = format!("transform: translateY({}px);", row_mid);
                            let tag = ROW_TAGS[row];
                            let has_gap = row + 1 < ROW_TAGS.len();
                            rsx! {
                                span { class: "kit-inspect__marker", "data-row": "{row}",
                                    "data-active": is_current.then_some("true"),
                                    style: "{top_style}",
                                    "{INSET_UNITS}"
                                }
                                span { class: "kit-inspect__inset", "data-row": "{row}",
                                    "data-active": is_current.then_some("true"),
                                    style: "{inset_style}",
                                    "{INSET_UNITS}"
                                }
                                if has_gap {
                                    span { class: "kit-inspect__gap", "data-row": "{row}",
                                        "data-active": is_current.then_some("true"),
                                        style: "{gap_style}",
                                        "{GAP_UNITS}"
                                    }
                                }
                                span { class: "kit-inspect__tag", "data-row": "{row}",
                                    "data-active": is_current.then_some("true"),
                                    style: "{tag_style}",
                                    "{tag}"
                                }
                            }
                        }
                    }
                    span { class: "kit-inspect__rail" }
                    span { class: "kit-inspect__guide", style: "{guide_style}" }
                    span { class: "kit-inspect__frame", key: "{frame_key}", style: "{frame_style}",
                        span { class: "kit-inspect__handle", "data-corner": "tl" }
                        span { class: "kit-inspect__handle", "data-corner": "tr" }
                        span { class: "kit-inspect__handle", "data-corner": "bl" }
                        span { class: "kit-inspect__handle", "data-corner": "br" }
                        span { class: "kit-inspect__width", "{width_label}" }
                    }
                    span { class: "kit-inspect__pointer", style: "{pointer_style}" }
                }
            }
            p { class: "kit-inspect__note", {note} }
        }
    }
}
