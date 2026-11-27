use dioxus::prelude::*;
use rv2_affect::step_rail::{RailState, RailStep, StepRail, current_index};

const CSS: &str = include_str!("../assets/step-rail.css");

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn ritual(at: usize, live: RailState) -> Vec<RailStep> {
    ["Think", "Spec", "Approve", "Plan", "Build"]
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let state = match i.cmp(&at) {
                std::cmp::Ordering::Less => RailState::Finished,
                std::cmp::Ordering::Equal => live,
                std::cmp::Ordering::Greater => RailState::Queued,
            };
            RailStep::new(name.to_lowercase(), *name, state)
        })
        .collect()
}

fn building() -> Element {
    rsx! {
        StepRail { steps: ritual(4, RailState::Current), aria_label: "Ritual steps" }
    }
}

fn drifted() -> Element {
    rsx! {
        StepRail { steps: ritual(1, RailState::Errored) }
    }
}

fn awaiting() -> Element {
    rsx! {
        StepRail { steps: ritual(2, RailState::AwaitingApproval) }
    }
}

#[test]
fn aria_current_marks_only_the_current_step() {
    let html = render(building);
    assert_eq!(html.matches("aria-current=").count(), 1, "{html}");
    let at = html.find("aria-current=\"step\"").unwrap();
    let li = &html[html[..at].rfind("<li").unwrap()..];
    let li = &li[..li.find("</li>").unwrap()];
    assert!(li.contains("data-state=\"current\""), "{li}");
    assert!(li.contains(">Build<"), "{li}");
}

#[test]
fn the_list_is_an_ordered_list_named_by_its_aria_label() {
    let html = render(building);
    assert!(
        html.contains("<ol class=\"ui-step-rail\" aria-label=\"Ritual steps\""),
        "{html}"
    );
    assert_eq!(html.matches("<li").count(), 5);
}

#[test]
fn a_rail_without_an_aria_label_is_named_steps() {
    assert!(render(drifted).contains("aria-label=\"Steps\""));
}

#[test]
fn every_step_carries_its_state_word() {
    let html = render(building);
    assert_eq!(
        html.matches("<span class=\"ui-step-rail__state\">finished</span>")
            .count(),
        4
    );
    assert!(html.contains("<span class=\"ui-step-rail__state\">in progress</span>"));
    assert!(render(drifted).contains("<span class=\"ui-step-rail__state\">queued</span>"));
    assert!(
        render(awaiting)
            .contains("<span class=\"ui-step-rail__state\">awaiting your approval</span>")
    );
}

#[test]
fn the_connecting_line_is_hidden_from_assistive_tech_and_missing_after_the_last_step() {
    let html = render(building);
    let lines: Vec<_> = html
        .match_indices("<svg")
        .map(|(at, _)| &html[at..])
        .collect();
    assert_eq!(lines.len(), 4, "every step but the last has a line");
    for line in lines {
        let tag = &line[..line.find('>').unwrap()];
        assert!(tag.contains("aria-hidden=\"true\""), "{tag}");
    }
    assert_eq!(html.matches("pathLength=\"1\"").count(), 4);
    assert_eq!(
        html.matches("<span class=\"ui-step-rail__node\" aria-hidden=\"true\">")
            .count(),
        5
    );
}

#[test]
fn lines_after_finished_steps_are_drawn_and_the_rest_are_not() {
    let html = render(awaiting);
    assert_eq!(html.matches("ui-step-rail__line--drawn").count(), 2);
}

#[test]
fn a_drifted_step_carries_the_errored_state_and_is_current() {
    let html = render(drifted);
    let at = html.find("data-state=\"errored\"").unwrap();
    let li = &html[html[..at].rfind("<li").unwrap()..];
    let li = &li[..li.find("</li>").unwrap()];
    assert!(li.contains("aria-current=\"step\""), "{li}");
    assert!(li.contains(">drifted<"), "{li}");
    assert!(CSS.contains(".ui-step-rail__step[data-state=\"errored\"] {"));
    assert!(CSS.contains("animation: ui-step-rail-flash"));
}

#[test]
fn the_current_step_is_the_first_live_one() {
    let mut steps = ritual(2, RailState::AwaitingApproval);
    steps[4].state = RailState::Errored;
    assert_eq!(current_index(&steps), Some(2));
    assert_eq!(current_index(&ritual(9, RailState::Current)), None);
}

#[test]
fn the_stylesheet_turns_all_motion_off_under_reduced_motion() {
    let at = CSS
        .find("@media (prefers-reduced-motion: reduce)")
        .expect("a reduced-motion block");
    let block = &CSS[at..];
    let block = &block[..block.find("\n}\n").unwrap()];
    assert!(block.contains(".ui-step-rail *"), "{block}");
    assert!(block.contains("animation: none !important"), "{block}");
    for keyframes in [
        "ui-step-rail-draw",
        "ui-step-rail-breathe",
        "ui-step-rail-flash",
    ] {
        assert!(
            CSS.contains(&format!("@keyframes {keyframes}")),
            "{keyframes}"
        );
    }
}

#[test]
fn the_stylesheet_takes_every_colour_from_an_aui_hook() {
    for line in CSS
        .lines()
        .filter(|l| l.contains('#') || l.contains("rgba("))
    {
        let trimmed = line.trim();
        if trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }
        assert!(
            trimmed.contains("var(--aui-"),
            "literal colour outside a hook: {trimmed}"
        );
    }
}
