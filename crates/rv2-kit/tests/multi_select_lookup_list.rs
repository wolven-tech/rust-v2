use dioxus::prelude::*;
use rv2_kit::components::multi_select_lookup::{
    LookupOption, MultiSelectLookup, OptionImage, create_offered,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn person(value: &str, label: &str, group: &str) -> LookupOption {
    LookupOption {
        option_hint: Some("Role".into()),
        option_image: Some(OptionImage::Initials),
        option_group: Some(group.into()),
        match_terms: vec![label.to_lowercase()],
        ..LookupOption::new(value, label)
    }
}

fn approvers() -> Vec<LookupOption> {
    vec![
        person("ada", "Ada Lovelace", "Platform"),
        person("ben", "Ben Carter", "Platform"),
        person("cy", "Cy Mora", "Platform"),
        person("dana", "Dana Ruiz", "Support"),
        person("eli", "Eli Brandt", "Support"),
        person("farah", "Farah Nasser", "Operations"),
        person("gus", "Gus Holm", "Operations"),
    ]
}

fn open_list() -> String {
    render(|| {
        rsx! {
            MultiSelectLookup {
                option_list: approvers(),
                selected_values: Some(vec!["ben".to_string(), "dana".to_string()]),
                field_caption: Some("Approvers".to_string()),
                start_open: true,
            }
        }
    })
}

#[test]
fn approver_list_shows_group_headings_in_the_order_given() {
    let html = open_list();
    assert_eq!(html.matches("class=\"kit-msl__group\"").count(), 3);
    let platform = html.find("Platform").expect("Platform heading");
    let support = html.find("Support").expect("Support heading");
    let operations = html.find("Operations").expect("Operations heading");
    assert!(platform < support && support < operations);
}

#[test]
fn approver_rows_show_an_initials_tile_and_a_role_line() {
    let html = open_list();
    assert_eq!(html.matches("class=\"kit-msl__initials\"").count(), 7);
    assert_eq!(html.matches("class=\"kit-msl__hint\"").count(), 7);
}

#[test]
fn picked_approvers_carry_a_check_mark_and_the_foot_counts_them() {
    let html = open_list();
    assert_eq!(html.matches("class=\"kit-msl__check\"").count(), 2);
    assert!(html.contains("2 selected"));
    assert!(html.contains("class=\"kit-msl__key\""));
}

#[test]
fn approver_list_opens_with_no_row_active_and_no_create_row() {
    let html = open_list();
    assert!(!html.contains("data-active"));
    assert!(!html.contains("kit-msl__row--create"));
}

#[test]
fn create_row_is_offered_only_for_text_that_matches_no_option() {
    let options = approvers();
    assert!(create_offered(&options, "Oslo", true));
    assert!(!create_offered(&options, "ada lovelace", true));
    assert!(!create_offered(&options, "Oslo", false));
}

#[test]
fn approver_list_scrolls_inside_a_fixed_maximum_height() {
    let css = include_str!("../assets/multi-select-lookup.css");
    let rule = css
        .lines()
        .find(|line| line.starts_with(".kit-msl__list {"))
        .expect("list rule");
    assert!(rule.contains("max-height:") && rule.contains("overflow: auto"));
}
