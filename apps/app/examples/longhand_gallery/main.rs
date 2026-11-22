use dioxus::prelude::*;
use rv2_affect::Rv2Theme;

mod chat;
mod costs;
mod data;
mod first_run;
mod goals;
mod mission_control;
mod outputs;
mod review_job;
mod settings;
mod skill_editor;
mod style;
mod tray;
mod work_board;
mod workflow_builder;

/// Epoch milliseconds at 2026-11-21T00:00Z. Scene data is written relative to it.
pub const NOW_MS: u64 = 1_795_219_200_000;

const SCENES: [(&str, &str); 12] = [
    ("work_board", "My Work Board"),
    ("review_job", "Review a Job"),
    ("skill_editor", "Skill editor"),
    ("workflow_builder", "Workflow builder"),
    ("mission_control", "Mission Control"),
    ("costs", "Costs and budgets"),
    ("chat", "Unified chat"),
    ("goals", "Goals and results"),
    ("outputs", "Stored outputs"),
    ("first_run", "First run"),
    ("settings", "Models and tools"),
    ("tray", "Tray and notices"),
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
.gl-top { padding: 20px 16px 12px; max-width: 1280px; margin: 0 auto; box-sizing: border-box; }
.gl-title { margin: 0 0 12px; font: 600 20px/1.3 system-ui, sans-serif; letter-spacing: -0.01em; }
.gl-toggles { display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 10px; }
.gl-toggle { display: inline-flex; gap: 6px; align-items: center; min-height: 30px; padding: 0 10px; border: 1px solid var(--color-rule, #e2e8f0); border-radius: 999px; font-size: 13px; cursor: pointer; background: var(--color-surface, #fff); }
.gl-toggle:has(input:checked) { border-color: var(--color-ink-muted, #64748b); }
.gl-toggle input { margin: 0; accent-color: var(--color-ink, #0f172a); }
.gl-nav { display: flex; flex-wrap: wrap; gap: 4px; font-size: 13px; }
.gl-nav a { color: var(--color-ink-muted, #475569); text-decoration: none; padding: 4px 10px; border-radius: 6px; }
.gl-nav a:hover { background: var(--color-raised, #f1f5f9); color: inherit; }
.gl button:not([class]) { font: 500 13px/1 system-ui, sans-serif; min-height: 30px; padding: 0 12px; border: 1px solid var(--color-rule, #e2e8f0); border-radius: 6px; background: var(--color-surface, #fff); color: inherit; cursor: pointer; }
.gl button:not([class]):hover { background: var(--color-raised, #f1f5f9); }
.gl button:not([class])[aria-pressed="true"] { background: var(--color-ink, #0f172a); color: var(--color-on-ink, #fff); border-color: transparent; }
.gl button:not([class]):focus-visible { outline: 2px solid var(--color-ink, #0f172a); outline-offset: 2px; }
.gl-main { padding: 16px; max-width: 1280px; margin: 0 auto; box-sizing: border-box; }
.gl-sec { padding-top: 24px; border-top: 1px solid #00000014; }
.gl-sec > h2 { font-size: 22px; margin: 0 0 4px; }
.gl-out { font: 13px ui-monospace, monospace; opacity: 0.8; min-height: 1.4em; }
.gl-canvas { position: relative; border: 1px dashed #0000002a; border-radius: 8px; padding: 12px; box-sizing: border-box; overflow: hidden; }
.gl[data-theme="dark"] .gl-canvas, .gl[data-theme="dark"] .gl-top, .gl[data-theme="dark"] .gl-sec { border-color: #ffffff2a; }
@media (max-width: 480px) { .gl-main { padding: 16px; } }
"#;

pub fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut dark = use_signal(|| false);
    // Off until the hash is read, so `#only=<scene>` can isolate one scene
    // before anything else mounts. No hash shows every scene.
    let mut shown = use_signal(|| [false; SCENES.len()]);
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
            let only = hash.strip_prefix("#only=").unwrap_or_default();
            let only = (!only.is_empty()).then(|| only.to_string());
            shown.set(std::array::from_fn(|i| {
                only.as_deref().is_none_or(|id| id == SCENES[i].0)
            }));
        });
    });
    let theme = if dark() { "dark" } else { "light" };
    let on = shown();
    rsx! {
        document::Title { "Longhand scenes" }
        document::Meta { name: "robots", content: "noindex" }
        style { {STYLE} }
        style { {style::SCENE_CSS} }
        Rv2Theme {}
        div { class: "gl", "data-theme": "{theme}",
            header { class: "gl-top",
                h1 { class: "gl-title", "Longhand, screens in use" }
                div { class: "gl-toggles",
                    label { class: "gl-toggle",
                        input {
                            r#type: "checkbox",
                            checked: dark(),
                            onchange: move |e| dark.set(e.checked()),
                        }
                        "Dark"
                    }
                    for (index, (id, title)) in SCENES.into_iter().enumerate() {
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
                nav { class: "gl-nav", aria_label: "Scenes",
                    for (id, title) in SCENES {
                        a { href: "#{id}", "{title}" }
                    }
                }
            }
            main { class: "gl-main lh-main",
                if on[0] { work_board::WorkBoard {} }
                if on[1] { review_job::ReviewJob {} }
                if on[2] { skill_editor::SkillEditor {} }
                if on[3] { workflow_builder::WorkflowBuilder {} }
                if on[4] { mission_control::MissionControl {} }
                if on[5] { costs::Costs {} }
                if on[6] { chat::UnifiedChat {} }
                if on[7] { goals::Goals {} }
                if on[8] { outputs::StoredOutputs {} }
                if on[9] { first_run::FirstRun {} }
                if on[10] { settings::ModelsAndTools {} }
                if on[11] { tray::TrayAndNotices {} }
            }
        }
    }
}

/// One scene: a short heading and the user story it serves, then the composed screen.
#[component]
pub fn Scene(
    id: &'static str,
    title: &'static str,
    story: &'static str,
    children: Element,
) -> Element {
    rsx! {
        section { id: "{id}", class: "gl-sec lh-scene", "data-gl": "{id}",
            h2 { "{title}" }
            p { class: "lh-desc", "{story}" }
            {children}
        }
    }
}

/// The gallery clock: `NOW_MS` plus the seconds since the page loaded.
pub fn gallery_now() -> u64 {
    try_consume_context::<Signal<u64>>().map_or(NOW_MS, |clock| clock())
}
