//! Settings: Members and Teams. Seat counts, finding and changing people, bulk
//! changes, invites, export, and grouping members into teams.
//!
//! Design choice: an invite is sent by one Enter. The field takes one or more
//! addresses separated by commas or spaces, and each becomes a chip on the same
//! press that sends them, so "two addresses with Enter" is one `on_invite` call.
//! Design choice: each row has an actions menu (remove, or resend and cancel for an
//! invite); removing opens an alert dialog before `on_remove` is called.
//! Design choice: the header checkbox's partial state is set on the DOM through
//! `indeterminate` and exposed to assistive technology as the same mixed state.
//! Browser checks: bulk bar wording, shift-click range, partial header state,
//! invite on Enter, Escape in the invite form, remove confirmation, phone width.

use dioxus::prelude::*;
use rv2_affect::dialog::AlertDialog;

use super::settings_profile::{
    SettingsCard, SettingsShell, SettingsSkel, SettingsStatus, focus_later,
};
use crate::internal::{focus_after_frame, next_id};

const MEMBERS_CSS: Asset = asset!("/assets/settings-members-teams.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MemberRole {
    Admin,
    #[default]
    Member,
    Viewer,
}

impl MemberRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Member => "member",
            Self::Viewer => "viewer",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Admin => "Admin",
            Self::Member => "Member",
            Self::Viewer => "Viewer",
        }
    }

    pub const ALL: [MemberRole; 3] = [Self::Admin, Self::Member, Self::Viewer];
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RoleFilter {
    #[default]
    All,
    Admin,
    Member,
    Viewer,
}

