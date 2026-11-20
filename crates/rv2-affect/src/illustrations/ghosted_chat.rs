//! Three unanswered outgoing messages, the last with a red shine and a soft glow, marked "Read".
//! The messages arrive one by one once the scene scrolls into view. On hover a thumbs-down reply
//! pops in. The 360 × 240 artboard is scaled to the width of its container.
//!
//! Port losses: none. The scale and the arrival trigger come from `document::eval` (a
//! ResizeObserver and an IntersectionObserver) and are written onto the stage directly.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};

const CSS: Asset = asset!("/assets/ghosted-chat.css");

const SCRIPT: &str = r#"
const root = document.getElementById("__ID__");
if (!root) return;
const stage = root.querySelector(".ui-gc-stage");
const fit = () => {
  const scale = root.clientWidth / 360;
  stage.style.transform = `scale(${scale})`;
  stage.style.visibility = scale > 0 ? "visible" : "hidden";
};
fit();
new ResizeObserver(fit).observe(root);
const arrive = new IntersectionObserver(([entry]) => {
  if (!entry.isIntersecting) return;
  stage.dataset.play = "";
  arrive.disconnect();
}, { threshold: 0.35 });
arrive.observe(root);
setTimeout(() => { stage.dataset.play = ""; }, 3000);
"#;

#[component]
pub fn GhostedChat(
    #[props(default, into)] class: String,
    #[props(default = "A chat with three unanswered messages marked as read".to_string(), into)]
    label: String,
) -> Element {
    let id = use_hook(|| format!("ui-gc-{}", next_id()));
    let effect_id = id.clone();
    use_effect(move || {
        document::eval(&SCRIPT.replace("__ID__", &effect_id));
    });

    let class = class_list("ui-gc", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { id: "{id}", class, role: "img", "aria-label": "{label}",
            div { class: "ui-gc-stage", "aria-hidden": "true", style: "transform: scale(0)",
                div { class: "ui-gc-content",
                    span { class: "ui-gc-status", "No new messages" }
                    div { class: "ui-gc-bubbles",
                        div { class: "ui-gc-bubble",
                            span { "Hi team!" }
                        }
                        div { class: "ui-gc-bubble",
                            span { "Are you by any chance looking for a recruiter?" }
                        }
                        div { class: "ui-gc-bubble ui-gc-bubble-last",
                            span { class: "ui-gc-alert", "Hello?" }
                        }
                        span { class: "ui-gc-read", "Read" }
                    }
                    div { class: "ui-gc-bubble ui-gc-reply",
                        span { "👎" }
                    }
                }
            }
        }
    }
}
