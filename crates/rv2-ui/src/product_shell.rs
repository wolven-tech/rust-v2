//! Product-shell patterns shared by public sites and browser applications.
//!
//! These components own navigation and progress semantics, responsive
//! disclosure, stable target sizes, and motion fallbacks. Product identity,
//! route enums, copy, legal text, and artwork remain with each product.

use dioxus::prelude::*;

/// A link in the kit's navigation: a header route, a rail entry, a footer
/// link, a header call-to-action.
///
/// One record for all of them. There used to be three — `NavItem`,
/// `ProductNavItem` and `ProductHeaderAction` — each carrying the same label,
/// href and external flag with its own constructors, so a footer link and a
/// header link could not be the same value.
///
/// `key` is the stable route identity that `current` is matched against, so a
/// label or URL can change without breaking current-page state. A link that is
/// never "the current page" — a footer link, a call-to-action — has no key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavLink {
    pub key: Option<String>,
    pub label: String,
    pub href: String,
    /// Opens in a new tab with `rel="noopener noreferrer"`.
    pub external: bool,
}

impl NavLink {
    /// A link that is never the current page.
    pub fn new(label: impl Into<String>, href: impl Into<String>) -> Self {
        Self {
            key: None,
            label: label.into(),
            href: href.into(),
            external: false,
        }
    }

    /// A route, matched against `current` by `key`.
    pub fn route(
        key: impl Into<String>,
        label: impl Into<String>,
        href: impl Into<String>,
    ) -> Self {
        Self {
            key: Some(key.into()),
            ..Self::new(label, href)
        }
    }

    /// The same link, leaving the site.
    #[must_use]
    pub fn external(self) -> Self {
        Self {
            external: true,
            ..self
        }
    }
}

fn is_current(item: &NavLink, current: Option<&str>) -> bool {
    item.key.is_some() && item.key.as_deref() == current
}

fn render_header_action(action: &NavLink) -> Element {
    rsx! {
        a {
            class: "rv2-product-header__action",
            href: "{action.href}",
            target: action.external.then_some("_blank"),
            rel: action.external.then_some("noopener noreferrer"),
            "{action.label}"
        }
    }
}