impl RoleFilter {
    fn role(self) -> Option<MemberRole> {
        match self {
            Self::All => None,
            Self::Admin => Some(MemberRole::Admin),
            Self::Member => Some(MemberRole::Member),
            Self::Viewer => Some(MemberRole::Viewer),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MembersTab {
    #[default]
    Members,
    Teams,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BulkAction {
    Role,
    Remove,
    Plan,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Member {
    pub id: String,
    pub display_name: Option<String>,
    pub email_address: String,
    pub member_role: MemberRole,
    pub plan_name: String,
    pub usage_ratio: f64,
    pub is_invited: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Team {
    pub id: String,
    pub display_name: String,
    pub member_ids: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Seats {
    pub used: u32,
    pub total: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeaderState {
    None,
    Some,
    All,
}

/// The name a row is shown and labelled with: display name, else the email.
pub fn member_name(member: &Member) -> String {
    member
        .display_name
        .clone()
        .unwrap_or_else(|| member.email_address.clone())
}

/// Members matching the role filter and, when non-empty, the query across name
/// and email. Order is kept.
pub fn filter_members(members: &[Member], role: RoleFilter, query: &str) -> Vec<Member> {
    let needle = query.trim().to_lowercase();
    members
        .iter()
        .filter(|m| role.role().is_none_or(|r| r == m.member_role))
        .filter(|m| {
            needle.is_empty()
                || member_name(m).to_lowercase().contains(&needle)
                || m.email_address.to_lowercase().contains(&needle)
        })
        .cloned()
        .collect()
}

/// Ids from `anchor` to `target` inclusive, in list order, whichever comes first.
pub fn range_between(order: &[String], anchor: &str, target: &str) -> Vec<String> {
    let (Some(a), Some(b)) = (
        order.iter().position(|id| id == anchor),
        order.iter().position(|id| id == target),
    ) else {
        return vec![target.to_string()];
    };
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    order[lo..=hi].to_vec()
}

/// `selected` out of `visible`, as the header checkbox state.
pub fn header_state(selected: usize, visible: usize) -> HeaderState {
    if visible == 0 || selected == 0 {
        HeaderState::None
    } else if selected >= visible {
        HeaderState::All
    } else {
        HeaderState::Some
    }
}

/// Count of selected rows as words for the bulk bar, up to ten.
pub fn count_words(n: usize) -> String {
    const WORDS: [&str; 11] = [
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
    ];
    WORDS
        .get(n)
        .map(|w| w.to_string())
        .unwrap_or_else(|| n.to_string())
}

/// Usage as a whole percentage for the meter's text.
pub fn usage_percent(ratio: f64) -> u32 {
    (ratio.clamp(0.0, 1.0) * 100.0).round() as u32
}

/// Members and teams settings page.
#[component]
pub fn SettingsMembersTeams(
    member_list: Vec<Member>,
    team_list: Vec<Team>,
    seats: Seats,
    current_user_id: String,
    #[props(default)] role_filter: RoleFilter,
    #[props(default)] query: String,
    #[props(default)] default_tab: MembersTab,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
    #[props(default)] docs_href: Option<String>,
    #[props(default)] on_invite: Option<EventHandler<(Vec<String>, MemberRole)>>,
    #[props(default)] on_change_role: Option<EventHandler<(String, MemberRole)>>,
    #[props(default)] on_remove: Option<EventHandler<String>>,
    #[props(default)] on_bulk: Option<EventHandler<(Vec<String>, BulkAction, Option<String>)>>,
    #[props(default)] on_export: Option<EventHandler<()>>,
    #[props(default)] on_create_team: Option<EventHandler<()>>,
    #[props(default)] on_edit_team: Option<EventHandler<String>>,
) -> Element {
    let uid = use_hook(|| next_id());
    let mut tab = use_signal(|| default_tab);
    let mut members = use_signal(|| member_list.clone());
    let mut search = use_signal(|| query.clone());
    let mut filter = use_signal(|| role_filter);
    let mut selected = use_signal(Vec::<String>::new);
    let mut anchor = use_signal(|| None::<String>);
    let mut invite_open = use_signal(|| false);
    let mut invite_text = use_signal(String::new);
    let mut invite_role = use_signal(|| MemberRole::Member);
    let mut pending_remove = use_signal(|| None::<String>);
    let mut expanded = use_signal(Vec::<String>::new);
    let mut row_menu = use_signal(|| None::<String>);
    let mut status = use_signal(String::new);

    let visible = filter_members(&members(), filter(), &search());
    let visible_ids: Vec<String> = visible.iter().map(|m| m.id.clone()).collect();
    let selected_now = selected();
    let state = header_state(
        visible_ids
            .iter()
            .filter(|id| selected_now.contains(id))
            .count(),
        visible_ids.len(),
    );
    let header_id = format!("{uid}-select-all");
    {
        let header_id = header_id.clone();
        use_effect(move || {
            let _ = (selected(), filter(), search(), members());
            document::eval(&format!(
                "const el = document.getElementById({header_id:?}); if (el) {{ const rows = [...document.querySelectorAll('tbody input[type=checkbox]')]; const n = rows.filter((r) => r.checked).length; el.indeterminate = n > 0 && n < rows.length; }}"
            ));
        });
    }

    let admins = members()
        .iter()
        .filter(|m| m.member_role == MemberRole::Admin)
        .count();
    let invited = members().iter().filter(|m| m.is_invited).count();
    let total_members = members().len();
    let shown_seats = if is_empty {
        Seats {
            used: 0,
            total: seats.total,
        }
    } else {
        seats
    };
    let remove_target = pending_remove();
    let remove_name = remove_target
        .as_ref()
        .and_then(|id| members().iter().find(|m| &m.id == id).map(member_name))
        .unwrap_or_default();

    if is_loading {
        return rsx! {
            SettingsShell { current: "Members", title: "Members and teams", description: "Who is in this workspace, their roles, and their teams.", on_navigate, docs_href: docs_href.clone(),
                document::Link { rel: "stylesheet", href: MEMBERS_CSS }
                div { class: "kit-set__loading", "aria-busy": "true",
                    div { class: "kit-mem__summary",
                        for slot in 0..3 { span { key: "{slot}", class: "kit-set__skel", "data-kind": "block", aria_hidden: "true" } }
                    }
                    SettingsCard { heading: "Members".to_string(),
                        for slot in 0..4 { span { key: "{slot}", class: "kit-set__skel", "data-kind": "row", aria_hidden: "true" } }
                    }
                    SettingsCard { heading: "Teams".to_string(),
                        SettingsSkel { kind: "block" }
                    }
                }
            }
        };
    }

    rsx! {
        SettingsShell { current: "Members", title: "Members and teams", description: "Who is in this workspace, their roles, and their teams.", on_navigate, docs_href: docs_href.clone(),
            document::Link { rel: "stylesheet", href: MEMBERS_CSS }
            div { class: "kit-mem__summary",
                div { class: "kit-mem__stat",
                    span { class: "kit-mem__figure", "{shown_seats.used} of {shown_seats.total}" }
                    span { class: "kit-mem__label", "seats in use" }
                }
                div { class: "kit-mem__stat",
                    span { class: "kit-mem__figure", "{total_members}" }
                    span { class: "kit-mem__label", "members, {invited} invited" }
                }
                div { class: "kit-mem__stat",
                    span { class: "kit-mem__figure", "{admins}" }
                    span { class: "kit-mem__label", "admins can manage billing, security and members" }
                }
            }
            div { class: "kit-mem__tabs", role: "tablist", aria_label: "Members and teams",
                onkeydown: move |e: KeyboardEvent| {
                    let next = match e.key() {
                        Key::ArrowRight | Key::ArrowLeft => Some(match tab() { MembersTab::Members => MembersTab::Teams, MembersTab::Teams => MembersTab::Members }),
                        _ => None,
                    };
                    if let Some(next) = next {
                        tab.set(next);
                        let id = if next == MembersTab::Members { format!("{uid}-tab-members") } else { format!("{uid}-tab-teams") };
                        document::eval(&format!("requestAnimationFrame(() => document.getElementById({id:?})?.focus());"));
                    }
                },
                button {
                    id: "{uid}-tab-members",
                    r#type: "button",
                    role: "tab",
                    class: "kit-mem__tab",
                    aria_selected: (tab() == MembersTab::Members).to_string(),
                    tabindex: if tab() == MembersTab::Members { "0" } else { "-1" },
                    aria_controls: "{uid}-panel-members",
                    onclick: move |_| tab.set(MembersTab::Members),
                    "Members"
                }
                button {
                    id: "{uid}-tab-teams",
                    r#type: "button",
                    role: "tab",
                    class: "kit-mem__tab",
                    aria_selected: (tab() == MembersTab::Teams).to_string(),
                    tabindex: if tab() == MembersTab::Teams { "0" } else { "-1" },
                    aria_controls: "{uid}-panel-teams",
                    onclick: move |_| tab.set(MembersTab::Teams),
                    "Teams"
                }
            }

            if tab() == MembersTab::Members {
                section { id: "{uid}-panel-members", role: "tabpanel", aria_labelledby: "{uid}-tab-members", class: "kit-mem__panel",
                    div { class: "kit-mem__toolbar",
                        input {
                            class: "kit-set__input kit-mem__search",
                            r#type: "search",
                            placeholder: "Search by name or email",
                            "aria-label": "Search members",
                            value: "{search()}",
                            onkeydown: move |e: KeyboardEvent| {
                                if e.key() == Key::Escape {
                                    search.set(String::new());
                                }
                            },
                            oninput: move |e: FormEvent| search.set(e.value()),
                        }
                        select {
                            class: "kit-set__input kit-mem__filter",
                            "aria-label": "Filter by role",
                            onchange: move |e: FormEvent| {
                                filter.set(match e.value().as_str() {
                                    "admin" => RoleFilter::Admin,
                                    "member" => RoleFilter::Member,
                                    "viewer" => RoleFilter::Viewer,
                                    _ => RoleFilter::All,
                                });
                            },
                            option { value: "all", selected: filter() == RoleFilter::All, "All roles" }
                            option { value: "admin", selected: filter() == RoleFilter::Admin, "Admins" }
                            option { value: "member", selected: filter() == RoleFilter::Member, "Members" }
                            option { value: "viewer", selected: filter() == RoleFilter::Viewer, "Viewers" }
                        }
                        button {
                            r#type: "button",
                            class: "kit-set__btn",
                            onclick: move |_| {
                                if let Some(h) = &on_export {
                                    h.call(());
                                }
                                status.set("Member list exported.".to_string());
                            },
                            "Export"
                        }
                        button {
                            id: "{uid}-invite-toggle",
                            r#type: "button",
                            class: "kit-set__btn kit-set__btn--primary",
                            aria_expanded: invite_open().to_string(),
                            onclick: move |_| {
                                invite_open.set(!invite_open());
                                if invite_open() {
                                    focus_later(&format!("{uid}-invite"));
                                }
                            },
                            "Invite members"
                        }
                    }

                    if invite_open() {
                        form { class: "kit-mem__invite", aria_label: "Invite members",
                            onkeydown: move |e: KeyboardEvent| {
                                if e.key() == Key::Escape {
                                    invite_open.set(false);
                                    invite_text.set(String::new());
                                }
                            },
                            onsubmit: move |e: FormEvent| {
                                e.prevent_default();
                                let emails: Vec<String> = rv2_affect::tag_input::split_tags(&invite_text(), true)
                                    .into_iter()
                                    .map(str::to_string)
                                    .collect();
                                if emails.is_empty() {
                                    status.set("Enter at least one email address.".to_string());
                                    return;
                                }
                                let count = emails.len();
                                if let Some(h) = &on_invite {
                                    h.call((emails, invite_role()));
                                }
                                status.set(format!("Invitations sent to {count} {}.", if count == 1 { "person" } else { "people" }));
                                invite_text.set(String::new());
                                invite_open.set(false);
                            },
                            label { r#for: "{uid}-invite", "Email addresses" }
                            input {
                                id: "{uid}-invite",
                                class: "kit-set__input",
                                r#type: "text",
                                placeholder: "name@company.com, another@company.com",
                                value: "{invite_text()}",
                                oninput: move |e: FormEvent| invite_text.set(e.value()),
                            }
                            div { class: "kit-set__radios", role: "radiogroup", aria_label: "Role for invitees",
                                for role in MemberRole::ALL {
                                    label { key: "{role.as_str()}", class: "kit-set__radio",
                                        input {
                                            r#type: "radio",
                                            name: "invite-role-{uid}",
                                            value: "{role.as_str()}",
                                            checked: invite_role() == role,
                                            onchange: move |_| invite_role.set(role),
                                        }
                                        "{role.label()}"
                                    }
                                }
                            }
                            p { class: "kit-set__info", "Press Enter to send. Escape closes the form." }
                        }
                    }

                    if !selected_now.is_empty() {
                        div { class: "kit-mem__bulk", role: "region", aria_label: "Bulk actions",
                            span { class: "kit-mem__bulk-count", "{count_words(selected_now.len())} selected" }
                            button {
                                r#type: "button",
                                class: "kit-set__btn",
                                onclick: {
                                    let ids = selected_now.clone();
                                    move |_| {
                                        if let Some(h) = &on_bulk {
                                            h.call((ids.clone(), BulkAction::Role, Some("member".to_string())));
                                        }
                                        status.set("Role changed for the selected members.".to_string());
                                    }
                                },
                                "Change role"
                            }
                            button {
                                r#type: "button",
                                class: "kit-set__btn",
                                onclick: {
                                    let ids = selected_now.clone();
                                    move |_| {
                                        if let Some(h) = &on_bulk {
                                            h.call((ids.clone(), BulkAction::Plan, None));
                                        }
                                        status.set("Plan changed for the selected members.".to_string());
                                    }
                                },
                                "Change plan"
                            }
                            button {
                                r#type: "button",
                                class: "kit-set__btn",
                                onclick: {
                                    let ids = selected_now.clone();
                                    move |_| {
                                        if let Some(h) = &on_bulk {
                                            h.call((ids.clone(), BulkAction::Remove, None));
                                        }
                                        selected.set(Vec::new());
                                        status.set("Selected members removed.".to_string());
                                    }
                                },
                                "Remove"
                            }
                        }
                    }

                    if visible.is_empty() {
                        div { class: "kit-mem__empty",
                            if members().is_empty() || is_empty {
                                p { class: "kit-set__info", "No members yet. Invite the people who will use this workspace." }
                                button {
                                    r#type: "button",
                                    class: "kit-set__btn kit-set__btn--primary",
                                    onclick: move |_| invite_open.set(true),
                                    "Invite members"
                                }
                            } else {
                                p { class: "kit-set__info", "No member matches this search." }
                                button {
                                    r#type: "button",
                                    class: "kit-set__btn",
                                    onclick: move |_| {
                                        search.set(String::new());
                                        filter.set(RoleFilter::All);
                                    },
                                    "Clear search"
                                }
                            }
                        }
                    } else {
                        div { class: "kit-mem__table-wrap",
                            table { class: "kit-mem__table",
                                thead {
                                    tr {
                                        th { scope: "col",
                                            input {
                                                id: "{header_id}",
                                                r#type: "checkbox",
                                                class: "kit-mem__check",
                                                checked: state == HeaderState::All,
                                                "aria-label": "Select all visible members",
                                                "aria-checked": match state { HeaderState::All => "true", HeaderState::Some => "mixed", HeaderState::None => "false" },
                                                onclick: {
                                                    let ids = visible_ids.clone();
                                                    move |_| {
                                                        if state == HeaderState::All {
                                                            selected.set(Vec::new());
                                                        } else {
                                                            selected.set(ids.clone());
                                                        }
                                                    }
                                                },
                                            }
                                        }
                                        th { scope: "col", "Member" }
                                        th { scope: "col", "Role" }
                                        th { scope: "col", class: "kit-mem__hide-narrow", "Plan" }
                                        th { scope: "col", class: "kit-mem__hide-narrow", "Usage" }
                                        th { scope: "col", "Actions" }
                                    }
                                }
                                tbody {
                                    for member in visible.clone() {
                                        {
                                            let name = member_name(&member);
                                            let is_self = member.id == current_user_id;
                                            let checked = selected_now.contains(&member.id);
                                            let id = member.id.clone();
                                            let order = visible_ids.clone();
                                            let pct = usage_percent(member.usage_ratio);
                                            let role_value = member.member_role;
                                            let usage_style = format!("width: {pct}%");
                                            let select_label = format!("Select {name}");
                                            let role_label = if is_self {
                                                format!("Role for {name}. Disabled: you cannot change your own role.")
                                            } else {
                                                format!("Role for {name}")
                                            };
                                            let remove_label = if is_self {
                                                format!("Remove {name}. Disabled: you cannot remove yourself.")
                                            } else {
                                                format!("Remove {name}")
                                            };
                                            let usage_label = format!("Usage for {name}");
                                            let menu_open = row_menu().as_deref() == Some(member.id.as_str());
                                            let more_id = format!("{uid}-more-{}", member.id);
                                            let actions_label = format!("Actions for {name}");
                                            let resend_label = format!("Resend invitation to {name}");
                                            let resend_done = format!("Invitation resent to {name}.");
                                            let cancel_label = format!("Cancel invitation for {name}");
                                            let cancel_done = format!("Invitation cancelled for {name}.");
                                            rsx! {
                                                tr { key: "{member.id}", "data-selected": checked.then_some("true"),
                                                    td {
                                                        input {
                                                            r#type: "checkbox",
                                                            class: "kit-mem__check",
                                                            checked,
                                                            "aria-label": "{select_label}",
                                                            onclick: {
                                                                let id = id.clone();
                                                                move |e: MouseEvent| {
                                                                    let shift = e.modifiers().shift();
                                                                    let ids = match (shift, anchor()) {
                                                                        (true, Some(from)) => range_between(&order, &from, &id),
                                                                        _ => vec![id.clone()],
                                                                    };
                                                                    let deselect = !shift && selected().contains(&id);
                                                                    selected.with_mut(|list| {
                                                                        if deselect {
                                                                            list.retain(|x| x != &id);
                                                                        } else {
                                                                            for x in ids.iter() {
                                                                                if !list.contains(x) {
                                                                                    list.push(x.clone());
                                                                                }
                                                                            }
                                                                        }
                                                                    });
                                                                    anchor.set(Some(id.clone()));
                                                                }
                                                            },
                                                        }
                                                    }
                                                    td { class: "kit-mem__who",
                                                        span { class: "kit-mem__name",
                                                            "{name}"
                                                            if is_self { span { class: "kit-mem__you", " (You)" } }
                                                        }
                                                        span { class: "kit-mem__email", "{member.email_address}" }
                                                    }
                                                    td {
                                                        select {
                                                            class: "kit-set__input kit-mem__role",
                                                            "aria-label": "{role_label}",
                                                            disabled: is_self,
                                                            onchange: {
                                                                let id = id.clone();
                                                                let name = name.clone();
                                                                move |e: FormEvent| {
                                                                    let next = match e.value().as_str() {
                                                                        "admin" => MemberRole::Admin,
                                                                        "viewer" => MemberRole::Viewer,
                                                                        _ => MemberRole::Member,
                                                                    };
                                                                    members.with_mut(|list| {
                                                                        if let Some(m) = list.iter_mut().find(|m| m.id == id) {
                                                                            m.member_role = next;
                                                                        }
                                                                    });
                                                                    if let Some(h) = &on_change_role {
                                                                        h.call((id.clone(), next));
                                                                    }
                                                                    status.set(format!("Role for {name} is now {}.", next.label()));
                                                                }
                                                            },
                                                            for role in MemberRole::ALL {
                                                                option { key: "{role.as_str()}", value: "{role.as_str()}", selected: role_value == role, "{role.label()}" }
                                                            }
                                                        }
                                                    }
                                                    td { class: "kit-mem__hide-narrow", "{member.plan_name}" }
                                                    td { class: "kit-mem__hide-narrow",
                                                        if member.is_invited {
                                                            span { class: "kit-set__tag", "Invited" }
                                                        } else {
                                                            div {
                                                                class: "kit-mem__usage",
                                                                role: "progressbar",
                                                                aria_label: "{usage_label}",
                                                                aria_valuemin: "0",
                                                                aria_valuemax: "100",
                                                                aria_valuenow: "{pct}",
                                                                span { class: "kit-mem__usage-fill", style: "{usage_style}" }
                                                            }
                                                            span { class: "kit-mem__usage-text", "{pct}%" }
                                                        }
                                                    }
                                                    td {
                                                        div { class: "kit-mem__actions",
                                                            button {
                                                                id: "{more_id}",
                                                                r#type: "button",
                                                                class: "kit-set__btn kit-mem__more",
                                                                "aria-haspopup": "menu",
                                                                "aria-expanded": menu_open.to_string(),
                                                                "aria-label": "{actions_label}",
                                                                onclick: {
                                                                    let id = id.clone();
                                                                    move |_| row_menu.set(if row_menu() == Some(id.clone()) { None } else { Some(id.clone()) })
                                                                },
                                                                "…"
                                                            }
                                                            if menu_open {
                                                                div { class: "kit-mem__menu", role: "menu", aria_label: "{actions_label}",
                                                                    onkeydown: {
                                                                        let more_id = more_id.clone();
                                                                        move |e: KeyboardEvent| {
                                                                            if e.key() == Key::Escape {
                                                                                row_menu.set(None);
                                                                                document::eval(&format!("document.getElementById({more_id:?})?.focus();"));
                                                                            }
                                                                        }
                                                                    },
                                                                    if member.is_invited {
                                                                        button {
                                                                            r#type: "button",
                                                                            role: "menuitem",
                                                                            class: "kit-mem__menu-item",
                                                                            "aria-label": "{resend_label}",
                                                                            onclick: {
                                                                                let done = resend_done.clone();
                                                                                move |_| {
                                                                                    row_menu.set(None);
                                                                                    status.set(done.clone());
                                                                                }
                                                                            },
                                                                            "Resend invitation"
                                                                        }
                                                                        button {
                                                                            r#type: "button",
                                                                            role: "menuitem",
                                                                            class: "kit-mem__menu-item",
                                                                            "aria-label": "{cancel_label}",
                                                                            onclick: {
                                                                                let done = cancel_done.clone();
                                                                                move |_| {
                                                                                    row_menu.set(None);
                                                                                    status.set(done.clone());
                                                                                }
                                                                            },
                                                                            "Cancel invitation"
                                                                        }
                                                                    } else {
                                                                        button {
                                                                            r#type: "button",
                                                                            role: "menuitem",
                                                                            class: "kit-mem__menu-item",
                                                                            disabled: is_self,
                                                                            "aria-label": "{remove_label}",
                                                                            onclick: {
                                                                                let id = id.clone();
                                                                                let more_id = more_id.clone();
                                                                                move |_| {
                                                                                    row_menu.set(None);
                                                                                    document::eval(&format!("document.getElementById({more_id:?})?.focus();"));
                                                                                    pending_remove.set(Some(id.clone()));
                                                                                }
                                                                            },
                                                                            "Remove"
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                section { id: "{uid}-panel-teams", role: "tabpanel", aria_labelledby: "{uid}-tab-teams", class: "kit-mem__panel",
                    div { class: "kit-mem__toolbar",
                        span { class: "kit-set__info", "{team_list.len()} teams" }
                        button {
                            r#type: "button",
                            class: "kit-set__btn kit-set__btn--primary",
                            onclick: move |_| {
                                if let Some(h) = &on_create_team {
                                    h.call(());
                                }
                            },
                            "New team"
                        }
                    }
                    if team_list.is_empty() {
                        div { class: "kit-mem__empty",
                            p { class: "kit-set__info", "No teams yet. Group members so access can follow the team." }
                        }
                    } else {
                        ul { class: "kit-mem__teams",
                            for team in team_list.clone() {
                                {
                                    let open = expanded().contains(&team.id);
                                    let team_id = team.id.clone();
                                    let team_count = team.member_ids.len();
                                    let edit_label = format!("Edit team {}", team.display_name);
                                    let names: Vec<String> = team
                                        .member_ids
                                        .iter()
                                        .filter_map(|id| members().iter().find(|m| &m.id == id).map(member_name))
                                        .collect();
                                    rsx! {
                                        li { key: "{team.id}", class: "kit-mem__team",
                                            div { class: "kit-mem__team-head",
                                                button {
                                                    r#type: "button",
                                                    class: "kit-mem__team-toggle",
                                                    aria_expanded: open.to_string(),
                                                    onclick: {
                                                        let team_id = team_id.clone();
                                                        move |_| {
                                                            let id = team_id.clone();
                                                            expanded.with_mut(|list| {
                                                                if let Some(pos) = list.iter().position(|x| *x == id) {
                                                                    list.remove(pos);
                                                                } else {
                                                                    list.push(id);
                                                                }
                                                            });
                                                        }
                                                    },
                                                    span { class: "kit-mem__team-name", "{team.display_name}" }
                                                    span { class: "kit-set__info", "{team_count} members" }
                                                }
                                                button {
                                                    r#type: "button",
                                                    class: "kit-set__btn",
                                                    "aria-label": "{edit_label}",
                                                    onclick: {
                                                        let team_id = team_id.clone();
                                                        move |_| {
                                                            if let Some(h) = &on_edit_team {
                                                                h.call(team_id.clone());
                                                            }
                                                        }
                                                    },
                                                    "Edit"
                                                }
                                            }
                                            if open {
                                                ul { class: "kit-mem__team-members",
                                                    for name in names.clone() {
                                                        li { key: "{name}", "{name}" }
                                                    }
                                                    if names.is_empty() {
                                                        li { class: "kit-set__info", "No members in this team." }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            AlertDialog {
                open: remove_target.is_some(),
                onopenchange: move |open: bool| {
                    if !open {
                        pending_remove.set(None);
                    }
                },
                title: rsx! { "Remove {remove_name}?" },
                description: rsx! { "They lose access to this workspace. Their chats stay with the workspace." },
                confirm_label: "Remove member",
                cancel_label: "Keep member",
                danger: true,
                onconfirm: move |_| {
                    if let Some(id) = pending_remove() {
                        if let Some(h) = &on_remove {
                            h.call(id.clone());
                        }
                        members.with_mut(|list| list.retain(|m| m.id != id));
                        selected.with_mut(|list| list.retain(|x| *x != id));
                        status.set(format!("Removed {remove_name}."));
                    }
                    pending_remove.set(None);
                    focus_after_frame(&format!("{uid}-invite-toggle"));
                },
                oncancel: move |_| pending_remove.set(None),
            }
            SettingsStatus { message: status() }
        }
    }
}
