//! Multi-step wizard structure: the step indicator, heading, field stack and action row at rest,
//! and the review step on the last stage.

use dioxus::prelude::*;
use rv2_kit::components::multi_step_wizard_form::{
    FormValues, MultiStepWizardForm, ReviewRow, StepContext, Task, WizardField, WizardReview,
    WizardStep,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn demo_steps() -> Vec<WizardStep> {
    vec![
        WizardStep {
            step_id: "workspace".into(),
            step_title: "Workspace".into(),
            step_heading: Some("Who is this workspace for?".into()),
            step_description: Some("Name it and choose its address.".into()),
            render_step: Callback::new(|_ctx: StepContext| {
                rsx! {
                    WizardField {
                        field_id: "wizard-address",
                        field_name: "address",
                        label: "Address",
                        prefix: "workspace.example/",
                        value: String::new(),
                        hint: "Lowercase letters and dashes.",
                        on_input: move |_: String| {},
                    }
                }
            }),
            check_step: None,
        },
        WizardStep {
            step_id: "team".into(),
            step_title: "Team".into(),
            step_heading: Some("Who else will work here?".into()),
            step_description: None,
            render_step: Callback::new(
                |_ctx: StepContext| rsx! { p { "Invite teammates later." } },
            ),
            check_step: None,
        },
        WizardStep {
            step_id: "plan".into(),
            step_title: "Plan".into(),
            step_heading: Some("Pick a plan".into()),
            step_description: None,
            render_step: Callback::new(
                |_ctx: StepContext| rsx! { p { "Plans can change later." } },
            ),
            check_step: None,
        },
        WizardStep {
            step_id: "review".into(),
            step_title: "Review".into(),
            step_heading: Some("Check the details".into()),
            step_description: None,
            render_step: Callback::new(|ctx: StepContext| {
                rsx! {
                    WizardReview {
                        rows: vec![
                            ReviewRow { row_label: "Name".into(), row_value: "Acme".into(), edit_step: Some(0) },
                            ReviewRow { row_label: "Address".into(), row_value: "acme".into(), edit_step: Some(0) },
                            ReviewRow { row_label: "Plan".into(), row_value: "Starter".into(), edit_step: Some(2) },
                        ],
                        go_to: ctx.go_to,
                    }
                }
            }),
            check_step: None,
        },
    ]
}

fn on_complete(_values: FormValues) -> Task {
    Box::pin(async { Ok(()) })
}

fn first_step_app() -> Element {
    rsx! {
        MultiStepWizardForm {
            step_list: demo_steps(),
            final_label: "Create workspace",
            on_complete: Callback::new(on_complete),
            current_index: Some(0),
        }
    }
}

fn last_step_app() -> Element {
    rsx! {
        MultiStepWizardForm {
            step_list: demo_steps(),
            final_label: "Create workspace",
            on_complete: Callback::new(on_complete),
            current_index: Some(3),
        }
    }
}

#[test]
fn wizard_stacks_indicator_heading_fields_then_actions() {
    let html = render(first_step_app);
    let indicator = html.find("kit-wiz__steps").expect("indicator");
    let heading = html.find("Who is this workspace for?").expect("heading");
    let description = html
        .find("Name it and choose its address.")
        .expect("description");
    let field = html.find("kit-wiz__field").expect("field stack");
    let actions = html.find("kit-wiz__actions").expect("action row");
    assert!(indicator < heading && heading < description && description < field && field < actions);
}

#[test]
fn wizard_indicator_shows_four_steps_with_only_the_first_current() {
    let html = render(first_step_app);
    assert_eq!(html.matches("class=\"kit-wiz__step\"").count(), 4);
    assert_eq!(html.matches("data-state=\"current\"").count(), 1);
    assert_eq!(html.matches("data-state=\"todo\"").count(), 3);
    for label in ["Workspace", "Team", "Plan", "Review"] {
        assert!(html.contains(label));
    }
}

#[test]
fn wizard_prefix_sits_before_the_entry_and_hint_after_it() {
    let html = render(first_step_app);
    let prefix = html.find("workspace.example/").expect("prefix");
    let input = html.find("id=\"wizard-address\"").expect("address input");
    let hint = html.find("Lowercase letters and dashes.").expect("hint");
    assert!(prefix < input && input < hint);
    assert!(html.contains("aria-describedby=\"wizard-address-hint\""));
}

#[test]
fn wizard_first_step_has_no_back_control_and_no_error_at_rest() {
    let html = render(first_step_app);
    assert!(!html.contains(">Back<"));
    assert!(html.contains("Continue"));
    assert!(!html.contains("kit-wiz__error"));
    assert!(!html.contains("Creating"));
}

#[test]
fn wizard_narrow_compact_line_precedes_the_step_list() {
    let html = render(first_step_app);
    let compact = html.find("Step 1 of 4").expect("compact line");
    let list = html.find("kit-wiz__steps").expect("step list");
    assert!(compact < list);
}

#[test]
fn wizard_last_step_shows_final_label_and_review_rows_with_edit_links() {
    let html = render(last_step_app);
    assert!(html.contains("Create workspace"));
    assert!(html.contains(">Back<"));
    assert_eq!(html.matches("class=\"kit-wiz__review-row\"").count(), 3);
    assert_eq!(html.matches(">Edit<").count(), 3);
}
