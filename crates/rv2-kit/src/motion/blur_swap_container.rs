//! Blur swap container: content changes in place with a blur crossfade while the
//! frame glides to the incoming height.
//!
//! Design choice: the outgoing layer is the previous render's `Element`, kept in a
//! plain `RefCell` (not a signal) so the swap is decided during render without a
//! second render pass. It is removed by a signal once its animation has finished.
//! Design choice: only height glides; the frame is `width: 100%` of its parent, so
//! width follows the container and never animates.
//! Design choice: the outgoing layer is `inert`, so Tab cannot reach it mid-swap.
//! Design choice: `content_key` is compared as a string; callers format their key.
//!
//! Browser checks: the outgoing content blurs out while the incoming one sharpens in;
//! the frame height glides to the incoming height; a key change during a swap leaves
//! one layer at the end; focus inside the old content lands in the new one; phone
//! width does not scroll sideways.

use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id, prefers_motion_script};

const CSS: Asset = asset!("/assets/blur-swap-container.css");

/// Length of the blur crossfade and height glide.
pub const SWAP_DURATION_MS: u32 = 480;

/// Selector for elements that can take focus; the first match in the incoming
/// content receives focus after a swap that started with focus inside.
pub const FOCUSABLE_SELECTOR: &str = "a[href],button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex=\"-1\"])";

/// True when the key differs, which starts a swap.
pub fn swap_needed(shown_key: &str, next_key: &str) -> bool {
    shown_key != next_key
}

/// The step after `current`, wrapping to the first step after the last.
pub fn next_step(current: usize, len: usize) -> usize {
    if len == 0 { 0 } else { (current + 1) % len }
}

/// Motion runs unless the viewer prefers reduced motion. `reduced_override`
/// wins over the preference when set (testing only).
pub fn motion_allowed(reduced_override: Option<bool>, prefers_reduced: bool) -> bool {
    !reduced_override.unwrap_or(prefers_reduced)
}

/// Keeps the content from the render before the current one, and the swap count.
struct Memory {
    key: String,
    content: Option<Element>,
    outgoing: Option<Element>,
    seq: u32,
}

