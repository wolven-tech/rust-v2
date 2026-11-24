use dioxus::prelude::*;
use rv2_kit::screens::autonomy_settings::{
    ActionClass, ActionRule, AmountLimit, AppEntry, AutonomyLevel, AutonomySettings, Outcome,
    RecentEvent, Setting, WeekCounts, action_counts, app_counts, is_changed, lately,
    limit_sentence,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn rule(
    app: &str,
    id: &str,
    class: ActionClass,
    setting: Setting,
    fixed: bool,
    name: &str,
) -> ActionRule {
    ActionRule {
        id: id.to_string(),
        app_id: app.to_string(),
        action_name: name.to_string(),
        detail_text: None,
        mark: None,
        rule_setting: setting,
        default_setting: setting,
        action_class: class,
        is_fixed: fixed,
        amount_limit: None,
        week_counts: WeekCounts {
            done: 3,
            asked: 1,
            blocked: 0,
        },
        recent_events: vec![RecentEvent {
            what: "Filed reply".to_string(),
            outcome: Outcome::Done,
            occurred_at: "Today, 9:14".to_string(),
        }],
    }
}

fn event(what: &str, outcome: Outcome, when: &str) -> RecentEvent {
    RecentEvent {
        what: what.to_string(),
        outcome,
        occurred_at: when.to_string(),
    }
}

fn app(id: &str, name: &str, account: &str) -> AppEntry {
    AppEntry {
        id: id.to_string(),
        entry_name: name.to_string(),
        account: Some(account.to_string()),
        mark: None,
    }
}

fn fixture_apps() -> Vec<AppEntry> {
    vec![
        app("mail", "Mail", "ops@juniper.example"),
        app("chat", "Team chat", "Juniper Works"),
        app("code", "Code host", "juniper-works"),
        app("drive", "Shared drive", "drive@juniper.example"),
        app("payments", "Payments", "finance@juniper.example"),
        app("calendar", "Calendar", "ada@juniper.example"),
        app("browser", "Web browser", "ada@juniper.example"),
    ]
}

fn fixture_rules() -> Vec<ActionRule> {
    use ActionClass::{Read, Risky, Write};
    use Setting::{Allow, Ask, Never};
    let mut charges = rule("payments", "charges", Read, Allow, false, "Read charges");
    charges.recent_events = vec![event("Read charges", Outcome::Done, "Today, 9:14")];
    let mut credit = rule(
        "payments",
        "credit",
        Risky,
        Allow,
        false,
        "Issue credit notes",
    );
    credit.amount_limit = Some(AmountLimit {
        amount: 200.0,
        currency: "USD".to_string(),
    });
    credit.recent_events = vec![event("Credit note 1042", Outcome::Approved, "Today, 8:02")];
    let mut prices = rule(
        "payments",
        "prices",
        Risky,
        Never,
        false,
        "Change plan prices",
    );
    prices.recent_events = vec![event("Price change", Outcome::Declined, "Yesterday, 17:40")];
    vec![
        rule("mail", "read-inbox", Read, Allow, false, "Read inbox"),
        rule("mail", "search", Read, Allow, false, "Search threads"),
        rule("mail", "file", Write, Allow, false, "File replies"),
        rule("mail", "refund", Risky, Ask, false, "Issue refunds"),
        rule("mail", "bulk", Risky, Never, false, "Send bulk email"),
        rule("chat", "read-channels", Read, Allow, false, "Read channels"),
        rule("chat", "post", Write, Allow, false, "Post messages"),
        rule(
            "chat",
            "share-out",
            Risky,
            Ask,
            false,
            "Share files outside the team",
        ),
        rule(
            "chat",
            "delete-history",
            Risky,
            Never,
            false,
            "Delete channel history",
        ),
        rule(
            "code",
            "read-repos",
            Read,
            Allow,
            false,
            "Read repositories",
        ),
        rule(
            "code",
            "open-prs",
            Write,
            Allow,
            false,
            "Open pull requests",
        ),
        rule("code", "merge", Risky, Ask, false, "Merge to main"),
        rule("code", "force-push", Risky, Never, false, "Force-push"),
        rule("drive", "list", Read, Allow, false, "List files"),
        rule("drive", "move", Write, Allow, false, "Move files"),
        rule(
            "drive",
            "sharing",
            Risky,
            Ask,
            false,
            "Change sharing settings",
        ),
        rule(
            "drive",
            "delete-folders",
            Risky,
            Never,
            false,
            "Delete shared folders",
        ),
        charges,
        credit,
        prices,
        rule(
            "calendar",
            "read-calendar",
            Read,
            Allow,
            false,
            "Read calendar",
        ),
        rule(
            "calendar",
            "create-events",
            Write,
            Allow,
            false,
            "Create events",
        ),
        rule(
            "calendar",
            "outside-invites",
            Risky,
            Ask,
            false,
            "Accept invitations from outside",
        ),
        rule("browser", "read-pages", Read, Allow, false, "Read pages"),
        rule("browser", "submit-forms", Risky, Ask, false, "Submit forms"),
        rule(
            "browser",
            "downloads",
            Risky,
            Never,
            false,
            "Download files",
        ),
        rule(
            "mail",
            "delete-threads",
            Risky,
            Never,
            true,
            "Delete threads",
        ),
        rule(
            "payments",
            "move-money",
            Risky,
            Never,
            true,
            "Move money between accounts",
        ),
        rule(
            "drive",
            "share-outside",
            Risky,
            Ask,
            true,
            "Share outside the workspace",
        ),
        rule(
            "browser",
            "publish",
            Risky,
            Never,
            true,
            "Publish to the public web",
        ),
        rule(
            "code",
            "delete-repos",
            Risky,
            Never,
            true,
            "Delete repositories",
        ),
        rule(
            "chat",
            "admin-roles",
            Risky,
            Never,
            true,
            "Change admin roles",
        ),
        rule("drive", "sign", Risky, Never, true, "Sign contracts"),
    ]
}

fn rest_app() -> Element {
    rsx! {
        AutonomySettings {
            initial_level: AutonomyLevel::Balanced,
            app_list: fixture_apps(),
            action_rules: fixture_rules(),
            initial_app_id: Some("payments".to_string()),
        }
    }
}

#[test]
fn summary_at_rest_reads_fourteen_on_its_own_six_ask_first_and_six_never() {
    let html = render(rest_app);
    assert_eq!(action_counts(&fixture_rules()), (14, 6, 6));
    assert!(html.contains("14 on its own"));
    assert!(html.contains("6 ask first"));
    assert!(html.contains("6 never"));
}

#[test]
fn apps_list_at_rest_has_seven_rows_whose_counts_sum_to_the_summary() {
    let html = render(rest_app);
    assert_eq!(html.matches("class=\"kit-aus__app\"").count(), 7);
    let rules = fixture_rules();
    let sums = fixture_apps()
        .iter()
        .map(|a| app_counts(&rules, &a.id))
        .fold((0, 0, 0), |(x, y, z), (a, b, c)| (x + a, y + b, z + c));
    assert_eq!(sums, action_counts(&rules));
}

#[test]
fn fixed_rules_at_rest_show_seven_rules_and_stay_out_of_the_summary_totals() {
    let html = render(rest_app);
    assert_eq!(html.matches("class=\"kit-aus__fixed-row\"").count(), 7);
    assert_eq!(html.matches("Fixed by policy.").count(), 7);
    assert!(html.contains("The level does not change these rules."));
    assert!(html.contains("14 on its own"));
}

#[test]
fn panel_at_rest_lists_read_only_then_sensitive_and_no_edits_heading() {
    let html = render(rest_app);
    let read = html.find("Read only ·").expect("read group");
    let sensitive = html.find("Sensitive ·").expect("sensitive group");
    assert!(read < sensitive);
    assert!(html.contains("Read only · 1"));
    assert!(html.contains("Sensitive · 2"));
    assert!(!html.contains("Edits ·"));
    assert_eq!(html.matches("class=\"kit-aus__action\"").count(), 3);
}

#[test]
fn no_row_is_marked_changed_at_rest_and_the_limit_row_states_its_threshold() {
    let html = render(rest_app);
    assert_eq!(html.matches("class=\"kit-aus__changed\"").count(), 0);
    assert_eq!(html.matches(">Reset<").count(), 0);
    assert!(html.contains("Runs on its own up to USD 200.00. Asks above that amount."));
    assert!(html.contains("value=\"200.00\""));
}

#[test]
fn lately_block_at_rest_lists_three_events_with_their_outcome_words() {
    let html = render(rest_app);
    assert!(html.contains("class=\"kit-aus__group kit-aus__lately\""));
    assert!(html.contains("Read charges: Done, Today, 9:14"));
    assert!(html.contains("Credit note 1042: Approved, Today, 8:02"));
    assert!(html.contains("Price change: Declined, Yesterday, 17:40"));
    assert_eq!(lately(&fixture_rules(), "payments", 3).len(), 3);
}

#[test]
fn lately_takes_the_first_events_of_the_app_rules_and_ignores_fixed_rules() {
    let mut rules = fixture_rules();
    rules[0].recent_events = vec![event("First", Outcome::Done, "A")];
    rules.push(rule(
        "mail",
        "fixed",
        ActionClass::Risky,
        Setting::Never,
        true,
        "Fixed",
    ));
    rules.last_mut().expect("fixed rule").recent_events =
        vec![event("Fixed event", Outcome::Blocked, "B")];
    let mail = lately(&rules, "mail", 2);
    assert_eq!(mail.len(), 2);
    assert_eq!(mail[0].what, "First");
    assert!(mail.iter().all(|e| e.what != "Fixed event"));
}

#[test]
fn a_rule_held_at_never_under_a_level_that_asks_first_is_not_marked_changed() {
    let never = rule("mail", "x", ActionClass::Risky, Setting::Never, false, "X");
    assert!(!is_changed(&never, AutonomyLevel::Balanced));
    let ask = rule("mail", "x", ActionClass::Risky, Setting::Ask, false, "X");
    assert!(!is_changed(&ask, AutonomyLevel::Balanced));
}

#[test]
fn a_rule_with_an_amount_limit_under_a_level_that_asks_first_is_not_marked_changed() {
    let mut limited = rule(
        "payments",
        "x",
        ActionClass::Risky,
        Setting::Allow,
        false,
        "X",
    );
    limited.amount_limit = Some(AmountLimit {
        amount: 50.0,
        currency: "USD".to_string(),
    });
    assert!(!is_changed(&limited, AutonomyLevel::Balanced));
}

#[test]
fn a_read_rule_set_to_ask_under_careful_is_marked_changed() {
    let asked = rule("mail", "x", ActionClass::Read, Setting::Ask, false, "X");
    assert!(is_changed(&asked, AutonomyLevel::Careful));
}

#[test]
fn a_fixed_rule_is_never_marked_changed() {
    let fixed = rule("mail", "x", ActionClass::Read, Setting::Ask, true, "X");
    assert!(!is_changed(&fixed, AutonomyLevel::Careful));
}

#[test]
fn limit_sentence_names_the_currency_and_the_threshold() {
    let limit = AmountLimit {
        amount: 500.0,
        currency: "EUR".to_string(),
    };
    assert_eq!(
        limit_sentence(&limit),
        "Runs on its own up to EUR 500.00. Asks above that amount."
    );
}
