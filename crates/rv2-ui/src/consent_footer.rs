//! Portable consent presentation; product adapters own persistence and tracking.

use dioxus::prelude::*;

pub const CONSENT_FOOTER_SCRIPT: Asset = asset!("/assets/consent-footer.js");
pub const CONSENT_FOOTER_STYLESHEET: Asset = asset!("/assets/consent-footer.css");
pub const CONSENT_FOOTER_JS: &str = include_str!("../assets/consent-footer.js");
pub const CONSENT_FOOTER_CSS: &str = include_str!("../assets/consent-footer.css");

/// Loads the shared browser renderer for SSG and hydrated product adapters.
/// Mount after `rv2-consent-ready`, or immediately when `window.Rv2ConsentFooter` exists.
#[component]
pub fn ConsentFooter() -> Element {
    rsx! {
        document::Stylesheet { href: CONSENT_FOOTER_STYLESHEET }
        document::Script { src: CONSENT_FOOTER_SCRIPT }
    }
}
