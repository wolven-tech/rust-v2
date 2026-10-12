use dioxus::prelude::*;
use rv2_ui::CommandSelect;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut workspace = use_signal(|| "strand".to_string());
    let mut count = use_signal(|| 0usize);
    let mut long_value = use_signal(|| "area-0".to_string());
    let mut disabled = use_signal(|| false);
    let options = vec![
        ("strand".into(), "Strand Property Hunt".into()),
        ("family".into(), "Family workbook".into()),
        ("development".into(), "Developer team".into()),
    ];
    rsx! {
        document::Title { "Shared command-select preview" }
        document::Meta { name: "robots", content: "noindex" }
        style { "body {{ margin: 0; background: #f5f2e7; color: #253a55; font: 16px system-ui; }} main {{ max-width: 680px; margin: auto; padding: 24px; }} section {{ max-width: 340px; margin-block: 24px; }} button {{ min-height: 44px; }}" }
        main {
            h1 { "Shared command-select" }
            p { "Synthetic workspaces. No account or network writes." }
            section {
                CommandSelect {
                    id: "workspace", label: "Your workspace", value: workspace(), options,
                    disabled: disabled(), search_placeholder: "Find workspace…",
                    onchange: move |value| { workspace.set(value); count += 1; },
                }
            }
            output { "Selected ID: {workspace}. Changes: {count}." }
            button { onclick: move |_| disabled.set(!disabled()), "Toggle disabled" }
            section {
                CommandSelect {
                    id: "areas", label: "Long list", value: long_value(),
                    options: (0..40).map(|i| (format!("area-{i}"), format!("Area {i:02} · property research workspace"))).collect::<Vec<_>>(),
                    onchange: move |value| long_value.set(value),
                }
            }
            output { "Area ID: {long_value}" }
            section {
                CommandSelect { id: "empty", label: "Empty options", value: "", options: vec![], onchange: |_| {} }
            }
            a { href: "#after", "After controls" }
        }
    }
}
