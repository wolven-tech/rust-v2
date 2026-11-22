//! SSR checks for the inline edit review: a removed and an inserted token that
//! both show are separated by a space in the markup.

use dioxus::prelude::*;
use rv2_kit::components::inline_edit_review::InlineEditReview;

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn rewrite() -> Element {
    rsx! {
        InlineEditReview {
            before_text: "The team will ship the update next week.",
            after_text: "The team will ships the update next week.",
        }
    }
}

#[test]
fn struck_and_inserted_tokens_that_both_show_have_a_space_between_them() {
    let html = render(rewrite);
    assert!(html.contains("kit-ier__removed"), "a removed token renders");
    assert!(
        html.contains("kit-ier__inserted"),
        "an inserted token renders"
    );
    assert!(
        html.contains("</del> <ins"),
        "struck text and inserted text are separated by a space"
    );
}
