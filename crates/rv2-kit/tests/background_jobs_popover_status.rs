use dioxus::prelude::*;
use rv2_kit::components::background_jobs_popover::{
    BackgroundJob, BackgroundJobsPopover, JobState, status_word,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn open_popover() -> Element {
    let jobs = vec![
        BackgroundJob {
            job_id: "a".into(),
            job_title: "Sync invoices".into(),
            current_step: "3 over the approval limit".into(),
            steps_done: 7,
            steps_total: 10,
            job_state: JobState::NeedsYou,
        },
        BackgroundJob {
            job_id: "b".into(),
            job_title: "Backup photos".into(),
            current_step: "Paused on low battery".into(),
            steps_done: 2,
            steps_total: 8,
            job_state: JobState::Paused,
        },
    ];
    rsx! {
        BackgroundJobsPopover { job_list: jobs, default_open: true }
    }
}

#[test]
fn status_pills_read_in_title_case_while_the_summary_stays_a_sentence() {
    let html = render(open_popover);
    assert!(html.contains("kit-jobs__badge\">Needs you<"));
    assert!(html.contains("kit-jobs__pill\">Paused<"));
    assert!(html.contains("1 needs you"));
}

#[test]
fn status_word_capitalises_only_the_first_letter() {
    assert_eq!(status_word("needs you"), "Needs you");
    assert_eq!(status_word(""), "");
}
