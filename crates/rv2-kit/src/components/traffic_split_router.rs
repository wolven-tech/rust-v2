//! Traffic split router: how an AI gateway spreads requests across models, with cost, latency and a fallback.
//!
//! Design choice: `on_deploy` is fire-and-forget. The caller reports progress through the `deploying` prop, which
//! locks the controls and reads "Deploying" while true.
//! Design choice: `share_map` is a list of `(route_id, percent)` pairs, so the data compares by value.
//! Design choice: blended cost is the share-weighted mean, `sum(share / 100 * cost_per_1k)`, in the same dollars as `cost_per_1k`.
//! Browser checks: pulse speed following share, band slide on strategy change, number roll, hover highlight.

use dioxus::prelude::*;

use crate::internal::*;

const CSS: Asset = asset!("/assets/traffic-split-router.css");

#[derive(Clone, PartialEq, Debug)]
pub struct Route {
    pub route_id: String,
    pub route_name: String,
    pub route_provider: Option<String>,
    pub route_mark: Option<String>,
    pub route_tone: Option<String>,
    /// 95th-percentile latency in seconds.
    pub p95_seconds: f64,
    /// Dollars per thousand tokens.
    pub cost_per_1k: f64,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Strategy {
    pub strategy_id: String,
    pub strategy_label: String,
    /// `(route_id, percent)`. Empty divides traffic evenly.
    pub share_map: Vec<(String, f64)>,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Fallback {
    pub enabled: bool,
    pub from_route_id: Option<String>,
    pub to_route_id: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Policy {
    pub active_strategy: String,
    pub fallback: Fallback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyState {
    Live,
    Unsaved,
}

impl PolicyState {
    pub fn word(self) -> &'static str {
        match self {
            Self::Live => "Live",
            Self::Unsaved => "Unsaved changes",
        }
    }
}

/// Shares for a strategy over the given routes, in route order, normalised to 100.
pub fn normalise_shares(strategy: &Strategy, route_ids: &[String]) -> Vec<(String, f64)> {
    let even = 100.0 / route_ids.len().max(1) as f64;
    if strategy.share_map.is_empty() {
        return route_ids.iter().map(|id| (id.clone(), even)).collect();
    }
    let raw: Vec<(String, f64)> = route_ids
        .iter()
        .map(|id| {
            let v = strategy
                .share_map
                .iter()
                .find(|(r, _)| r == id)
                .map(|(_, v)| v.max(0.0))
                .unwrap_or(0.0);
            (id.clone(), v)
        })
        .collect();
    let total: f64 = raw.iter().map(|(_, v)| v).sum();
    if total <= 0.0 {
        return route_ids.iter().map(|id| (id.clone(), even)).collect();
    }
    raw.into_iter()
        .map(|(id, v)| (id, v / total * 100.0))
        .collect()
}

/// Share-weighted cost per thousand tokens for a set of shares.
pub fn blended_cost(shares: &[(String, f64)], routes: &[Route]) -> f64 {
    shares
        .iter()
        .map(|(id, share)| {
            let cost = routes
                .iter()
                .find(|r| r.route_id == *id)
                .map(|r| r.cost_per_1k)
                .unwrap_or(0.0);
            share / 100.0 * cost
        })
        .sum()
}

/// The fallback pair that will actually be used: a missing or identical target moves to the next route.
pub fn resolve_fallback(fallback: &Fallback, routes: &[Route]) -> Option<(String, String)> {
    if !fallback.enabled || routes.len() < 2 {
        return None;
    }
    let has = |id: &Option<String>| {
        id.as_ref()
            .filter(|i| routes.iter().any(|r| &r.route_id == *i))
            .cloned()
    };
    let from = has(&fallback.from_route_id).unwrap_or_else(|| routes[0].route_id.clone());
    let to = has(&fallback.to_route_id)
        .filter(|t| *t != from)
        .or_else(|| {
            routes
                .iter()
                .map(|r| r.route_id.clone())
                .find(|id| *id != from)
        })?;
    Some((from, to))
}

/// Live when the draft equals the live policy, else unsaved.
pub fn policy_state(draft: &Policy, live: &Policy) -> PolicyState {
    if draft == live {
        PolicyState::Live
    } else {
        PolicyState::Unsaved
    }
}

/// Gateway route split with a strategy choice, a fallback and a deploy control.
#[component]
pub fn TrafficSplitRouter(
    route_list: Vec<Route>,
    strategy_list: Vec<Strategy>,
    live_policy: Policy,
    #[props(default)] policy_revision: u32,
    #[props(default, into)] environment_label: String,
    #[props(default)] on_deploy: Option<EventHandler<Policy>>,
    #[props(default)] on_draft_change: Option<EventHandler<Policy>>,
    #[props(default)] deploying: bool,
    #[props(default = "Traffic split".to_string(), into)] heading: String,
    #[props(default)] extra_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-tsr-{}", next_id()));
    let mut draft = use_signal(|| live_policy.clone());
    let route_ids: Vec<String> = route_list.iter().map(|r| r.route_id.clone()).collect();
    let draft_policy = draft();
    let state = policy_state(&draft_policy, &live_policy);
    let dirty = state == PolicyState::Unsaved;
    let class = class_list("kit-tsr", &extra_class);

    if route_list.is_empty() {
        return rsx! {
            document::Link { rel: "stylesheet", href: KIT_CSS }
            document::Link { rel: "stylesheet", href: CSS }
            section { class: "{class}", "aria-label": "{heading}", p { class: "kit-tsr__empty", "No models to route to" } }
        };
    }

    let strategy = strategy_list
        .iter()
        .find(|s| s.strategy_id == draft_policy.active_strategy)
        .cloned()
        .or_else(|| strategy_list.first().cloned());
    let shares = strategy
        .as_ref()
        .map(|s| normalise_shares(s, &route_ids))
        .unwrap_or_else(|| {
            route_ids
                .iter()
                .map(|id| (id.clone(), 100.0 / route_ids.len() as f64))
                .collect()
        });
    let blended = blended_cost(&shares, &route_list);
    let live_shares = strategy_list
        .iter()
        .find(|s| s.strategy_id == live_policy.active_strategy)
        .map(|s| normalise_shares(s, &route_ids))
        .unwrap_or_default();
    let live_blended = blended_cost(&live_shares, &route_list);
    let resolved = resolve_fallback(&draft_policy.fallback, &route_list);
    let show_fallback = route_list.len() > 1;
    let deploy_label = if deploying { "Deploying" } else { "Deploy" };
    let state_text = if deploying { "Deploying" } else { state.word() };
    let deploy_disabled = !dirty || deploying;
    let revision_label = format!("Policy v{policy_revision}");
    let summary = format!("${blended:.4} per 1k tokens · live ${live_blended:.4}");

    let commit = use_callback(move |next: Policy| {
        draft.set(next.clone());
        if let Some(h) = on_draft_change {
            h.call(next);
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{class}", "aria-labelledby": "{base}-title", "data-state": if dirty { "unsaved" } else { "live" },
            header { class: "kit-tsr__head",
                div {
                    h2 { id: "{base}-title", class: "kit-tsr__title", "{heading}" }
                    p { class: "kit-tsr__env", "{environment_label} · {revision_label}" }
                }
                p { class: "kit-tsr__summary", "{summary}" }
            }
            div { class: "kit-tsr__strategies", role: "radiogroup", "aria-label": "Strategy",
                for (i, s) in strategy_list.iter().enumerate() {
                    {
                        let checked = strategy.as_ref().map(|c| c.strategy_id == s.strategy_id).unwrap_or(false);
                        let id = s.strategy_id.clone();
                        let label = s.strategy_label.clone();
                        let dom_id = format!("{base}-s-{i}");
                        let ids = strategy_list.iter().map(|x| x.strategy_id.clone()).collect::<Vec<_>>();
                        let fallback = draft_policy.fallback.clone();
                        rsx! {
                            button {
                                id: "{dom_id}", r#type: "button", role: "radio", class: "kit-tsr__strategy",
                                "aria-checked": "{checked}", tabindex: if checked { "0" } else { "-1" },
                                disabled: deploying,
                                onclick: {
                                    let fallback = fallback.clone();
                                    move |_| commit.call(Policy { active_strategy: id.clone(), fallback: fallback.clone() })
                                },
                                onkeydown: {
                                    let ids = ids.clone();
                                    let fb = fallback.clone();
                                    let base = base.clone();
                                    let active = draft_policy.active_strategy.clone();
                                    move |event| {
                                        let len = ids.len();
                                        let current = ids.iter().position(|x| *x == active).unwrap_or(0);
                                        let next = match event.key() {
                                            Key::ArrowRight | Key::ArrowDown => Some((current + 1) % len),
                                            Key::ArrowLeft | Key::ArrowUp => Some((current + len - 1) % len),
                                            _ => None,
                                        };
                                        if let Some(n) = next {
                                            event.prevent_default();
                                            commit.call(Policy { active_strategy: ids[n].clone(), fallback: fb.clone() });
                                            document::eval(&format!("document.getElementById('{base}-s-{n}')?.focus();"));
                                        }
                                    }
                                },
                                "{label}"
                            }
                        }
                    }
                }
            }
            figure { class: "kit-tsr__flow", "aria-hidden": "true",
                div { class: "kit-tsr__gateway", "Gateway" }
                for (i, (id, share)) in shares.iter().enumerate() {
                    {
                        let name = route_list.iter().find(|r| r.route_id == *id).map(|r| r.route_name.clone()).unwrap_or_default();
                        let width = (share * 0.12).clamp(1.0, 14.0);
                        let style = format!("--tsr-share: {share:.2}%; --tsr-width: {width:.1}px; --tsr-delay: {}ms;", i * 180);
                        rsx! {
                            div { key: "{id}", class: "kit-tsr__band", style: "{style}",
                                span { class: "kit-tsr__pulse" }
                                span { class: "kit-tsr__band-name", "{name}" }
                            }
                        }
                    }
                }
            }
            table { class: "kit-tsr__table", "aria-label": "Routes",
                thead {
                    tr {
                        th { scope: "col", "Route" }
                        th { scope: "col", "Share" }
                        th { scope: "col", "p95" }
                        th { scope: "col", "Cost / 1k" }
                    }
                }
                tbody {
                    for (id, share) in shares.iter() {
                        {
                            let route = route_list.iter().find(|r| r.route_id == *id).cloned();
                            let name = route.as_ref().map(|r| r.route_name.clone()).unwrap_or_default();
                            let p95 = route.as_ref().map(|r| format!("{:.1}s", r.p95_seconds)).unwrap_or_default();
                            let cost = route.as_ref().map(|r| format!("${:.4}", r.cost_per_1k)).unwrap_or_default();
                            let share_text = format!("{share:.1}%");
                            rsx! {
                                tr { key: "{id}",
                                    th { scope: "row", "{name}" }
                                    td { "{share_text}" }
                                    td { "{p95}" }
                                    td { "{cost}" }
                                }
                            }
                        }
                    }
                }
            }
            if show_fallback {
                div { class: "kit-tsr__fallback",
                    button {
                        r#type: "button", role: "switch", "aria-checked": "{draft_policy.fallback.enabled}",
                        class: "kit-tsr__switch", "data-on": draft_policy.fallback.enabled.then_some("true"),
                        disabled: deploying,
                        onclick: {
                            let p = draft_policy.clone();
                            move |_| {
                                let mut next = p.clone();
                                next.fallback.enabled = !next.fallback.enabled;
                                commit.call(next);
                            }
                        },
                        "Fallback"
                    }
                    label { class: "kit-tsr__label", "From",
                        select {
                            class: "kit-tsr__select", disabled: !draft_policy.fallback.enabled || deploying,
                            onchange: {
                                let p = draft_policy.clone();
                                move |event: FormEvent| {
                                    let mut next = p.clone();
                                    next.fallback.from_route_id = Some(event.value());
                                    commit.call(next);
                                }
                            },
                            for id in route_ids.iter() {
                                option { key: "{id}", value: "{id}", selected: resolved.as_ref().map(|(f, _)| f == id).unwrap_or(false), "{id}" }
                            }
                        }
                    }
                    label { class: "kit-tsr__label", "To",
                        select {
                            class: "kit-tsr__select", disabled: !draft_policy.fallback.enabled || deploying,
                            onchange: {
                                let p = draft_policy.clone();
                                move |event: FormEvent| {
                                    let mut next = p.clone();
                                    next.fallback.to_route_id = Some(event.value());
                                    commit.call(next);
                                }
                            },
                            for id in route_ids.iter() {
                                option { key: "{id}", value: "{id}", selected: resolved.as_ref().map(|(_, t)| t == id).unwrap_or(false), "{id}" }
                            }
                        }
                    }
                }
            }
            footer { class: "kit-tsr__foot", role: "status", "aria-live": "polite",
                span { class: "kit-tsr__state", "{state_text}" }
                button {
                    r#type: "button", class: "kit-tsr__revert", disabled: !dirty || deploying,
                    onclick: move |_| commit.call(live_policy.clone()),
                    "Revert"
                }
                button {
                    r#type: "button", class: "kit-tsr__deploy", "aria-disabled": "{deploy_disabled}", disabled: deploy_disabled,
                    onclick: move |_| {
                        if let Some(h) = on_deploy {
                            h.call(draft());
                        }
                    },
                    "{deploy_label}"
                }
            }
        }
    }
}
