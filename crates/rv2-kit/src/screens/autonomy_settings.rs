//! Autonomy settings: how far an agent may act without asking. One level sets every rule that is not fixed; each
//! rule can be refined by hand, and a fixed set of rules the agent never runs on its own stays as it is.
//!
//! Design choice: a rule is "changed by hand" when its setting differs from what the chosen level implies for its
//! action class. Choosing the current level again re-applies the implied settings and clears those marks.
//! Design choice: the level group is a radio group; each arrow key moves the level and calls `on_preset`, as the spec asks.
//! Design choice: the sidebar is drawn inline so the screen stands alone, without the workspace frame.

use dioxus::prelude::*;

use crate::internal::*;

const CSS: Asset = asset!("/assets/autonomy-settings.css");

/// The three presets, in the order the level group shows them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AutonomyLevel {
    Careful,
    #[default]
    Balanced,
    HandsOff,
}

impl AutonomyLevel {
    pub const ALL: [Self; 3] = [Self::Careful, Self::Balanced, Self::HandsOff];

    pub fn label(self) -> &'static str {
        match self {
            Self::Careful => "Careful",
            Self::Balanced => "Balanced",
            Self::HandsOff => "Hands-off",
        }
    }

    pub fn sentence(self) -> &'static str {
        match self {
            Self::Careful => "Reads run on their own. Anything that changes data asks first.",
            Self::Balanced => "Actions run on their own unless they need care. Those ask first.",
            Self::HandsOff => "Everything runs on its own except the fixed rules.",
        }
    }
}

/// What the agent does with an action without asking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setting {
    Allow,
    Ask,
    Never,
}

