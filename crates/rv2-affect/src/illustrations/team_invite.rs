//! Two connected teammates and an empty dashed slot. A cursor drags a third avatar into the
//! slot: it lands, the dashed line turns solid and the status dot lights up, then the scene
//! starts over. Laid out on a 600 × 400 grid inside a 3 : 2 frame.
//!
//! Port losses: none. The loop is CSS keyframes.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};

const CSS: Asset = asset!("/assets/team-invite.css");

#[derive(Clone, Debug, PartialEq)]
pub struct TeamInviteAvatar {
    /// Square portrait, shown in a circle.
    pub src: String,
    /// Name for the accessible description.
    pub name: Option<String>,
}

#[component]
fn Node(src: String, class: &'static str, online: bool) -> Element {
    rsx! {
        div { class: "ui-ti-node {class}", "aria-hidden": "true",
            span { class: "ui-ti-face",
                img { src, alt: "", draggable: "false" }
            }
            if online {
                span { class: "ui-ti-dot" }
            }
        }
    }
}

#[component]
pub fn TeamInvite(
    /// The two teammates that are already connected, then the one being invited.
    avatars: [TeamInviteAvatar; 3],
    #[props(default, into)] class: String,
    #[props(default)] label: Option<String>,
) -> Element {
    let id = use_hook(|| format!("ui-ti{}", next_id()));
    let names: Vec<String> = avatars
        .iter()
        .enumerate()
        .map(|(i, avatar)| {
            avatar
                .name
                .clone()
                .unwrap_or_else(|| format!("Teammate {}", i + 1))
        })
        .collect();
    let label = label.unwrap_or_else(|| {
        format!(
            "{} and {} are connected; {} is dragged into the empty slot and joins the team",
            names[0], names[1], names[2]
        )
    });
    let class = class_list("ui-ti", &class);
    let [a, b, c] = avatars;
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { class, role: "img", "aria-label": "{label}",
            svg { class: "ui-ti-lines", "viewBox": "0 0 600 400", "aria-hidden": "true",
                defs {
                    linearGradient { id: "{id}-line", x1: "0", y1: "0", x2: "1", y2: "0",
                        stop { class: "ui-ti-stop-a", offset: "0%" }
                        stop { class: "ui-ti-stop-b", offset: "100%" }
                    }
                }
                path { class: "ui-ti-line", d: "M162 159C214 159 222 105 258 105", stroke: "url(#{id}-line)" }
                path { class: "ui-ti-line ui-ti-line--dashed", d: "M162 159C214 165 214 245 248 245" }
                path { class: "ui-ti-line ui-ti-line--joined", d: "M162 159C214 165 214 245 248 245", "pathLength": "1", stroke: "url(#{id}-line)" }
                g { class: "ui-ti-slot",
                    circle { class: "ui-ti-line ui-ti-line--slot", cx: "290", cy: "245", r: "40" }
                    path { class: "ui-ti-plus", d: "M290 234v22M279 245h22" }
                }
            }

            Node { src: a.src, class: "ui-ti-node--a", online: true }
            Node { src: b.src, class: "ui-ti-node--b", online: true }

            div { class: "ui-ti-drag", "aria-hidden": "true",
                Node { src: c.src, class: "ui-ti-node--c", online: true }
                svg { class: "ui-ti-cursor", "viewBox": "-2.186 -0.256 18.514 22.277", "aria-hidden": "true",
                    path { transform: "rotate(21.37)", d: CURSOR }
                }
            }
        }
    }
}

const CURSOR: &str = "M13.92 4.07c1.69.66 3.03 1.18 3.94 1.68.46.25.87.53 1.16.86.31.35.51.79.48 1.29-.03.51-.27.92-.62 1.23-.33.3-.77.53-1.25.74-.96.4-2.36.78-4.12 1.26-.56.15-.98.27-1.26.37-.26.1-.38.18-.47.27-.09.09-.17.21-.27.47-.1.28-.22.7-.37 1.26-.48 1.76-.86 3.16-1.26 4.12-.2.49-.44.93-.74 1.25-.32.35-.72.6-1.23.62-.51.03-.94-.17-1.29-.48-.33-.29-.61-.7-.86-1.16-.5-.91-1.02-2.25-1.68-3.94L1.66 7.76C.96 5.99.41 4.59.16 3.51-.09 2.44-.12 1.39.63.63 1.39-.12 2.44-.09 3.51.16c1.08.25 2.48.8 4.25 1.5l6.16 2.41Z";
