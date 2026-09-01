//! `dx serve --package app --platform web --port 4402`.

use app::{PUBLIC_PRODUCT_NAME, Route};
use dioxus::prelude::*;

const TAILWIND: Asset = asset!("/assets/tailwind.css");
const FAVICON: Asset = asset!("/assets/favicon.svg");

fn main() {
    dioxus::logger::initialize_default();
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Title { "{PUBLIC_PRODUCT_NAME} — app" }
        document::Meta { name: "robots", content: "noindex,nofollow,noarchive" }
        document::Link { rel: "icon", r#type: "image/svg+xml", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND }
        Router::<Route> {}
    }
}
