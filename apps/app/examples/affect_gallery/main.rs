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
.gl-top { padding: 20px 16px 12px; max-width: 1100px; margin: 0 auto; box-sizing: border-box; }
.gl-title { margin: 0 0 12px; font: 600 20px/1.3 system-ui, sans-serif; letter-spacing: -0.01em; }
.gl-toggles { display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 10px; }
.gl-toggle { display: inline-flex; gap: 6px; align-items: center; min-height: 30px; padding: 0 10px; border: 1px solid #0000001f; border-radius: 999px; font-size: 13px; cursor: pointer; }
.gl-toggle:has(input:checked) { border-color: #00000066; }
.gl[data-theme="dark"] .gl-toggle { border-color: #ffffff26; }
.gl-toggle input { margin: 0; accent-color: currentColor; }
.gl-nav { display: flex; flex-wrap: wrap; gap: 4px; font-size: 13px; }
.gl-nav a { color: inherit; opacity: 0.75; text-decoration: none; padding: 4px 10px; border-radius: 6px; }
.gl-nav a:hover { opacity: 1; background: #0000000d; }
.gl button:not([class]) { font: 500 13px/1 system-ui, sans-serif; min-height: 30px; padding: 0 12px; border: 1px solid #0000001f; border-radius: 6px; background: #fff; color: inherit; cursor: pointer; }
.gl button:not([class]):hover { background: #f4f4f5; }
.gl[data-theme="dark"] button:not([class]) { background: #1c1c1e; border-color: #ffffff26; }
.gl button:not([class]):focus-visible, .gl-input:focus-visible, .gl-link:focus-visible { outline: 2px solid currentColor; outline-offset: 2px; }
.gl-input { box-sizing: border-box; width: 100%; height: 36px; padding: 0 10px; border: 0; border-radius: 8px; background: var(--aui-surface, #fff); color: inherit; font: inherit; font-size: 14px; box-shadow: 0 0 0 1px var(--aui-border-strong, #0000001f), 0 1px 2px #0000000a; }
.gl-input[aria-invalid="true"] { box-shadow: 0 0 0 1px #e5484d; }
.gl-list { list-style: none; margin: 0; padding: 0; max-width: 520px; }
.gl-list li { padding: 10px 12px; font-size: 14px; border-bottom: 1px solid #0000000f; }
.gl[data-theme="dark"] .gl-list li { border-color: #ffffff14; }
.gl-link {color: inherit; font-weight: 500; text-decoration: underline; text-decoration-color: #0000004d; text-underline-offset: 3px; }
.gl-main { padding: 16px; max-width: 1100px; margin: 0 auto; box-sizing: border-box; }
.gl-sec { padding-top: 24px; border-top: 1px solid #00000014; }
.gl-sec h2 { font-size: 22px; margin: 0 0 12px; }
.gl-item { margin: 0 0 20px; }
.gl-item > h3 { font: 600 13px/1.4 system-ui, sans-serif; text-transform: uppercase; letter-spacing: 0.04em; opacity: 0.6; margin: 0 0 8px; }
.gl-row { display: flex; flex-wrap: wrap; gap: 12px; align-items: flex-start; }
.gl-stack { display: grid; gap: 12px; max-width: 360px; }
.gl-out { font: 12px/1.5 ui-monospace, SFMono-Regular, Menlo, monospace; opacity: 0.6; margin-top: 6px; }
.gl-box { width: 100%; max-width: 520px; }
.gl-canvas { position: relative; min-height: 220px; border: 1px dashed #0000002a; border-radius: 8px; padding: 12px; box-sizing: border-box; overflow: hidden; }
.gl-canvas-tall { min-height: 360px; }
.gl-tag { font: 12px ui-monospace, monospace; opacity: 0.6; }
.gl-stage { display: flex; justify-content: center; padding: 40px 24px; border: 1px solid #0000000d; border-radius: 12px; background: #f1f1f3; box-sizing: border-box; }
.gl-card { box-sizing: border-box; width: 100%; max-width: 560px; display: grid; gap: 20px; padding: 28px; border-radius: 12px; background: #fff; box-shadow: 0 1px 2px #0000000d, 0 0 0 1px #0000000d; }
.gl[data-theme="dark"] .gl-stage { background: #0b0b0c; border-color: #ffffff12; }
.gl[data-theme="dark"] .gl-card { background: #1a1a1c; box-shadow: 0 0 0 1px #ffffff14; }
.gl-ex { display: grid; gap: 10px; min-width: 0; }
.gl-ex + .gl-ex { padding-top: 20px; border-top: 1px solid #0000000f; }
.gl[data-theme="dark"] .gl-ex + .gl-ex { border-color: #ffffff14; }
.gl-ex-label { margin: 0; font: 500 12px/1.4 system-ui, sans-serif; opacity: 0.6; }
.gl-ex-body { display: flex; flex-wrap: wrap; gap: 12px; align-items: center; min-width: 0; }
.gl-ex-body > .gl-box { flex: 1 1 260px; }
.gl-ex-body > .gl-canvas-tall { width: 100%; }
.gl-space { position: relative; width: 100%; min-height: 340px; }
/* A contained layer's content box must exceed the 520px phone-sheet breakpoint (its padding is 16px a side) */
.gl-space-wide { width: calc(100% + 50px); min-height: 400px; margin: 0 -25px; }
.gl-page { position: absolute; inset: 0; display: grid; align-content: start; gap: 10px; padding: 18px 20px; box-sizing: border-box; font-size: 13px; }
.gl-page-row { display: flex; align-items: center; gap: 10px; padding: 10px 12px; border-radius: 10px; background: #0000000a; }
.gl[data-theme="dark"] .gl-page-row { background: #ffffff0f; }
.gl-metrics { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 12px; }
.gl-metric-k { font-size: 12px; opacity: 0.6; }
.gl-metric-v { font-size: 14px; font-variant-numeric: tabular-nums; }
.gl-tile { display: grid; place-items: center; flex: none; width: 32px; height: 32px; border-radius: 8px; background: #0000000a; }
.gl[data-theme="dark"] .gl-tile { background: #ffffff14; }
.gl-toolbar { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-top: 40px; padding: 6px 8px 6px 14px; border: 1px solid #0000000f; border-radius: 12px; }
.gl[data-theme="dark"] .gl-toolbar { border-color: #ffffff14; }
.gl-toolbar-tools { display: flex; gap: 4px; }
.gl-shortcuts { border: 1px solid #0000000f; border-radius: 12px; overflow: hidden; font-size: 13px; }
.gl[data-theme="dark"] .gl-shortcuts { border-color: #ffffff14; }
.gl-shortcuts > * { padding: 10px 14px; }
.gl-shortcuts-head { display: flex; align-items: center; justify-content: space-between; gap: 8px; border-bottom: 1px solid #0000000f; }
.gl-shortcuts-title { font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; opacity: 0.7; }
.gl[data-theme="dark"] .gl-shortcuts-head { border-color: #ffffff14; }
.gl-shortcut-row { display: flex; align-items: center; justify-content: space-between; gap: 12px; border-bottom: 1px solid #0000000a; }
.gl-shortcuts-foot { font-size: 12px; opacity: 0.6; }
.gl-card-wide { max-width: 800px; }
.gl-tree { display: grid; gap: 2px; padding: 8px 6px; font-size: 13px; }
.gl-tree-meta { font-size: 12px; opacity: 0.55; white-space: nowrap; }
.gl-foot { display: flex; align-items: center; justify-content: space-between; gap: 8px; padding-top: 12px; border-top: 1px solid #0000000f; font-size: 12px; opacity: 0.8; }
.gl[data-theme="dark"] .gl-foot { border-color: #ffffff14; }
@media (max-width: 480px) { .gl-main { padding: 16px; } .gl-stack { max-width: none; } .gl-stage { padding: 16px 12px; } .gl-card { padding: 16px; gap: 20px; } }
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
pub fn Item(name: &'static str, #[props(default)] wide: bool, children: Element) -> Element {
    let card = if wide {
        "gl-card gl-card-wide"
    } else {
        "gl-card"
    };
    rsx! {
        article { class: "gl-item",
            h3 { "{name}" }
            div { class: "gl-stage",
                div { class: "{card}", {children} }
            }
        }
    }
}

#[component]
pub fn Ex(label: &'static str, children: Element) -> Element {
    rsx! {
        div { class: "gl-ex",
            p { class: "gl-ex-label", "{label}" }
            div { class: "gl-ex-body", {children} }
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
