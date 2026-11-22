use dioxus::prelude::*;
use rv2_affect::Rv2Theme;

mod additions;
mod agents;
mod blocks;
mod charts;
mod decisions;
mod decisions_a;
mod decisions_b;
mod decisions_c;
mod graphs;
mod illustrations;
mod inputs;
mod motion;
mod screens_a;
mod screens_agents;
mod screens_agents_b;
mod screens_b;
mod screens_b_canvas;
mod screens_frames;
mod timelines;

const SECTIONS: [(&str, &str); 14] = [
    ("agents", "Agents"),
    ("decisions", "Decisions and chat"),
    ("charts", "Charts"),
    ("inputs", "Inputs"),
    ("graphs", "Graphs"),
    ("timelines", "Timelines"),
    ("motion", "Motion"),
    ("blocks", "Blocks"),
    ("illustrations", "Illustrations"),
    ("screens_frames", "Frames"),
    ("screens_agents", "Agent screens"),
    ("screens_a", "Assistant screens"),
    ("screens_b", "Data screens"),
    ("additions", "New items"),
];

const STYLE: &str = r#"
body { margin: 0; background: #f8fafc; color: #0f172a; font: 15px/1.5 system-ui, sans-serif; }
body:has(.gl[data-theme="dark"]) { background: #0b0f17; }
.gl { min-height: 100vh; background: #f8fafc; color: #0f172a; }
.gl[data-theme="dark"] {
  background: #0b0f17; color: #e2e8f0;
  --color-ground: #0b0f17; --color-surface: #141a24; --color-raised: #1c2432;
  --color-ink: #e2e8f0; --color-ink-hover: #cbd5e1; --color-ink-muted: #94a3b8; --color-ink-faint: #64748b;
  --color-on-ink: #0f172a; --color-rule: #334155; --color-rule-soft: #1e293b; --color-accent: #e2e8f0;
  --color-takeable: #10b981; --color-takeable-soft: #064e3b; --color-takeable-ink: #6ee7b7;
  --color-caution: #f59e0b; --color-caution-ink: #fcd34d;
  --color-fault: #ef4444; --color-fault-ink: #fca5a5;
  --aui-ink: #e2e8f0; --aui-muted: #94a3b8; --aui-surface: #141a24; --aui-overlay: #141a24;
  --aui-subtle: #1c2432; --aui-border: #1e293b; --aui-border-strong: #334155;
  --aui-primary: #e2e8f0; --aui-on-primary: #0f172a;
}
.gl.gl-test {
  background: #fbf7f2; color: #2a1f17;
  --color-ground: #fbf7f2; --color-surface: #fffdf9; --color-raised: #f3ebe1;
  --color-ink: #2a1f17; --color-ink-hover: #4a382a; --color-ink-muted: #6b5545; --color-ink-faint: #a08a78;
  --color-on-ink: #fffdf9; --color-rule: #dccbb8; --color-rule-soft: #ece1d4; --color-accent: #0e7490;
  --color-takeable: #15803d; --color-takeable-soft: #dcfce7; --color-takeable-ink: #14532d;
  --color-caution: #ca8a04; --color-caution-ink: #713f12;
  --color-fault: #dc2626; --color-fault-ink: #7f1d1d;
  --color-id-1: #0e7490; --color-id-2: #6d28d9; --color-id-3: #3f6212; --color-id-4: #9333ea; --color-id-5: #1d4ed8; --color-id-6: #57534e;
  --color-series-1: #0e7490; --color-series-2: #6d28d9; --color-series-3: #3f6212; --color-series-4: #9333ea; --color-series-5: #1d4ed8; --color-series-6: #57534e;
  --kit-radius-sm: 2px; --kit-radius: 4px; --kit-radius-card: 4px; --kit-radius-lg: 6px; --kit-radius-pill: 4px;
  --kit-space-1: 3px; --kit-space-2: 6px; --kit-space-3: 9px; --kit-space-4: 12px; --kit-space-5: 15px; --kit-space-6: 18px;
  --kit-tint-bg-c: 0.06; --kit-tint-ink-c: 0.14;
  --kit-font: Georgia, "Iowan Old Style", serif;
}
.gl-top { padding: 20px 16px 12px; max-width: 1100px; margin: 0 auto; box-sizing: border-box; }
.gl-title { margin: 0 0 12px; font: 600 20px/1.3 system-ui, sans-serif; letter-spacing: -0.01em; }
.gl-toggles { display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 10px; }
.gl-toggle { display: inline-flex; gap: 6px; align-items: center; min-height: 30px; padding: 0 10px; border: 1px solid var(--color-rule, #e2e8f0); border-radius: 999px; font-size: 13px; cursor: pointer; background: var(--color-surface, #fff); }
.gl-toggle:has(input:checked) { border-color: var(--color-ink-muted, #64748b); }
.gl-toggle input { margin: 0; accent-color: var(--color-ink, #0f172a); }
.gl-nav { display: flex; flex-wrap: wrap; gap: 4px; font-size: 13px; }
.gl-nav a { color: var(--color-ink-muted, #475569); text-decoration: none; padding: 4px 10px; border-radius: 6px; }
.gl-nav a:hover { background: var(--color-raised, #f1f5f9); color: inherit; }
.gl button:not([class]), .gl .gl-btn { font: 500 13px/1 system-ui, sans-serif; min-height: 30px; padding: 0 12px; border: 1px solid var(--color-rule, #e2e8f0); border-radius: 6px; background: var(--color-surface, #fff); color: inherit; cursor: pointer; }
.gl button:not([class]):hover, .gl .gl-btn:hover { background: var(--color-raised, #f1f5f9); }
.gl button:not([class])[aria-pressed="true"] { background: var(--color-ink, #0f172a); color: var(--color-on-ink, #fff); border-color: transparent; }
.gl button:not([class]):focus-visible, .gl .gl-btn:focus-visible { outline: 2px solid var(--color-ink, #0f172a); outline-offset: 2px; }
.gl-main { padding: 16px; max-width: 1100px; margin: 0 auto; box-sizing: border-box; }
.gl-sec { padding-top: 24px; border-top: 1px solid #00000014; }
.gl-sec > h2 { font-size: 22px; margin: 0 0 12px; }
.gl-item { margin: 0 0 20px; }
.gl-item > h3 { font: 600 13px/1.4 system-ui, sans-serif; text-transform: uppercase; letter-spacing: 0.04em; opacity: 0.6; margin: 0 0 8px; }
.gl-row { display: flex; flex-wrap: wrap; gap: 12px; align-items: flex-start; }
.gl-stack { display: grid; gap: 12px; max-width: 360px; }
.gl-out { font: 12px/1.5 ui-monospace, SFMono-Regular, Menlo, monospace; opacity: 0.6; min-height: 1.4em; margin-top: 8px; padding-top: 6px; border-top: 1px dashed currentColor; }
.gl-out:not(:empty)::before { content: "Demo event: "; font-family: system-ui, sans-serif; }
.gl-canvas { position: relative; border: 1px dashed #0000002a; border-radius: 8px; padding: 12px; box-sizing: border-box; overflow: hidden; }
.gl[data-theme="dark"] .gl-canvas, .gl[data-theme="dark"] .gl-top, .gl[data-theme="dark"] .gl-sec { border-color: #ffffff2a; }
.gl-tag { font: 12px ui-monospace, monospace; opacity: 0.6; }
@media (max-width: 480px) { .gl-main { padding: 16px; } .gl-stack { max-width: none; } }
"#;

pub fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut dark = use_signal(|| false);
    // A third theme that moves only registers and --kit-* tokens. An item that
    // renders the same under it ignores the tokens somewhere.
    let mut test_theme = use_signal(|| false);
    // Off until the hash is read, so `#only=<section>` can isolate one section
    // before anything else mounts. No hash shows every section.
    let mut shown = use_signal(|| [false; SECTIONS.len()]);
    let mut only_item = use_context_provider(|| Signal::new(None::<String>));
    let mut clock = use_context_provider(|| Signal::new(NOW_MS));
    use_hook(move || {
        spawn(async move {
            loop {
                if document::eval(
                    "await new Promise((resolve) => setTimeout(resolve, 1000)); return true;",
                )
                .await
                .is_err()
                {
                    break;
                }
                clock += 1000;
            }
        });
    });
    use_hook(move || {
        spawn(async move {
            let hash = document::eval("return window.location.hash;")
                .await
                .ok()
                .and_then(|v| v.as_str().map(str::to_string))
                .unwrap_or_default();
            let rest = hash.strip_prefix("#only=").unwrap_or_default();
            let (section, item) = rest.split_once("&item=").unwrap_or((rest, ""));
            let only = (!section.is_empty()).then(|| section.to_string());
            only_item.set((!item.is_empty()).then(|| item.to_string()));
            shown.set(std::array::from_fn(|i| {
                only.as_deref().is_none_or(|id| id == SECTIONS[i].0)
            }));
        });
    });
    let theme = if dark() { "dark" } else { "light" };
    let root_class = if test_theme() && !dark() {
        "gl gl-test"
    } else {
        "gl"
    };
    let on = shown();
    rsx! {
        document::Title { "rv2-kit preview gallery" }
        document::Meta { name: "robots", content: "noindex" }
        style { {STYLE} }
        Rv2Theme {}
        div { class: "{root_class}", "data-theme": "{theme}",
            header { class: "gl-top",
                h1 { class: "gl-title", "rv2-kit preview" }
                div { class: "gl-toggles",
                    label { class: "gl-toggle",
                        input {
                            r#type: "checkbox",
                            checked: dark(),
                            onchange: move |e| dark.set(e.checked()),
                        }
                        "Dark"
                    }
                    label { class: "gl-toggle",
                        input {
                            r#type: "checkbox",
                            checked: test_theme(),
                            onchange: move |e| test_theme.set(e.checked()),
                        }
                        "Test theme"
                    }
                    for (index, (id, title)) in SECTIONS.into_iter().enumerate() {
                        label { class: "gl-toggle",
                            input {
                                r#type: "checkbox",
                                "data-section": "{id}",
                                checked: on[index],
                                onchange: move |e| shown.with_mut(|s| s[index] = e.checked()),
                            }
                            "{title}"
                        }
                    }
                }
                nav { class: "gl-nav", aria_label: "Sections",
                    for (id, title) in SECTIONS {
                        a { href: "#{id}", "{title}" }
                    }
                }
            }
            main { class: "gl-main",
                if on[0] { agents::Agents {} }
                if on[1] { decisions::Decisions {} }
                if on[2] { charts::Charts {} }
                if on[3] { inputs::Inputs {} }
                if on[4] { graphs::Graphs {} }
                if on[5] { timelines::Timelines {} }
                if on[6] { motion::Motion {} }
                if on[7] { blocks::Blocks {} }
                if on[8] { illustrations::Illustrations {} }
                if on[9] { screens_frames::ScreensFrames {} }
                if on[10] { screens_agents::ScreensAgents {} }
                if on[11] { screens_a::ScreensA {} }
                if on[12] { screens_b::ScreensB {} }
                if on[13] { additions::Additions {} }
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
pub fn Item(slug: &'static str, name: &'static str, children: Element) -> Element {
    let only_item = use_context::<Signal<Option<String>>>();
    let mounted = only_item().is_none_or(|only| only == slug);
    rsx! {
        article { class: "gl-item", "data-gl": "{slug}",
            h3 { "{name}" }
            if mounted { {children} }
        }
    }
}

/// Epoch milliseconds at 2026-11-21T00:00Z. Demo data is written relative to it.
pub const NOW_MS: u64 = 1_795_219_200_000;

/// The gallery clock: `NOW_MS` plus the seconds since the page loaded. Demos pass
/// `gallery_now()` as `clock_ms` so elapsed times advance and stay consistent
/// across sections.
pub fn gallery_now() -> u64 {
    try_consume_context::<Signal<u64>>().map_or(NOW_MS, |clock| clock())
}
