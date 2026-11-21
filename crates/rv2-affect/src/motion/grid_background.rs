//! Fine line grid behind content. It drifts one cell per loop and fades towards the
//! edges; with `reveal` a circle grows from the centre the first time it scrolls into view.
//!
//! Port losses: none. The reveal's IntersectionObserver runs in `document::eval`.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id, prefers_motion_script};

const CSS: Asset = asset!("/assets/grid-background.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GridDirection {
    Up,
    Down,
    Left,
    Right,
    /// From top right to bottom left.
    #[default]
    Diagonal,
}

impl GridDirection {
    /// Drift in whole cells per loop, `(x, y)`.
    pub fn offset(self) -> (i32, i32) {
        match self {
            Self::Up => (0, -1),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
            Self::Diagonal => (-1, 1),
        }
    }
}

/// Inline custom properties the stylesheet reads. `speed` is seconds per cell.
pub fn grid_style(
    size: u32,
    direction: GridDirection,
    speed: f64,
    fade: f64,
    color: Option<&str>,
) -> String {
    let cell = f64::from(size);
    let (dx, dy) = direction.offset();
    let mut style = format!(
        "--ui-gb-size: {cell}px; --ui-gb-dx: {}px; --ui-gb-dy: {}px; --ui-gb-speed: {}s; --ui-gb-fade: {fade}%;",
        f64::from(dx) * cell,
        f64::from(dy) * cell,
        speed.max(0.0),
    );
    if let Some(color) = color {
        style.push_str(&format!(" --ui-gb-line: {color};"));
    }
    style
}

/// Resolves once the element is at least 20% visible, or straight away when there is no
/// observer or the viewer asked for reduced motion.
fn reveal_script(id: &str) -> String {
    format!(
        "const el = document.getElementById({id:?});
         const done = () => dioxus.send(true);
         if (!el || typeof IntersectionObserver === 'undefined' || !({motion})) {{ done(); }}
         else {{
           const io = new IntersectionObserver((entries) => {{
             if (entries.some((e) => e.isIntersecting)) {{ io.disconnect(); done(); }}
           }}, {{ threshold: 0.2 }});
           io.observe(el);
         }}",
        motion = prefers_motion_script(),
    )
}

#[component]
pub fn GridBackground(
    #[props(default = 48)] size: u32,
    #[props(default, into)] color: Option<String>,
    #[props(default = 3.0)] speed: f64,
    #[props(default)] direction: GridDirection,
    #[props(default = true)] reveal: bool,
    #[props(default = 70.0)] fade: f64,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let id = use_hook(|| format!("ui-gb-{}", next_id()));
    let mut shown = use_signal(|| !reveal);

    let target = id.clone();
    use_effect(move || {
        if !reveal {
            return;
        }
        let mut channel = document::eval(&reveal_script(&target));
        spawn(async move {
            if channel.recv::<bool>().await.is_ok() {
                shown.set(true);
            }
        });
    });

    let style = grid_style(size, direction, speed, fade, color.as_deref());
    let class = class_list("ui-gb", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            id: "{id}",
            class,
            style,
            "data-reveal": reveal.then_some(""),
            "data-shown": shown().then_some(""),
            "data-static": (speed <= 0.0).then_some(""),
            div { class: "ui-gb-grid", "aria-hidden": "true",
                div { class: "ui-gb-reveal",
                    div { class: "ui-gb-lines" }
                }
            }
            div { class: "ui-gb-content", {children} }
        }
    }
}