impl Setting {
    /// The words a menu and an accessible name use for this setting.
    pub fn label(self) -> &'static str {
        match self {
            Self::Allow => "On its own",
            Self::Ask => "Ask first",
            Self::Never => "Never",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionClass {
    Read,
    Write,
    Risky,
}

impl ActionClass {
    pub fn heading(self) -> &'static str {
        match self {
            Self::Read => "Read only",
            Self::Write => "Edits",
            Self::Risky => "Sensitive",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Done,
    Approved,
    Declined,
    Blocked,
}

impl Outcome {
    pub fn word(self) -> &'static str {
        match self {
            Self::Done => "Done",
            Self::Approved => "Approved",
            Self::Declined => "Declined",
            Self::Blocked => "Blocked",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppEntry {
    pub id: String,
    pub entry_name: String,
    pub account: Option<String>,
    pub mark: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AmountLimit {
    pub amount: f64,
    pub currency: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WeekCounts {
    pub done: u32,
    pub asked: u32,
    pub blocked: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecentEvent {
    pub what: String,
    pub outcome: Outcome,
    pub occurred_at: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActionRule {
    pub id: String,
    pub app_id: String,
    pub action_name: String,
    pub detail_text: Option<String>,
    pub mark: Option<String>,
    pub rule_setting: Setting,
    pub default_setting: Setting,
    pub action_class: ActionClass,
    pub is_fixed: bool,
    pub amount_limit: Option<AmountLimit>,
    pub week_counts: WeekCounts,
    pub recent_events: Vec<RecentEvent>,
}

/// The setting a level implies for an action class. Fixed rules are excluded by the callers; see `is_changed`.
pub fn implied_setting(level: AutonomyLevel, class: ActionClass) -> Setting {
    match (level, class) {
        (AutonomyLevel::Careful, ActionClass::Read) => Setting::Allow,
        (AutonomyLevel::Careful, _) => Setting::Ask,
        (AutonomyLevel::Balanced, ActionClass::Risky) => Setting::Ask,
        (AutonomyLevel::Balanced, _) => Setting::Allow,
        (AutonomyLevel::HandsOff, _) => Setting::Allow,
    }
}

/// True when a non-fixed rule's setting is not what the level implies.
///
/// A level that implies Ask first already holds a Never rule back, and an amount limit is the threshold for
/// that same Ask, so neither is a departure from the level.
pub fn is_changed(rule: &ActionRule, level: AutonomyLevel) -> bool {
    if rule.is_fixed {
        return false;
    }
    let implied = implied_setting(level, rule.action_class);
    if implied == Setting::Ask
        && (rule.rule_setting == Setting::Never || rule.amount_limit.is_some())
    {
        return false;
    }
    rule.rule_setting != implied
}

/// Counts of rules at each setting, as `(on its own, asks first, never)`.
pub fn setting_counts(rules: &[ActionRule]) -> (usize, usize, usize) {
    rules
        .iter()
        .fold((0, 0, 0), |(a, s, n), r| match r.rule_setting {
            Setting::Allow => (a + 1, s, n),
            Setting::Ask => (a, s + 1, n),
            Setting::Never => (a, s, n + 1),
        })
}

/// The app action rules, as `(on its own, asks first, never)`. Fixed rules are counted separately.
pub fn action_counts(rules: &[ActionRule]) -> (usize, usize, usize) {
    let owned: Vec<ActionRule> = rules.iter().filter(|r| !r.is_fixed).cloned().collect();
    setting_counts(&owned)
}

/// The rules an app owns, as `(on its own, asks first, never)`. Fixed rules are counted separately.
pub fn app_counts(rules: &[ActionRule], app_id: &str) -> (usize, usize, usize) {
    let owned: Vec<ActionRule> = rules
        .iter()
        .filter(|r| r.app_id == app_id)
        .cloned()
        .collect();
    action_counts(&owned)
}

/// The first `limit` events of the app's action rules, in rule order. Each rule's `recent_events` must be newest
/// first, because the host has no timestamp it can sort on.
pub fn lately(rules: &[ActionRule], app_id: &str, limit: usize) -> Vec<RecentEvent> {
    rules
        .iter()
        .filter(|r| r.app_id == app_id && !r.is_fixed)
        .flat_map(|r| r.recent_events.iter().cloned())
        .take(limit)
        .collect()
}

/// One app's count sentence, as its accessible name reads it.
pub fn app_sentence(app: &AppEntry, rules: &[ActionRule]) -> String {
    let (allow, ask, never) = app_counts(rules, &app.id);
    let who = match &app.account {
        Some(account) => format!("{}, account: {account}", app.entry_name),
        None => app.entry_name.clone(),
    };
    format!("{who}: {allow} on its own, {ask} asks, {never} never")
}

/// The rules that follow a level: each non-fixed rule takes the setting the level implies.
pub fn apply_level(rules: &[ActionRule], level: AutonomyLevel) -> Vec<ActionRule> {
    rules
        .iter()
        .map(|r| {
            let mut next = r.clone();
            if !r.is_fixed {
                next.rule_setting = implied_setting(level, r.action_class);
            }
            next
        })
        .collect()
}

/// Settings a rule's menu offers: a fixed rule offers only Ask first and Never.
pub fn menu_options(rule: &ActionRule) -> Vec<Setting> {
    if rule.is_fixed {
        vec![Setting::Ask, Setting::Never]
    } else {
        vec![Setting::Allow, Setting::Ask, Setting::Never]
    }
}

/// What an amount limit does, in words, under the limit field.
pub fn limit_sentence(limit: &AmountLimit) -> String {
    format!(
        "Runs on its own up to {} {:.2}. Asks above that amount.",
        limit.currency, limit.amount
    )
}

/// Parses a limit typed in the field. Blank, negative and non-numeric input is refused.
pub fn parse_limit(text: &str) -> Option<f64> {
    let value: f64 = text.trim().parse().ok()?;
    (value.is_finite() && value >= 0.0).then_some(value)
}

/// Share of a total that a count takes, as a percentage; zero when there are no rules.
pub fn share_percent(part: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        part as f64 / total as f64 * 100.0
    }
}

fn initial_of(name: &str) -> String {
    name.chars()
        .next()
        .map(|c| c.to_string())
        .unwrap_or_default()
}

/// Autonomy settings page: level group, summary bar, apps, fixed rules and the open app's actions.
#[component]
pub fn AutonomySettings(
    #[props(default = "Juniper Works".to_string(), into)] workspace_name: String,
    #[props(default = "Ada Lovelace".to_string(), into)] account_name: String,
    #[props(default)] initial_level: AutonomyLevel,
    #[props(default)] app_list: Vec<AppEntry>,
    #[props(default)] action_rules: Vec<ActionRule>,
    #[props(default)] initial_app_id: Option<String>,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] on_change: EventHandler<ActionRule>,
    #[props(default)] on_preset: EventHandler<AutonomyLevel>,
    #[props(default)] on_select: EventHandler<String>,
    #[props(default)] on_connect: EventHandler<()>,
    #[props(default)] on_navigate: EventHandler<String>,
) -> Element {
    let base = use_hook(|| format!("kit-aus-{}", next_id()));
    let level = use_signal(|| initial_level);
    let rules = use_signal(|| action_rules.clone());
    let mut app_id = use_signal(|| {
        initial_app_id
            .clone()
            .or_else(|| app_list.first().map(|a| a.id.clone()))
    });
    let menu_for = use_signal(|| None::<String>);
    let menu_index = use_signal(|| 0_usize);
    let limit_draft = use_signal(|| None::<(String, String)>);
    let announce = use_signal(String::new);

    let current_level = level();
    let rules_now = rules();
    let (allow, ask, never) = action_counts(&rules_now);
    let total = allow + ask + never;
    let pct_allow = share_percent(allow, total);
    let pct_ask = share_percent(ask, total);
    let pct_never = share_percent(never, total);
    let open_app = app_id();

    let fixed_rules: Vec<ActionRule> = rules_now.iter().filter(|r| r.is_fixed).cloned().collect();
    let groups: Vec<(ActionClass, Vec<ActionRule>)> =
        [ActionClass::Read, ActionClass::Write, ActionClass::Risky]
            .into_iter()
            .map(|c| {
                (
                    c,
                    rules_now
                        .iter()
                        .filter(|r| {
                            !r.is_fixed
                                && Some(&r.app_id) == open_app.as_ref()
                                && r.action_class == c
                        })
                        .cloned()
                        .collect(),
                )
            })
            .collect();
    let open_name = app_list
        .iter()
        .find(|a| Some(&a.id) == open_app.as_ref())
        .map(|a| a.entry_name.clone())
        .unwrap_or_default();
    let apps_views: Vec<(AppEntry, String, usize, usize, usize, f64)> = app_list
        .iter()
        .map(|a| {
            let (on, asks, never) = app_counts(&rules_now, &a.id);
            let owned = on + asks + never;
            (
                a.clone(),
                app_sentence(a, &rules_now),
                on,
                asks,
                never,
                share_percent(on, owned),
            )
        })
        .collect();
    let mark_text = |a: &AppEntry| a.mark.clone().unwrap_or_else(|| initial_of(&a.entry_name));
    let recent = open_app
        .as_deref()
        .map(|id| lately(&rules_now, id, 3))
        .unwrap_or_default();

    let on_level_keys = move |event: KeyboardEvent| {
        let index = AutonomyLevel::ALL
            .iter()
            .position(|l| *l == level())
            .unwrap_or(0);
        let count = AutonomyLevel::ALL.len();
        let next = match event.key() {
            Key::ArrowRight | Key::ArrowDown => Some((index + 1) % count),
            Key::ArrowLeft | Key::ArrowUp => Some((index + count - 1) % count),
            _ => None,
        };
        if let Some(i) = next {
            event.prevent_default();
            choose_level(AutonomyLevel::ALL[i], level, rules, announce, on_preset);
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "kit-aus", "aria-labelledby": "{base}-title",
            nav { class: "kit-aus__nav", "aria-label": "Sidebar",
                p { class: "kit-aus__ws", "{workspace_name}" }
                ul { class: "kit-aus__nav-list",
                    for item in ["Home", "Inbox", "Assistant", "Integrations", "Settings"] {
                        li { key: "{item}",
                            button { class: "kit-aus__nav-item", r#type: "button",
                                "aria-current": if item == "Settings" { "page" } else { "false" },
                                onclick: move |_| on_navigate.call(item.to_string()),
                                "{item}"
                            }
                        }
                    }
                }
                p { class: "kit-aus__account", "{account_name}" }
            }

            div { class: "kit-aus__page",
                header { class: "kit-aus__head",
                    h1 { id: "{base}-title", class: "kit-aus__title", "Agent autonomy" }
                    p { class: "kit-aus__lede", "Choose how far an agent may act without asking you." }
                }

                div { class: "kit-aus__levels", role: "radiogroup", "aria-label": "Autonomy level", onkeydown: on_level_keys,
                    for level_option in AutonomyLevel::ALL {
                        button { key: "{level_option.label()}", class: "kit-aus__level", r#type: "button", role: "radio",
                            "aria-checked": if level_option == current_level { "true" } else { "false" },
                            tabindex: if level_option == current_level { "0" } else { "-1" },
                            onclick: move |_| choose_level(level_option, level, rules, announce, on_preset),
                            span { class: "kit-aus__level-name", "{level_option.label()}" }
                            span { class: "kit-aus__level-note", "{level_option.sentence()}" }
                        }
                    }
                }
                p { class: "kit-aus__level-live", "Level: {current_level.label()}" }

                if is_loading {
                    div { class: "kit-aus__summary kit-aus__ph", "aria-busy": "true" }
                } else if !is_empty {
                    section { class: "kit-aus__summary", "aria-label": "Rules by setting",
                        p { class: "kit-aus__counts",
                            span { "{allow} on its own" }
                            span { "{ask} ask first" }
                            span { "{never} never" }
                        }
                        div { class: "kit-aus__bar", "aria-hidden": "true",
                            span { class: "kit-aus__seg", "data-setting": "allow", style: "width: {pct_allow:.1}%;" }
                            span { class: "kit-aus__seg", "data-setting": "ask", style: "width: {pct_ask:.1}%;" }
                            span { class: "kit-aus__seg", "data-setting": "never", style: "width: {pct_never:.1}%;" }
                        }
                    }
                }

                div { class: "kit-aus__columns",
                    section { class: "kit-aus__card", "aria-labelledby": "{base}-apps",
                        h2 { id: "{base}-apps", class: "kit-aus__card-title", "Apps" }
                        if is_loading {
                            for i in 0..3 {
                                span { key: "{i}", class: "kit-aus__ph kit-aus__ph--row" }
                            }
                        } else if is_empty || app_list.is_empty() {
                            div { class: "kit-aus__empty",
                                p { "No apps connected yet." }
                                button { class: "kit-aus__btn kit-aus__btn--primary", r#type: "button", onclick: move |_| on_connect.call(()), "Connect app" }
                            }
                        } else {
                            ul { class: "kit-aus__apps",
                                for (app, sentence, on, asks, never, bar) in apps_views.iter() {
                                    li { key: "{app.id}",
                                        button { class: "kit-aus__app", r#type: "button",
                                            "aria-pressed": if Some(&app.id) == open_app.as_ref() { "true" } else { "false" },
                                            "aria-label": "{sentence}",
                                            onclick: {
                                                let id = app.id.clone();
                                                move |_| {
                                                    app_id.set(Some(id.clone()));
                                                    on_select.call(id.clone());
                                                }
                                            },
                                            span { class: "kit-aus__mark", "aria-hidden": "true", "{mark_text(app)}" }
                                            span { class: "kit-aus__app-body",
                                                span { class: "kit-aus__app-name", "{app.entry_name}" }
                                                if let Some(account) = app.account.clone() {
                                                    span { class: "kit-aus__app-account", "{account}" }
                                                }
                                                span { class: "kit-aus__app-counts", "{on} on its own · {asks} asks · {never} never" }
                                            }
                                            span { class: "kit-aus__app-bar", "aria-hidden": "true",
                                                span { class: "kit-aus__app-seg", style: "width: {bar:.1}%;" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if !is_empty && !is_loading {
                        section { class: "kit-aus__card", "aria-labelledby": "{base}-fixed",
                            h2 { id: "{base}-fixed", class: "kit-aus__card-title", "Fixed rules" }
                            p { class: "kit-aus__note", "The level does not change these rules. Each one either asks first or never runs." }
                            ul { class: "kit-aus__fixed",
                                for rule in fixed_rules.iter() {
                                    li { key: "{rule.id}", class: "kit-aus__fixed-row",
                                        span { class: "kit-aus__fixed-name", "{rule.action_name}" }
                                        SettingMenu {
                                            rule: rule.clone(),
                                            menu_for,
                                            menu_index,
                                            on_pick: move |(rule_id, next): (String, Setting)| {
                                                apply_pick(&rule_id, next, rules, announce, on_change);
                                            },
                                        }
                                        p { class: "kit-aus__why", "Fixed by policy." }
                                    }
                                }
                            }
                        }
                    }

                    if !is_empty && !is_loading {
                        section { class: "kit-aus__card kit-aus__card--wide", "aria-labelledby": "{base}-actions",
                            h2 { id: "{base}-actions", class: "kit-aus__card-title", "Actions in {open_name}" }
                            if open_app.is_none() {
                                p { class: "kit-aus__note", "Choose an app to see its actions." }
                            }
                            for (class, views) in groups.iter() {
                                if !views.is_empty() {
                                    div { key: "{class.heading()}", class: "kit-aus__group",
                                        h3 { class: "kit-aus__group-title", "{class.heading()} · {views.len()}" }
                                        ul { class: "kit-aus__actions",
                                            for rule in views.iter() {
                                                li { key: "{rule.id}", class: "kit-aus__action",
                                                    "data-changed": if is_changed(rule, current_level) { "true" } else { "false" },
                                                    div { class: "kit-aus__action-head",
                                                        if let Some(mark) = rule.mark.clone() {
                                                            span { class: "kit-aus__mark", "aria-hidden": "true", "{mark}" }
                                                        }
                                                        span { class: "kit-aus__action-name", "{rule.action_name}" }
                                                        if is_changed(rule, current_level) {
                                                            span { class: "kit-aus__changed", "Changed by hand" }
                                                        }
                                                        SettingMenu {
                                                            rule: rule.clone(),
                                                            menu_for,
                                                            menu_index,
                                                            on_pick: move |(rule_id, next): (String, Setting)| {
                                                                apply_pick(&rule_id, next, rules, announce, on_change);
                                                            },
                                                        }
                                                    }
                                                    if let Some(detail) = rule.detail_text.clone() {
                                                        p { class: "kit-aus__detail", "{detail}" }
                                                    }
                                                    p { class: "kit-aus__week", "This week: {rule.week_counts.done} done, {rule.week_counts.asked} asked, {rule.week_counts.blocked} blocked" }
                                                    if let Some(limit) = rule.amount_limit.clone() {
                                                        LimitField {
                                                            rule: rule.clone(),
                                                            limit: limit.clone(),
                                                            draft: limit_draft,
                                                            rules,
                                                            announce,
                                                            on_change,
                                                        }
                                                        p { class: "kit-aus__detail", "{limit_sentence(&limit)}" }
                                                    }
                                                    if !rule.recent_events.is_empty() {
                                                        ul { class: "kit-aus__events",
                                                            for (i, event) in rule.recent_events.iter().enumerate() {
                                                                li { key: "{i}", "{event.what}: {event.outcome.word()}, {event.occurred_at}" }
                                                            }
                                                        }
                                                    }
                                                    if is_changed(rule, current_level) {
                                                        button { class: "kit-aus__btn", r#type: "button",
                                                            onclick: {
                                                                let id = rule.id.clone();
                                                                move |_| {
                                                                    let default = rules().iter().find(|r| r.id == id).map(|r| r.default_setting);
                                                                    if let Some(setting) = default {
                                                                        apply_pick(&id, setting, rules, announce, on_change);
                                                                    }
                                                                }
                                                            },
                                                            "Reset"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if !recent.is_empty() {
                                div { class: "kit-aus__group kit-aus__lately",
                                    h3 { class: "kit-aus__group-title", "Lately" }
                                    ul { class: "kit-aus__events",
                                        for (i, event) in recent.iter().enumerate() {
                                            li { key: "{i}", "{event.what}: {event.outcome.word()}, {event.occurred_at}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            p { class: "kit-sr", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}

/// Applies a level to every non-fixed rule, then tells the host which level was chosen.
fn choose_level(
    chosen: AutonomyLevel,
    mut level: Signal<AutonomyLevel>,
    mut rules: Signal<Vec<ActionRule>>,
    mut announce: Signal<String>,
    on_preset: EventHandler<AutonomyLevel>,
) {
    let next = apply_level(&rules(), chosen);
    rules.set(next);
    level.set(chosen);
    announce.set(format!("{} level", chosen.label()));
    on_preset.call(chosen);
}

/// Sets one rule's setting, announces it, and reports the updated rule to the host.
fn apply_pick(
    rule_id: &str,
    next: Setting,
    mut rules: Signal<Vec<ActionRule>>,
    mut announce: Signal<String>,
    on_change: EventHandler<ActionRule>,
) {
    let mut picked = None;
    rules.with_mut(|list| {
        if let Some(r) = list.iter_mut().find(|r| r.id == rule_id) {
            r.rule_setting = next;
            picked = Some(r.clone());
        }
    });
    if let Some(rule) = picked {
        announce.set(format!("{}: {}", rule.action_name, next.label()));
        on_change.call(rule);
    }
}

/// A rule's amount limit. Enter saves a valid amount and reports it; Escape drops the draft.
#[component]
fn LimitField(
    rule: ActionRule,
    limit: AmountLimit,
    mut draft: Signal<Option<(String, String)>>,
    mut rules: Signal<Vec<ActionRule>>,
    mut announce: Signal<String>,
    on_change: EventHandler<ActionRule>,
) -> Element {
    let shown = match draft() {
        Some((id, text)) if id == rule.id => text,
        _ => format!("{:.2}", limit.amount),
    };
    let rule_id = rule.id.clone();
    let currency = limit.currency.clone();
    rsx! {
        label { class: "kit-aus__limit",
            span { "Ask above {currency}" }
            input { class: "kit-aus__input", r#type: "text", inputmode: "decimal", value: "{shown}",
                oninput: {
                    let rule_id = rule_id.clone();
                    move |e: FormEvent| draft.set(Some((rule_id.clone(), e.value())))
                },
                onkeydown: move |event: KeyboardEvent| {
                    match event.key() {
                        Key::Enter => {
                            event.prevent_default();
                            let text = draft().map(|(_, d)| d).unwrap_or_default();
                            if let Some(value) = parse_limit(&text) {
                                let mut updated = rule.clone();
                                updated.amount_limit = Some(AmountLimit { amount: value, currency: currency.clone() });
                                rules.with_mut(|list| {
                                    if let Some(r) = list.iter_mut().find(|r| r.id == updated.id) {
                                        r.amount_limit = updated.amount_limit.clone();
                                    }
                                });
                                draft.set(None);
                                announce.set(format!("{} limit saved", updated.action_name));
                                on_change.call(updated);
                            }
                        }
                        Key::Escape => {
                            event.prevent_default();
                            draft.set(None);
                        }
                        _ => {}
                    }
                },
            }
        }
    }
}

/// A rule's setting menu. Enter or Space opens it, arrows move, Enter picks, Escape closes and returns focus.
#[component]
fn SettingMenu(
    rule: ActionRule,
    mut menu_for: Signal<Option<String>>,
    mut menu_index: Signal<usize>,
    on_pick: EventHandler<(String, Setting)>,
) -> Element {
    let id = use_hook(|| format!("kit-aus-menu-{}", next_id()));
    let trigger = format!("{id}-trigger");
    let list_id = format!("{id}-list");
    let options = menu_options(&rule);
    let open = menu_for() == Some(rule.id.clone());
    let current = rule.rule_setting;
    let rule_id = rule.id.clone();
    let rule_name = rule.action_name.clone();
    let option_count = options.len();
    let option_list = options.clone();
    let rule_for_pick = rule_id.clone();
    let trigger_for_keys = trigger.clone();
    let list_for_open = list_id.clone();
    let rule_for_open = rule_id.clone();
    rsx! {
        span { class: "kit-aus__menu-wrap",
            button { id: "{trigger}", class: "kit-aus__btn kit-aus__setting", r#type: "button", "aria-haspopup": "menu",
                "aria-expanded": "{open}", "aria-label": "{rule_name}: {current.label()}",
                onclick: move |_| {
                    if open {
                        menu_for.set(None);
                    } else {
                        let at = options.iter().position(|o| *o == current).unwrap_or(0);
                        menu_index.set(at);
                        menu_for.set(Some(rule_for_open.clone()));
                        focus_after_frame(&list_for_open);
                    }
                },
                "{current.label()}"
            }
            if open {
                ul { id: "{list_id}", class: "kit-aus__menu", role: "menu", tabindex: "-1",
                    onkeydown: move |event: KeyboardEvent| {
                        let last = option_count.saturating_sub(1);
                        match event.key() {
                            Key::ArrowDown => {
                                event.prevent_default();
                                menu_index.set((menu_index() + 1).min(last));
                            }
                            Key::ArrowUp => {
                                event.prevent_default();
                                menu_index.set(menu_index().saturating_sub(1));
                            }
                            Key::Enter => {
                                event.prevent_default();
                                if let Some(next) = option_list.get(menu_index()) {
                                    on_pick.call((rule_for_pick.clone(), *next));
                                }
                                menu_for.set(None);
                                focus_after_frame(&trigger_for_keys);
                            }
                            Key::Escape => {
                                event.prevent_default();
                                menu_for.set(None);
                                focus_after_frame(&trigger_for_keys);
                            }
                            _ => {}
                        }
                    },
                    for (i, option) in menu_options(&rule).into_iter().enumerate() {
                        li { key: "{i}", role: "menuitem", tabindex: "-1",
                            class: if i == menu_index() { "kit-aus__opt is-active" } else { "kit-aus__opt" },
                            onclick: {
                                let rule_id = rule_id.clone();
                                move |_| {
                                    on_pick.call((rule_id.clone(), option));
                                    menu_for.set(None);
                                }
                            },
                            "{option.label()}"
                        }
                    }
                }
            }
        }
    }
}
