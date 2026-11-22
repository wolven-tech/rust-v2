use dioxus::prelude::*;
use rv2_kit::components::quick_action_palette::{
    Command, CommandGroup, QuickActionPalette, filtered_rows,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn cmd(id: &str, name: &str, note: &str, combo: Option<&str>) -> Command {
    Command {
        command_note: Some(note.into()),
        key_combo: combo.map(str::to_string),
        ..Command::new(id, name)
    }
}

fn groups() -> Vec<CommandGroup> {
    vec![
        CommandGroup {
            group_id: "suggested".into(),
            group_name: "Suggested".into(),
            commands: vec![
                cmd("sync", "Sync sources", "2 new", Some("Mod 1")),
                cmd("digest", "Morning digest", "Today", Some("Mod 2")),
                cmd("ledger", "Ledger export", "Monthly", Some("Mod 3")),
                cmd("plan", "Plan the week", "Mon", Some("Mod 4")),
                cmd("reply", "Draft reply", "Inbox", Some("Mod 5")),
            ],
        },
        CommandGroup {
            group_id: "nav".into(),
            group_name: "Go to".into(),
            commands: vec![
                cmd("inbox", "Inbox", "6 waiting on you", Some("g i")),
                cmd("runs", "Runs", "3 running", Some("g r")),
                cmd("billing", "Billing", "Renews 1 Dec", Some("g b")),
            ],
        },
        CommandGroup {
            group_id: "act".into(),
            group_name: "Actions".into(),
            commands: vec![
                cmd("new", "New request", "Start a run", Some("n")),
                cmd("export", "Export log", "Last 30 days", None),
            ],
        },
    ]
}

fn palette() -> String {
    render(|| {
        rsx! { QuickActionPalette { command_groups: groups(), empty_hint: "Search commands" } }
    })
}

#[test]
fn palette_lists_three_groups_in_order_with_ten_commands() {
    let html = palette();
    assert_eq!(html.matches("role=\"option\"").count(), 10);
    assert_eq!(html.matches("class=\"kit-qap__heading\"").count(), 3);
    let suggested = html.find("Suggested").expect("Suggested heading");
    let go_to = html.find("Go to").expect("Go to heading");
    let actions = html.find("Actions").expect("Actions heading");
    assert!(suggested < go_to && go_to < actions);
    assert!(html.contains("10 commands"));
}

#[test]
fn palette_rest_state_highlights_only_the_first_row() {
    let html = palette();
    assert_eq!(html.matches("data-active=\"true\"").count(), 1);
    assert!(html.contains("Sync sources"));
    assert!(!html.contains("✓"));
}

#[test]
fn two_key_sequences_render_as_two_keycaps() {
    let html = palette();
    assert!(html.contains("class=\"kit-qap__key\">g</kbd>"));
    assert!(html.contains("class=\"kit-qap__key\">i</kbd>"));
    assert!(html.contains("class=\"kit-qap__key\">n</kbd>"));
}

#[test]
fn suggested_rows_carry_quick_slot_keycaps_one_to_five() {
    let html = palette();
    for slot in 1..=5 {
        assert!(
            html.contains(&format!("class=\"kit-qap__key\">{slot}</kbd>")),
            "missing keycap {slot}"
        );
    }
}

#[test]
fn filtering_hides_groups_with_no_match() {
    let rows = filtered_rows(&groups(), "bill");
    assert_eq!(rows, vec![(1, 2)]);
}

#[test]
fn palette_rows_keep_group_order_when_unfiltered() {
    let rows = filtered_rows(&groups(), "");
    assert_eq!(rows.len(), 10);
    assert_eq!(rows.first(), Some(&(0, 0)));
    assert_eq!(rows.last(), Some(&(2, 1)));
}
