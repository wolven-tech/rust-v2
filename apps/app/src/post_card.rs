//! `PostCard` — the one component that knows about a domain type.
//!
//! It lives here rather than in `rv2-ui` on purpose. Taking a [`PostView`]
//! forced the whole component kit to depend on `rv2-api-types`, which meant
//! every consumer of a `Button` inherited the API's shapes and rebuilt whenever
//! they changed. The kit is now `dioxus`-only, and the single component that
//! needs a domain type sits next to the only view that renders it.

use dioxus::prelude::*;
use rv2_api_types::PostView;
use rv2_ui::Card;

/// A single post in the list. Takes a [`PostView`] — the *same* type the API
/// serves and the folders produce — so there is no view-model to keep in sync.
#[component]
pub fn PostCard(post: PostView, #[props(default)] actions: Option<Element>) -> Element {
    // Formatted outside `rsx!`: the macro's interpolation parser does not
    // accept a call carrying a quoted argument.
    let iso = post.created_at.to_rfc3339();
    let human = post.created_at.format("%Y-%m-%d %H:%M UTC").to_string();
    rsx! {
        Card {
            article {
                h2 { class: "text-lg font-medium text-slate-900", "{post.title}" }
                p { class: "mt-1 whitespace-pre-wrap text-sm text-slate-600", "{post.content}" }
                footer { class: "mt-3 flex items-center justify-between",
                    time { class: "text-xs text-slate-400", datetime: "{iso}", "{human}" }
                    div { class: "flex gap-2", {actions} }
                }
            }
        }
    }
}
