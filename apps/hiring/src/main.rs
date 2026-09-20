//! `dx serve --package hiring --platform web --port 4403`.

use dioxus::prelude::*;
use hiring::App;

const TAILWIND: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::logger::initialize_default();
    dioxus::launch(Shell);
}

#[component]
fn Shell() -> Element {
    rsx! {
        document::Title { "UK tech hiring register" }
        document::Meta { name: "robots", content: "noindex,nofollow,noarchive" }
        document::Link { rel: "stylesheet", href: TAILWIND }
        App {}
    }
}
