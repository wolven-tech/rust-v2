//! `dx serve --package app --platform web --port 4402`.

use app::Route;
use dioxus::prelude::*;

const TAILWIND: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::logger::initialize_default();
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND }
        Router::<Route> {}
    }
}
