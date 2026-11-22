//! Points affectUI's `--aui-*` hooks at the app's rv2 registers.

use dioxus::prelude::*;

const BRIDGE: Asset = asset!("/assets/rv2-bridge.css");

/// Render once near the root of an app that themes rv2's registers, so these
/// components take the app's colours instead of affectUI's defaults.
#[component]
pub fn Rv2Theme() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: BRIDGE }
    }
}
