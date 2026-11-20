//! Thinking loader: a small "the model is working" pill. A spinner of eight rays
//! and a label with a light sweeping over it. The label walks through phases; when
//! done, the spinner becomes a check and the label settles on "Thought for 6s".
//!
//! Port losses: the React original reads the clock and swaps the label during
//! render, so the new label appears in the same frame it animates. Here both run
//! in an effect after the render. The done label therefore shows `Thought for 0s`
//! for one frame before the measured time, and the old label rolls out one frame
//! after the new text appears. `loop` is renamed `looping` (a Rust keyword).

use dioxus::prelude::*;

use crate::internal::{class_list, next_id, sleep};

const CSS: Asset = asset!("/assets/thinking-loader.css");
const DEFAULT_PHASES: [&str; 4] = [
    "Thinking",
    "Searching the web",
    "Reading sources",
    "Writing response",
];

/// `75` → `"1m 15s"`, `125` → `"2m 5s"`, `3725` → `"1h 02m"`, `90000` → `"1d 1h"`.
pub fn took(seconds: u64) -> String {
    if seconds < 60 {
        return format!("{seconds}s");
    }
    let minutes = seconds / 60;
    if minutes < 60 {
        return format!("{minutes}m {}s", seconds % 60);
    }
    let hours = minutes / 60;
    if hours < 24 {
        format!("{hours}h {:02}m", minutes % 60)
    } else {
        format!("{}d {}h", hours / 24, hours % 24)
    }
}

/// The default done label: `Thought for 6s`.
pub fn thought_for(seconds: u64) -> String {
    format!("Thought for {}", took(seconds))
}

/// Whole seconds from a millisecond span, never below one.
pub fn elapsed_seconds(elapsed_ms: f64) -> u64 {
    ((elapsed_ms / 1000.0).round() as u64).max(1)
}

/// Milliseconds between phase steps. Zero means the default; anything else is
/// raised to 200 so a tiny value does not spin the timer every frame.
pub fn tick_ms(interval: u32) -> u32 {
    if interval == 0 {
        2200
    } else {
        interval.max(200)
    }
}

/// An empty list still shows the first default phase.
pub fn normalise_phases(given: Vec<String>) -> Vec<String> {
    if given.is_empty() {
        vec![DEFAULT_PHASES[0].to_string()]
    } else {
        given
    }
}

/// The phase on screen: the controlled phase if given, else the timer's step,
/// clamped to the last phase.
pub fn phase_index(controlled: Option<u32>, step: usize, count: usize) -> usize {
    let wanted = controlled.map_or(step, |phase| phase as usize);
    wanted.min(count.saturating_sub(1))
}

#[derive(Clone)]
pub enum DoneLabel {
    /// Shown as is.
    Text(String),
    /// Built from the seconds since the loader started.
    Elapsed(fn(u64) -> String),
}

/// Function pointers are not compared: any two `Elapsed` labels count as equal.
impl PartialEq for DoneLabel {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Text(a), Self::Text(b)) => a == b,
            (Self::Elapsed(_), Self::Elapsed(_)) => true,
            _ => false,
        }
    }
}

impl Default for DoneLabel {
    fn default() -> Self {
        Self::Elapsed(thought_for)
    }
}

