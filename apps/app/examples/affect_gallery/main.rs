use dioxus::prelude::*;
use rv2_affect::Rv2Theme;

mod content;
mod illustrations;
mod inputs;
mod navigation;
mod overlays;
mod primitives;
mod visuals;

const SECTIONS: [(&str, &str); 7] = [
    ("primitives", "Primitives"),
    ("inputs", "Inputs"),
    ("overlays", "Overlays"),
    ("navigation", "Navigation"),
    ("content", "AI and content"),
    ("visuals", "Motion and blocks"),
    ("illustrations", "Illustrations"),
];

const STYLE: &str = r#"
body { margin: 0; background: #fafafa; color: #1d1d1f; font: 15px/1.5 system-ui, sans-serif; }
body:has(.gl[data-theme="dark"]) { background: #121213; }
.gl { min-height: 100vh; background: #fafafa; color: #1d1d1f; }
.gl[data-theme="dark"] { background: #121213; color: #ededee; }
.gl-top { position: sticky; top: 0; z-index: 5; padding: 16px; background: inherit; border-bottom: 1px solid #00000014; }
.gl-title { margin: 0 0 8px; font-size: 20px; }
.gl-toggles { display: flex; flex-wrap: wrap; gap: 16px; margin-bottom: 8px; }
.gl-toggle { display: inline-flex; gap: 6px; align-items: center; min-height: 32px; }
.gl-nav { display: flex; flex-wrap: wrap; gap: 4px 12px; font-size: 14px; }
.gl-nav a { color: inherit; }
.gl-main { padding: 16px; max-width: 1100px; margin: 0 auto; box-sizing: border-box; }
.gl-sec { padding-top: 24px; border-top: 1px solid #00000014; }
.gl-sec h2 { font-size: 22px; margin: 0 0 12px; }
.gl-item { margin: 0 0 20px; }
.gl-item h3 { font: 600 13px/1.4 system-ui, sans-serif; text-transform: uppercase; letter-spacing: 0.04em; opacity: 0.6; margin: 0 0 8px; }
.gl-row { display: flex; flex-wrap: wrap; gap: 12px; align-items: flex-start; }
.gl-stack { display: grid; gap: 12px; max-width: 360px; }
.gl-out { font: 13px ui-monospace, monospace; opacity: 0.8; }
.gl-box { width: 100%; max-width: 520px; }
.gl-canvas { position: relative; min-height: 220px; border: 1px dashed #0000002a; border-radius: 8px; padding: 12px; box-sizing: border-box; overflow: hidden; }
.gl-canvas-tall { min-height: 360px; }
.gl-tag { font: 12px ui-monospace, monospace; opacity: 0.6; }
@media (max-width: 480px) { .gl-main { padding: 16px; } .gl-stack { max-width: none; } }
"#;

pub fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut rv2 = use_signal(|| false);
    let mut dark = use_signal(|| false);
    let theme = if dark() { "dark" } else { "light" };
    rsx! {
        document::Title { "rv2-affect preview gallery" }
        document::Meta { name: "robots", content: "noindex" }
        style { {STYLE} }
        if rv2() {
            Rv2Theme {}
        }
        div { class: "gl", "data-theme": "{theme}",
            header { class: "gl-top",
                h1 { class: "gl-title", "rv2-affect preview" }
                div { class: "gl-toggles",
                    label { class: "gl-toggle",
                        input {
                            r#type: "checkbox",
                            checked: rv2(),
                            onchange: move |e| rv2.set(e.checked()),
                        }
                        "rv2 registers"
                    }
                    label { class: "gl-toggle",
                        input {
                            r#type: "checkbox",
                            checked: dark(),
                            onchange: move |e| dark.set(e.checked()),
                        }
                        "Dark"
                    }
                }
                nav { class: "gl-nav", aria_label: "Sections",
                    for (id, title) in SECTIONS {
                        a { href: "#{id}", "{title}" }
                    }
                }
            }
            main { class: "gl-main",
                primitives::Primitives {}
                inputs::Inputs {}
                overlays::Overlays {}
                navigation::Navigation {}
                content::Content {}
                visuals::Visuals {}
                illustrations::Illustrations {}
            }
        }
    }
}

#[component]
pub fn Section(id: &'static str, title: &'static str, children: Element) -> Element {
    rsx! {
        section { id: "{id}", class: "gl-sec",
            h2 { "{title}" }
            {children}
        }
    }
}

#[component]
pub fn Item(name: &'static str, children: Element) -> Element {
    rsx! {
        article { class: "gl-item",
            h3 { "{name}" }
            {children}
        }
    }
}

pub fn glyph(path: &'static str) -> Element {
    rsx! {
        svg {
            view_box: "0 0 16 16",
            width: "16",
            height: "16",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.6",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: path }
        }
    }
}
