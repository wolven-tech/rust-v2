//! Amendment 6 of the assistant chat with progress panel: at-rest structure and pure rules.

use dioxus::prelude::*;
use rv2_kit::components::prompt_input_bar::ApprovalPolicy;
use rv2_kit::screens::assistant_chat_progress_panel::{
    AssistantChatProgressPanel, CallState, Message, Output, OutputKind, Part, Step, StepState,
    ToolCall, UsageRecord, app_mark, earlier_steps_words, folded_count, pages_words, uploads_words,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn finished(id: &str, tool: &str, label: &str) -> Part {
    Part::Tool(ToolCall {
        id: id.into(),
        tool_name: tool.into(),
        app_label: None,
        entry_label: label.into(),
        started_at: None,
        duration: Some(1_200),
        arguments_data: "{}".into(),
        result_data: Some("ok".into()),
        call_state: CallState::Done,
        error: None,
    })
}

fn rest_chat() -> Vec<Message> {
    vec![
        Message::User {
            id: "u1".into(),
            text: "Compare last quarter's hosting costs and draft a reply.".into(),
        },
        Message::Assistant {
            id: "a1".into(),
            parts: vec![
                finished("t1", "read_sheet", "Read the hosting ledger"),
                Part::Text("The ledger shows a rise of 18%.".into()),
                finished("t2", "search_web", "Check the vendor's price page"),
                Part::Text("Three points: the new price, the renewal date, the seat limit.".into()),
            ],
        },
        Message::User {
            id: "u2".into(),
            text: "Keep it under 120 words.".into(),
        },
        Message::Assistant {
            id: "a2".into(),
            parts: vec![
                finished("t3", "write_draft", "Draft the reply"),
                Part::Text("The draft is saved.".into()),
            ],
        },
    ]
}

fn rest_steps() -> Vec<Step> {
    [
        "Read the ledger",
        "Collect price pages",
        "Check the seat limits",
        "Draft the reply",
        "Save the draft",
    ]
    .iter()
    .enumerate()
    .map(|(n, label)| Step {
        id: format!("s{}", n + 1),
        entry_label: label.to_string(),
        step_state: StepState::Done,
    })
    .collect()
}

fn rest_outputs() -> Vec<Output> {
    vec![Output {
        entry_name: "hosting-reply-november.docx".into(),
        output_kind: OutputKind::Document,
        app_name: Some("Docs".into()),
        preview_lines: vec!["Hi Harbor team.".into(), "Reply by Friday.".into()],
    }]
}

fn rest_usage() -> UsageRecord {
    UsageRecord {
        upload_names: vec![
            "ledger-q3.csv".into(),
            "terms-2025.pdf".into(),
            "invoice-oct.pdf".into(),
        ],
        app_names: vec!["Sheets".into(), "Docs".into(), "Mail".into()],
        web_search_pages: Some(4),
        skill_names: vec!["Vendor letters".into()],
    }
}

fn rest_app() -> Element {
    rsx! {
        AssistantChatProgressPanel {
            active_chat: rest_chat(),
            step_list: rest_steps(),
            output_list: rest_outputs(),
            usage_record: rest_usage(),
            approval_policy: ApprovalPolicy::AutoSafe,
            on_send: move |_: String| {},
        }
    }
}

fn count(html: &str, needle: &str) -> usize {
    html.matches(needle).count()
}

#[test]
fn at_rest_thread_holds_two_user_messages_three_text_blocks_and_three_tool_cards() {
    let html = render(rest_app);
    assert_eq!(count(&html, "kit-pp__msg--user"), 2);
    assert_eq!(count(&html, "kit-pp__text"), 3);
    assert_eq!(count(&html, "kit-pp__tool\""), 3);
    assert_eq!(count(&html, ">Done<"), 3);
}

#[test]
fn bar_button_renders_while_the_panel_is_closed_and_no_panel_before_the_viewport_is_read() {
    // The panel opens only after the viewport is read in the browser, so SSR shows the closed state.
    let html = render(rest_app);
    assert_eq!(count(&html, "kit-pp__bar"), 1);
    assert!(!html.contains("kit-pp__panel"));
}

#[test]
fn earlier_steps_fold_into_a_line_that_names_their_count() {
    assert_eq!(folded_count(&rest_steps(), 2), 3);
    assert_eq!(earlier_steps_words(3), "3 earlier steps");
    assert_eq!(earlier_steps_words(1), "1 earlier step");
}

#[test]
fn connectors_show_one_letter_marks_not_app_names() {
    assert_eq!(app_mark("Sheets"), "S");
    assert_eq!(app_mark("mail"), "M");
    assert_eq!(app_mark(""), "");
}

#[test]
fn uploads_row_names_one_file_and_a_count_of_the_others() {
    assert_eq!(
        uploads_words(&rest_usage().upload_names),
        "ledger-q3.csv and 2 more"
    );
    assert_eq!(
        uploads_words(&["ledger-q3.csv".to_string()]),
        "ledger-q3.csv"
    );
    assert_eq!(uploads_words(&[]), "");
}

#[test]
fn web_search_row_counts_pages_in_words() {
    assert_eq!(pages_words(1), "1 page");
    assert_eq!(pages_words(4), "4 pages");
}