impl DoneLabel {
    fn render(&self, seconds: u64) -> String {
        match self {
            Self::Text(text) => text.clone(),
            Self::Elapsed(build) => build(seconds),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThinkingLoaderSize {
    Sm,
    #[default]
    Md,
}

impl ThinkingLoaderSize {
    fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThinkingLoaderVariant {
    /// Bordered capsule.
    #[default]
    Pill,
    /// Just the spinner and the label, e.g. inside a chat bubble.
    Plain,
}

impl ThinkingLoaderVariant {
    fn as_str(self) -> &'static str {
        match self {
            Self::Pill => "pill",
            Self::Plain => "plain",
        }
    }
}

fn default_phases() -> Vec<String> {
    DEFAULT_PHASES
        .iter()
        .map(|phase| phase.to_string())
        .collect()
}

#[component]
pub fn ThinkingLoader(
    /// Labels to walk through, one after another.
    #[props(default = default_phases())]
    phases: Vec<String>,
    /// Time per phase, in ms.
    #[props(default = 2200)]
    interval: u32,
    /// Show a specific phase instead of advancing on a timer, e.g. from stream events.
    #[props(default)]
    phase: Option<u32>,
    /// Start over after the last phase instead of holding it.
    #[props(default)]
    looping: bool,
    /// Finished: the spinner turns into a check and the label into `done_label`.
    #[props(default)]
    done: bool,
    #[props(default)] done_label: DoneLabel,
    #[props(default)] size: ThinkingLoaderSize,
    #[props(default)] variant: ThinkingLoaderVariant,
    #[props(default)] class: String,
) -> Element {
    let phases = normalise_phases(phases);
    let phase_count = phases.len();
    let interval = tick_ms(interval);
    let id = use_hook(|| format!("ui-tl-{}", next_id()));
    let box_id = format!("{id}-box");
    let cur_id = format!("{id}-cur");

    let mut step = use_signal(|| 0usize);
    let mut ticket = use_signal(|| 0u32);
    let mut started = use_signal(|| None::<f64>);
    let mut seconds = use_signal(|| None::<u64>);

    // Phase timer. A new run bumps the ticket, so a stale timer stops on its next tick.
    use_effect(use_reactive(
        (&done, &phase, &phase_count, &interval, &looping),
        move |(done, phase, count, interval, looping)| {
            let mine = *ticket.peek() + 1;
            ticket.set(mine);
            if done || phase.is_some() || count < 2 {
                return;
            }
            spawn(async move {
                loop {
                    sleep(interval).await;
                    if *ticket.peek() != mine {
                        break;
                    }
                    let current = *step.peek();
                    step.set(if current + 1 < count {
                        current + 1
                    } else if looping {
                        0
                    } else {
                        current
                    });
                }
            });
        },
    ));

    // Clock: the start is taken when the loader mounts or work starts again;
    // the elapsed time is taken when done turns on.
    use_effect(use_reactive((&done,), move |(done,)| {
        spawn(async move {
            let now = now_ms().await;
            if done {
                if seconds.peek().is_none() {
                    let from = started.peek().unwrap_or(now);
                    seconds.set(Some(elapsed_seconds(now - from)));
                }
            } else {
                seconds.set(None);
                started.set(Some(now));
                step.set(0);
            }
        });
    }));

    let index = phase_index(phase, step(), phase_count);
    let label = if done {
        done_label.render(seconds().unwrap_or(0))
    } else {
        phases[index].clone()
    };

    let mut shown = use_signal(|| (0u32, label.clone()));
    let mut leaving = use_signal(|| None::<(u32, String)>);
    use_effect(use_reactive((&label,), move |(label,)| {
        if shown.peek().1 != label {
            let previous = shown.peek().clone();
            leaving.set(Some(previous.clone()));
            shown.set((previous.0 + 1, label));
        }
    }));

    // The text box takes the width of the incoming label, capped by the room
    // beside the spinner. A ResizeObserver refits when the container changes.
    let fit_ids = [id.clone(), box_id.clone(), cur_id.clone()];
    use_effect(use_reactive((&shown().0,), move |(_key,)| {
        document::eval(&fit_script(&fit_ids));
    }));
    let drop_id = id.clone();
    use_drop(move || {
        document::eval(&format!(
            "document.getElementById({drop_id:?})?.__fit?.disconnect();"
        ));
    });

    let (shown_key, shown_text) = shown();
    let mut current_class = String::from("ui-tl__label");
    if shown_key > 0 {
        current_class.push_str(" ui-tl__label--in");
    }
    if !done {
        current_class.push_str(" ui-tl__shine");
    }
    let class = class_list("ui-tl", &class);

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        span {
            id: "{id}",
            class,
            "data-size": size.as_str(),
            "data-variant": variant.as_str(),
            "data-done": done.then_some("true"),
            role: "status",
            aria_live: "polite",
            span { class: "ui-tl__icon", aria_hidden: "true",
                svg { class: "ui-tl__spinner", view_box: "0 0 16 16",
                    for i in 0..8i32 {
                        line {
                            key: "{i}",
                            x1: "8",
                            y1: "1.75",
                            x2: "8",
                            y2: "4.75",
                            transform: format!("rotate({} 8 8)", i * 45),
                            style: format!("animation-delay: {}ms", (i - 8) * 100),
                        }
                    }
                }
                svg { class: "ui-tl__check", view_box: "0 0 16 16",
                    path { d: "M3.5 8.4 6.6 11.3 12.5 4.9", path_length: "1" }
                }
            }
            span { id: "{box_id}", class: "ui-tl__text",
                if let Some((out_key, out_text)) = leaving() {
                    span {
                        key: "{out_key}",
                        class: "ui-tl__label ui-tl__label--out",
                        aria_hidden: "true",
                        dir: "auto",
                        onanimationend: move |_| leaving.set(None),
                        "{out_text}"
                    }
                }
                span {
                    id: "{cur_id}",
                    key: "{shown_key}",
                    class: current_class,
                    "data-text": "{shown_text}",
                    title: "{shown_text}",
                    dir: "auto",
                    "{shown_text}"
                }
            }
        }
    }
}

async fn now_ms() -> f64 {
    dioxus::document::eval("return Date.now();")
        .await
        .ok()
        .and_then(|value| value.as_f64())
        .unwrap_or(0.0)
}

fn fit_script(ids: &[String; 3]) -> String {
    let [root, bx, cur] = ids;
    format!(
        "const ids = [{root:?}, {bx:?}, {cur:?}];
         const get = () => ids.map((id) => document.getElementById(id));
         const fit = () => {{
           const [r, b, c] = get();
           if (!r || !b || !c) return;
           const full = c.scrollWidth + 1;
           const parent = r.parentElement;
           const was = {{ width: b.style.width, transition: b.style.transition }};
           b.style.transition = 'none';
           b.style.width = full + 'px';
           const pad = parent ? getComputedStyle(parent) : null;
           const inner = parent && pad ? parent.clientWidth - parseFloat(pad.paddingLeft) - parseFloat(pad.paddingRight) : 0;
           const room = parent ? Math.floor(inner - (r.offsetWidth - b.offsetWidth)) : full;
           b.style.width = was.width;
           b.getBoundingClientRect();
           b.style.transition = was.transition;
           const cut = room > 0 && full > room;
           r.toggleAttribute('data-cut', cut);
           b.style.width = (cut ? room : full) + 'px';
         }};
         const [root, box] = get();
         if (!root || !box) return;
         const first = !root.hasAttribute('data-fitted');
         if (first) box.style.transition = 'none';
         fit();
         if (first) {{ box.getBoundingClientRect(); box.style.transition = ''; root.setAttribute('data-fitted', ''); }}
         if (!root.__fit && root.parentElement && typeof ResizeObserver !== 'undefined') {{
           root.__fit = new ResizeObserver(fit);
           root.__fit.observe(root.parentElement);
         }}"
    )
}