/// Stable product header with route state and a native mobile disclosure.
///
/// Navigation uses plain URLs to keep this crate router-neutral. Browser apps
/// may use these links as deliberate full navigations or keep router-specific
/// navigation outside this component.
#[component]
pub fn ProductHeader(
    brand: String,
    #[props(default = "/".to_string())] brand_href: String,
    #[props(default)] brand_subtitle: Option<String>,
    #[props(default)] brand_mark: Option<Element>,
    #[props(default)] items: Vec<NavLink>,
    #[props(default)] current: Option<String>,
    #[props(default)] action: Option<NavLink>,
    #[props(default)] tail: Option<Element>,
    #[props(default = "Menu".to_string())] menu_label: String,
    #[props(default)] class: String,
) -> Element {
    let current = current.as_deref();

    rsx! {
        header { class: "rv2-product-header {class}",
            div { class: "rv2-product-header__inner",
                a {
                    class: "rv2-product-header__brand",
                    href: "{brand_href}",
                    aria_label: "{brand} home",
                    if let Some(mark) = brand_mark {
                        span { class: "rv2-product-header__mark", {mark} }
                    }
                    span { class: "rv2-product-header__identity",
                        strong { "{brand}" }
                        if let Some(subtitle) = brand_subtitle {
                            small { "{subtitle}" }
                        }
                    }
                }

                nav { class: "rv2-product-header__desktop", aria_label: "Main navigation",
                    ul {
                        for item in items.iter() {
                            li {
                                a {
                                    href: "{item.href}",
                                    aria_current: is_current(item, current).then_some("page"),
                                    target: item.external.then_some("_blank"),
                                    rel: item.external.then_some("noopener noreferrer"),
                                    "{item.label}"
                                }
                            }
                        }
                    }
                }

                if let Some(action) = action.as_ref() {
                    div { class: "rv2-product-header__desktop-action", {render_header_action(action)} }
                }
                if let Some(tail) = tail {
                    div { class: "rv2-product-header__tail", {tail} }
                }

                details { class: "rv2-product-header__mobile",
                    summary { "{menu_label}" }
                    nav { aria_label: "Mobile navigation",
                        ul {
                            for item in items.iter() {
                                li {
                                    a {
                                        href: "{item.href}",
                                        aria_current: is_current(item, current).then_some("page"),
                                        target: item.external.then_some("_blank"),
                                        rel: item.external.then_some("noopener noreferrer"),
                                        "{item.label}"
                                    }
                                }
                            }
                            if let Some(action) = action.as_ref() {
                                li { class: "rv2-product-header__mobile-action", {render_header_action(action)} }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Compact route, policy, or article index.
#[component]
pub fn NavigationRail(
    label: String,
    #[props(default)] items: Vec<NavLink>,
    #[props(default)] current: Option<String>,
    #[props(default)] class: String,
) -> Element {
    let current = current.as_deref();
    rsx! {
        nav { class: "rv2-navigation-rail {class}", aria_label: "{label}",
            ul {
                for item in items.iter() {
                    li {
                        a {
                            href: "{item.href}",
                            aria_current: is_current(item, current).then_some("page"),
                            target: item.external.then_some("_blank"),
                            rel: item.external.then_some("noopener noreferrer"),
                            "{item.label}"
                        }
                    }
                }
            }
        }
    }
}

/// One stage in a [`ProcessRail`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessStep {
    pub label: String,
    pub hint: String,
    pub href: Option<String>,
}

impl ProcessStep {
    pub fn new(label: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            hint: hint.into(),
            href: None,
        }
    }

    pub fn linked(
        label: impl Into<String>,
        hint: impl Into<String>,
        href: impl Into<String>,
    ) -> Self {
        Self {
            label: label.into(),
            hint: hint.into(),
            href: Some(href.into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProcessState {
    Complete,
    Current,
    Upcoming,
}

fn process_state(index: usize, active_step: Option<usize>, count: usize) -> ProcessState {
    let active = active_step.filter(|step| (1..=count).contains(step));
    match active {
        Some(active) if index + 1 < active => ProcessState::Complete,
        Some(active) if index + 1 == active => ProcessState::Current,
        _ => ProcessState::Upcoming,
    }
}

impl ProcessState {
    const fn class(self) -> &'static str {
        match self {
            Self::Complete => "is-complete",
            Self::Current => "is-current",
            Self::Upcoming => "is-upcoming",
        }
    }
}

/// Ordered process with optional current-step semantics.
///
/// `active_step` is one-based. Omit it for a static process explanation.
#[component]
pub fn ProcessRail(
    label: String,
    steps: Vec<ProcessStep>,
    #[props(default)] active_step: Option<usize>,
    #[props(default)] class: String,
) -> Element {
    let count = steps.len();
    let column_style = format!("--rv2-process-columns: {};", count.max(1));
    rsx! {
        nav { class: "rv2-process-rail {class}", aria_label: "{label}", style: "{column_style}",
            ol {
                for (index, step) in steps.iter().enumerate() {
                    ProcessRailItem {
                        step: step.clone(),
                        index,
                        active_step,
                        count,
                    }
                }
            }
        }
    }
}

#[component]
fn ProcessRailItem(
    step: ProcessStep,
    index: usize,
    active_step: Option<usize>,
    count: usize,
) -> Element {
    let state = process_state(index, active_step, count);
    let number = format!("{:02}", index + 1);
    let current = (state == ProcessState::Current).then_some("step");

    rsx! {
        li { class: "rv2-process-step {state.class()}",
            if let Some(href) = step.href.as_ref() {
                a { href: "{href}", aria_current: current,
                    span { "{number}" }
                    strong { "{step.label}" }
                    small { "{step.hint}" }
                }
            } else {
                div { aria_current: current,
                    span { "{number}" }
                    strong { "{step.label}" }
                    small { "{step.hint}" }
                }
            }
        }
    }
}

/// Opt-in finite page entrance. Reduced-motion users receive no animation.
#[component]
pub fn PageEntrance(#[props(default)] class: String, children: Element) -> Element {
    rsx! { div { class: "rv2-page-entrance {class}", {children} } }
}

#[cfg(test)]
mod tests {
    use super::{NavLink, ProcessState, is_current, process_state};

    #[test]
    fn route_state_matches_stable_key_not_label_or_href() {
        let item = NavLink::route("guide", "Owner guide", "/guide");
        assert!(is_current(&item, Some("guide")));
        assert!(!is_current(&item, Some("Owner guide")));
        assert!(!is_current(&item, Some("/guide")));
    }

    /// A footer link or call-to-action has no route identity, so nothing —
    /// not even an empty `current` — can mark it as the page you are on.
    #[test]
    fn a_link_without_a_key_is_never_the_current_page() {
        let item = NavLink::new("Pricing", "/#pricing");
        assert!(!is_current(&item, None));
        assert!(!is_current(&item, Some("")));
        assert!(!is_current(&item, Some("Pricing")));
    }

    #[test]
    fn external_keeps_the_key_label_and_href() {
        let item = NavLink::route("docs", "Docs", "https://example.com").external();
        assert_eq!(item.key.as_deref(), Some("docs"));
        assert_eq!(item.label, "Docs");
        assert!(item.external);
    }

    #[test]
    fn process_state_marks_only_prior_steps_complete() {
        assert_eq!(process_state(0, Some(3), 4), ProcessState::Complete);
        assert_eq!(process_state(1, Some(3), 4), ProcessState::Complete);
        assert_eq!(process_state(2, Some(3), 4), ProcessState::Current);
        assert_eq!(process_state(3, Some(3), 4), ProcessState::Upcoming);
    }

    #[test]
    fn invalid_or_missing_active_step_keeps_process_static() {
        assert_eq!(process_state(0, None, 4), ProcessState::Upcoming);
        assert_eq!(process_state(0, Some(0), 4), ProcessState::Upcoming);
        assert_eq!(process_state(0, Some(5), 4), ProcessState::Upcoming);
    }
}
