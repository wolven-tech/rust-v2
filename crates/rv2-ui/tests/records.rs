//! Dialogs, toasts and the record-keeping instructions, rendered. Each test
//! pins the part of the contract that holds without sight of the colour.

use dioxus::prelude::*;
use rv2_ui::{
    ArmedDeleteButton, Card, Choice, ChoiceGroup, CompareRow, CompareTable, ConfirmDialog,
    DeleteClick, Dialog, DialogKind, Gate, GateList, ProcessRail, ProcessStep, RailLayout, Redline,
    RedlineDiff, Stamp, Surface, ToastQueue, Verdict,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn a_dialog_is_named_by_its_title_and_described_by_its_description() {
    let html = render(|| {
        rsx! {
            Dialog { id: "d", open: false, title: "Rename", description: "Seen by the client.", on_open_change: |_| {}, "Body" }
        }
    });
    assert!(html.contains("<dialog"), "{html}");
    assert!(html.contains(r#"aria-labelledby="d-title""#), "{html}");
    assert!(
        html.contains(r#"aria-describedby="d-description""#),
        "{html}"
    );
    assert!(html.contains(r#"id="d-title""#), "{html}");
    assert!(html.contains(r#"role="dialog""#), "{html}");
}

#[test]
fn a_drawer_is_the_same_dialog_with_another_kind() {
    let html = render(|| {
        rsx! {
            Dialog { id: "d", open: false, title: "Detail", kind: DialogKind::Drawer, on_open_change: |_| {} }
        }
    });
    assert!(html.contains(r#"data-kind="drawer""#), "{html}");
}

#[test]
fn a_confirm_dialog_is_an_alertdialog_that_focuses_the_safe_action_first() {
    let html = render(|| {
        rsx! {
            ConfirmDialog {
                id: "c",
                open: false,
                title: "Delete?",
                description: "Cannot be undone.",
                confirm_label: "Delete",
                on_open_change: |_| {},
                on_confirm: |_| {},
            }
        }
    });
    assert!(html.contains(r#"role="alertdialog""#), "{html}");
    let keep = html.find("Keep it").expect("cancel button");
    let autofocus = html
        .find("autofocus")
        .expect("autofocus on the cancel button");
    let delete = html.rfind("Delete<").expect("confirm button");
    assert!(autofocus < keep && keep < delete, "{html}");
}

#[test]
fn a_pending_confirm_cannot_be_dismissed_and_reports_busy() {
    let html = render(|| {
        rsx! {
            ConfirmDialog {
                id: "c",
                open: false,
                title: "Delete?",
                description: "Cannot be undone.",
                confirm_label: "Delete",
                pending: true,
                error: "The server refused.",
                on_open_change: |_| {},
                on_confirm: |_| {},
            }
        }
    });
    assert!(
        html.contains(r#"aria-busy=true"#) || html.contains(r#"aria-busy="true""#),
        "{html}"
    );
    assert!(html.contains("The server refused."), "{html}");
    assert!(html.contains(r#"role="alert""#), "{html}");
}

#[test]
fn the_toast_queue_keeps_the_newest_four_and_dismisses_by_id() {
    let mut queue = ToastQueue::default();
    let ids: Vec<u64> = (0..5)
        .map(|n| queue.push(Verdict::Note, None, format!("t{n}")))
        .collect();
    assert_eq!(queue.toasts().len(), ToastQueue::MAX);
    assert_eq!(queue.toasts()[0].message, "t1");
    queue.dismiss(ids[2]);
    assert!(queue.toasts().iter().all(|t| t.id != ids[2]));
    assert_eq!(queue.toasts().len(), 3);
}

#[test]
fn a_failure_toast_interrupts_and_outlives_a_success() {
    let mut queue = ToastQueue::default();
    queue.push(Verdict::Clear, None, "ok".into());
    queue.push(Verdict::Stop, None, "failed".into());
    let [ok, failed] = queue.toasts() else {
        panic!("two toasts");
    };
    assert!(!ok.assertive());
    assert!(failed.assertive());
    assert!(failed.life_ms() > ok.life_ms());
}

#[test]
fn a_delete_arms_on_the_first_click_and_confirms_on_the_second() {
    assert_eq!(DeleteClick::next(false), DeleteClick::Arm);
    assert_eq!(DeleteClick::next(true), DeleteClick::Confirm);
}

#[test]
fn an_armed_delete_says_what_the_next_click_does() {
    let html = render(|| {
        rsx! {
            ArmedDeleteButton { label: "Remove", confirm_label: "Click again to remove", armed: true, on_armed_change: |_| {}, on_confirm: |_| {} }
        }
    });
    assert!(html.contains("Click again to remove"), "{html}");
    assert!(
        html.contains(r#"data-armed=true"#) || html.contains(r#"data-armed="true""#),
        "{html}"
    );
}

#[test]
fn a_gate_speaks_its_state_and_hides_the_glyph() {
    let html = render(|| {
        rsx! {
            GateList {
                label: "Readiness",
                gates: vec![
                    Gate::new("Vector master", Verdict::Clear),
                    Gate::new("Rights", Verdict::Stop).detail("Unsigned"),
                ],
            }
        }
    });
    assert!(html.contains(r#"aria-label="Readiness""#), "{html}");
    assert!(html.contains("Passed: "), "{html}");
    assert!(html.contains("Failed: "), "{html}");
    assert_eq!(html.matches(r#"aria-hidden="true""#).count(), 2, "{html}");
    assert!(html.contains("Unsigned"), "{html}");
}

#[test]
fn a_redline_marks_removed_and_inserted_text_semantically() {
    let html = render(|| {
        rsx! {
            RedlineDiff { lines: vec![Redline::new("Hours", "10", "12")] }
        }
    });
    assert!(html.contains("<del"), "{html}");
    assert!(html.contains("<ins"), "{html}");
    assert!(html.contains("<dt"), "{html}");
}

#[test]
fn every_compare_cell_carries_its_column_name_for_the_stacked_layout() {
    let html = render(|| {
        rsx! {
            CompareTable {
                caption: "Plans",
                columns: vec!["Feature".into(), "Free".into(), "Studio".into()],
                rows: vec![CompareRow::new("Seats", vec!["1".into(), "3".into()])],
                highlight: Some(1),
            }
        }
    });
    assert!(html.contains("<caption"), "{html}");
    assert!(html.contains(r#"data-label="Free""#), "{html}");
    assert!(html.contains(r#"data-label="Studio""#), "{html}");
    assert!(html.contains(r#"scope="row""#), "{html}");
    assert_eq!(html.matches(r#"scope="col""#).count(), 3, "{html}");
}

#[test]
fn a_choice_group_is_native_radios_under_one_name_with_the_value_checked() {
    let html = render(|| {
        rsx! {
            ChoiceGroup {
                legend: "Split",
                name: "plan",
                choices: vec![Choice::new("a", "Split").hint("Each pays"), Choice::new("b", "Share")],
                value: "b",
                onchange: |_| {},
            }
        }
    });
    assert!(html.contains("<legend"), "{html}");
    assert_eq!(html.matches(r#"type="radio""#).count(), 2, "{html}");
    assert_eq!(html.matches(r#"name="plan""#).count(), 2, "{html}");
    let input = |value: &str| {
        html.split("<input")
            .find(|tag| tag.contains(&format!(r#"value="{value}""#)))
            .map(|tag| tag.split('>').next().unwrap_or_default().to_string())
            .expect("radio rendered")
    };
    assert!(input("b").contains("checked"), "{html}");
    assert!(!input("a").contains("checked"), "{html}");
    assert!(html.contains("Each pays"), "{html}");
}

#[test]
fn a_stamp_and_a_slab_take_their_look_from_operands() {
    let html = render(|| {
        rsx! {
            Stamp { verdict: Verdict::Clear, "Ready" }
            Card { surface: Surface::Slab, "Body" }
        }
    });
    assert!(html.contains("text-takeable-ink"), "{html}");
    assert!(html.contains("uppercase"), "{html}");
    assert!(html.contains("border-2 border-ink"), "{html}");
}

#[test]
fn a_column_rail_keeps_the_current_step_semantics() {
    let html = render(|| {
        rsx! {
            ProcessRail {
                label: "Timeline",
                layout: RailLayout::Column,
                active_step: Some(2),
                steps: vec![ProcessStep::new("Brief", "Done"), ProcessStep::new("Assets", "Now")],
            }
        }
    });
    assert!(html.contains(r#"data-layout="column""#), "{html}");
    assert!(html.contains(r#"aria-current="step""#), "{html}");
    assert!(html.contains("is-complete"), "{html}");
}

#[test]
fn the_page_entrance_releases_its_transform_so_the_toaster_stays_on_screen() {
    let css = include_str!("../assets/product-shell.css");
    let rule = css
        .lines()
        .find(|line| line.contains("animation: rv2-page-entrance"))
        .expect("page entrance animation rule");
    assert!(
        !rule.contains(" both") && !rule.contains(" forwards"),
        "{rule}"
    );
}