#[component]
pub fn BlurSwapContainer(
    children: Element,
    #[props(into)] content_key: String,
    #[props(default)] on_swap_complete: Option<EventHandler<()>>,
    #[props(default)] reduced_motion_override: Option<bool>,
    #[props(default)] class: String,
) -> Element {
    let id = use_hook(|| format!("kit-blur-swap-{}", next_id()));
    let memory = use_hook(|| {
        Rc::new(RefCell::new(Memory {
            key: content_key.clone(),
            content: None,
            outgoing: None,
            seq: 0,
        }))
    });
    let mut done_seq = use_signal(|| 0u32);

    let (seq, outgoing) = {
        let mut m = memory.borrow_mut();
        if swap_needed(&m.key, &content_key) {
            m.outgoing = m.content.take();
            m.key = content_key.clone();
            m.seq += 1;
        }
        m.content = Some(children.clone());
        (m.seq, m.outgoing.clone())
    };
    let show_outgoing = outgoing.is_some() && done_seq() < seq;

    let frame_id = id.clone();
    use_effect(move || {
        let script = format!(
            "const frame = document.getElementById({frame_id:?});
             if (!frame || frame.dataset.tracked) return true;
             frame.dataset.tracked = '1';
             frame.addEventListener('focusin', () => {{ frame.dataset.focusWithin = 'true'; }});
             frame.addEventListener('focusout', (e) => {{
               if (!frame.contains(e.relatedTarget)) frame.dataset.focusWithin = 'false';
             }});
             return true;"
        );
        spawn(async move {
            let _ = document::eval(&script).await;
        });
    });

    let swap_id = id.clone();
    let override_js = match reduced_motion_override {
        Some(true) => "true",
        Some(false) => "false",
        None => "null",
    };
    use_effect(use_reactive((&seq,), move |(seq,)| {
        if seq == 0 {
            return;
        }
        let script = format!(
            "const frame = document.getElementById({swap_id:?});
             const out = document.getElementById({swap_id:?} + '-out');
             const cur = document.getElementById({swap_id:?} + '-in');
             if (!frame || !cur) return true;
             const hadFocus = frame.dataset.focusWithin === 'true';
             const override = {override_js};
             const allowed = override === null ? ({motion}) : !override;
             if (hadFocus) {{
               const target = cur.querySelector({FOCUSABLE_SELECTOR:?});
               if (target) target.focus({{ preventScroll: true }});
             }}
             if (!allowed) {{
               frame.dataset.reduced = 'true';
               return true;
             }}
             delete frame.dataset.reduced;
             const from = out ? out.getBoundingClientRect().height : frame.getBoundingClientRect().height;
             const to = cur.getBoundingClientRect().height;
             frame.dataset.swapping = 'true';
             const glide = frame.animate(
               [{{ height: from + 'px' }}, {{ height: to + 'px' }}],
               {{ duration: {SWAP_DURATION_MS}, easing: 'cubic-bezier(0.2, 0.8, 0.2, 1)' }}
             );
             await glide.finished.catch(() => {{}});
             delete frame.dataset.swapping;
             return true;",
            motion = prefers_motion_script(),
        );
        let on_done = on_swap_complete;
        spawn(async move {
            let _ = document::eval(&script).await;
            if done_seq() < seq {
                done_seq.set(seq);
                if let Some(handler) = &on_done {
                    handler.call(());
                }
            }
        });
    }));

    let class = class_list("kit-blur-swap", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { id: "{id}", class,
            div { id: "{id}-in", key: "{seq}", class: "kit-blur-swap__current", "data-swap": "{seq}",
                {children}
            }
            if show_outgoing {
                if let Some(content) = outgoing {
                    div {
                        id: "{id}-out",
                        key: "{seq}",
                        class: "kit-blur-swap__outgoing",
                        "aria-hidden": "true",
                        "inert": "true",
                        {content}
                    }
                }
            }
        }
    }
}

/// One step of a `BlurSwapStepCard`: the tag above its sentence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepBody {
    pub tag: String,
    pub sentence: String,
}

/// Header, a body that swaps between `steps`, and a footer with one dot per step and a Next button.
/// The caller owns `current`, so Next is a caller decision.
#[component]
pub fn BlurSwapStepCard(
    #[props(into)] title: String,
    #[props(into)] status: String,
    #[props(into)] primary_label: String,
    #[props(default)] on_primary: Option<EventHandler<()>>,
    steps: Vec<StepBody>,
    current: usize,
    on_next: EventHandler<()>,
) -> Element {
    let current = current.min(steps.len().saturating_sub(1));
    let body = steps.get(current).cloned();
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class: "kit-blur-step",
            header { class: "kit-blur-step__header",
                div { class: "kit-blur-step__heading",
                    h3 { class: "kit-blur-step__title", "{title}" }
                    p { class: "kit-blur-step__status", "{status}" }
                }
                button {
                    r#type: "button",
                    class: "kit-blur-step__primary",
                    onclick: move |_| {
                        if let Some(handler) = &on_primary {
                            handler.call(());
                        }
                    },
                    "{primary_label}"
                }
            }
            BlurSwapContainer { content_key: current.to_string(), class: "kit-blur-step__body",
                if let Some(step) = body {
                    div { class: "kit-blur-step__panel",
                        span { class: "kit-blur-step__tag", "{step.tag}" }
                        p { class: "kit-blur-step__sentence", "{step.sentence}" }
                    }
                }
            }
            footer { class: "kit-blur-step__footer",
                ol { class: "kit-blur-step__dots", "aria-label": "Steps",
                    for i in 0..steps.len() {
                        li {
                            class: "kit-blur-step__dot",
                            "data-current": (i == current).then_some("true"),
                            "aria-current": (i == current).then_some("step"),
                        }
                    }
                }
                button {
                    r#type: "button",
                    class: "kit-blur-step__next",
                    onclick: move |_| on_next.call(()),
                    "Next"
                }
            }
        }
    }
}
