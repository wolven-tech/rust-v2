use dioxus::prelude::*;
use rv2_kit::components::model_selector::{
    MenuSide, ModelOption, ModelSelector, ThinkingEffort, bars_filled, first_token_text,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn rated(id: &str, name: &str, ranks: [u8; 3], seconds: f64) -> ModelOption {
    ModelOption {
        speed_rank: Some(ranks[0]),
        quality_rank: Some(ranks[1]),
        cost_rank: Some(ranks[2]),
        first_token_time: Some(seconds),
        price_per_million: Some(1.0),
        ..ModelOption::new(id, name)
    }
}

fn options() -> Vec<ModelOption> {
    vec![
        rated("fast", "Model Fast", [5, 2, 2], 0.6),
        rated("deep", "Model Deep", [2, 5, 4], 1.9),
        ModelOption::new("auto", "Model Auto"),
    ]
}

fn open_menu(thinking: bool) -> String {
    let app: fn() -> Element = if thinking {
        || {
            rsx! {
                ModelSelector {
                    model_options: options(),
                    selected_model_id: Some("deep".to_string()),
                    initial_highlight: Some("fast".to_string()),
                    initial_open: true,
                    menu_side: MenuSide::Top,
                    thinking_on: Some(true),
                }
            }
        }
    } else {
        || {
            rsx! {
                ModelSelector {
                    model_options: options(),
                    selected_model_id: Some("deep".to_string()),
                    initial_open: true,
                    thinking_on: Some(false),
                }
            }
        }
    };
    render(app)
}

#[test]
fn model_menu_header_names_the_four_columns_in_one_row() {
    let html = open_menu(true);
    let head = html
        .split("class=\"kit-ms__head\"")
        .nth(1)
        .expect("header row");
    let head = head.split("</div>").next().expect("header end");
    for label in ["Model", "Speed", "Quality", "Cost"] {
        assert!(head.contains(label), "header lacks {label}");
    }
}

#[test]
fn model_menu_lists_one_row_per_option_and_the_router_reads_varies() {
    let html = open_menu(true);
    assert_eq!(html.matches("class=\"kit-ms__row\"").count(), 3);
    assert_eq!(html.matches("class=\"kit-ms__varies\"").count(), 1);
    assert_eq!(html.matches("class=\"kit-ms__gauge\"").count(), 6);
}

#[test]
fn model_menu_marks_the_selected_row_with_a_check_mark() {
    let html = open_menu(true);
    assert_eq!(html.matches("aria-selected=\"true\"").count(), 1);
    assert_eq!(html.matches("✓").count(), 1);
}

#[test]
fn model_menu_hides_the_effort_group_while_thinking_is_off() {
    assert!(!open_menu(false).contains("kit-ms__effort"));
    let on = open_menu(true);
    assert_eq!(on.matches("role=\"radio\"").count(), 3);
    assert!(on.contains("aria-checked=\"true\""));
}

#[test]
fn model_footer_shows_the_highlighted_row_not_the_selected_one() {
    let html = open_menu(true);
    assert!(html.contains("First token 0.6s"));
    assert!(!html.contains("First token 1.9s"));
}

#[test]
fn effort_moves_one_step_and_clamps_at_the_ends() {
    assert_eq!(ThinkingEffort::Low.step(1), ThinkingEffort::Medium);
    assert_eq!(ThinkingEffort::Low.step(-1), ThinkingEffort::Low);
    assert_eq!(ThinkingEffort::High.step(1), ThinkingEffort::High);
}

#[test]
fn effort_scales_the_first_token_estimate_only_while_thinking() {
    assert_eq!(
        first_token_text(Some(1.0), ThinkingEffort::High, true),
        "1.6s"
    );
    assert_eq!(
        first_token_text(Some(1.0), ThinkingEffort::High, false),
        "1.0s"
    );
}

#[test]
fn ranks_outside_the_scale_fill_no_more_than_five_bars() {
    assert_eq!(bars_filled(None), 0);
    assert_eq!(bars_filled(Some(9)), 5);
}
