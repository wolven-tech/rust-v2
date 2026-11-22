//! Structure and demo checks for the agent clash map, amendment 2: the gallery
//! data (eight people, nine chips, eight clashes, sixteen curves), the badge
//! spacing in the centre column, and the narrow-layout geometry that the
//! stylesheet encodes.

use dioxus::prelude::*;
use rv2_kit::agents::agent_clash_map::{
    AgentClashMap, Clash, ClashWeight, Participant, Side, accent_slot, left_count, normalise,
    slot_centres, stage_height,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn person(id: &str, name: &str) -> Participant {
    Participant {
        id: id.to_string(),
        name: name.to_string(),
        note: None,
    }
}

fn side(participant: &str, agent: &str, harness: Option<&str>) -> Side {
    Side {
        participant_id: participant.to_string(),
        agent_name: agent.to_string(),
        harness_key: harness.map(str::to_string),
    }
}

fn clash(id: &str, weight: ClashWeight, label: &str, sides: Vec<Side>) -> Clash {
    Clash {
        id: id.to_string(),
        weight,
        label: label.to_string(),
        summary: format!("{label} summary"),
        sides,
    }
}

/// The gallery's demo data, mirrored here so the structure can be checked
/// without the example crate.
fn gallery_participants() -> Vec<Participant> {
    vec![
        person("ana", "Ana Cruz"),
        person("mira", "Mira Okafor"),
        person("dev", "Dev Patel"),
        person("ivo", "Ivo Lind"),
        person("sol", "Sol Varga"),
        person("lee", "Lee Park"),
        person("rui", "Rui Santos"),
        person("yuki", "Yuki Tanaka"),
    ]
}

fn gallery_clashes() -> Vec<Clash> {
    vec![
        clash(
            "files",
            ClashWeight::Decide,
            "Shared files",
            vec![
                side("ana", "Coder", Some("cli")),
                side("sol", "Docs writer", None),
            ],
        ),
        clash(
            "overwrite",
            ClashWeight::Decide,
            "Overwrite risk",
            vec![
                side("ivo", "Migrator", Some("cli")),
                side("rui", "Fixer", Some("cloud")),
            ],
        ),
        clash(
            "refund",
            ClashWeight::Notice,
            "Contradiction",
            vec![
                side("mira", "Release bot", Some("cloud")),
                side("lee", "Billing bot", Some("cli")),
            ],
        ),
        clash(
            "same-files",
            ClashWeight::Notice,
            "Same files",
            vec![
                side("ana", "Reviewer", Some("cli")),
                side("lee", "Tester", Some("cli")),
            ],
        ),
        clash(
            "retry",
            ClashWeight::Notice,
            "Duplicate fix",
            vec![
                side("dev", "Tester", Some("cli")),
                side("rui", "Fixer", Some("cloud")),
            ],
        ),
        clash(
            "plan",
            ClashWeight::Notice,
            "Stale plan",
            vec![
                side("ivo", "Migrator", Some("cli")),
                side("lee", "Tester", Some("cli")),
            ],
        ),
        clash(
            "guide",
            ClashWeight::Faint,
            "Same work",
            vec![
                side("dev", "Tester", Some("cli")),
                side("sol", "Docs writer", None),
            ],
        ),
        clash(
            "tests",
            ClashWeight::Faint,
            "Same tests",
            vec![
                side("mira", "Release bot", Some("cloud")),
                side("rui", "Fixer", Some("cloud")),
            ],
        ),
    ]
}

fn gallery() -> Element {
    let participants = gallery_participants();
    let clashes = gallery_clashes();
    rsx! { AgentClashMap { participants, clashes, min_height: 340 } }
}

#[test]
fn gallery_map_draws_nine_chips_five_on_the_left_and_four_on_the_right() {
    let html = render(gallery);
    assert_eq!(html.matches("class=\"kit-acm__chip\"").count(), 9);
    assert_eq!(html.matches("data-side=\"left\"").count(), 5);
    assert_eq!(html.matches("data-side=\"right\"").count(), 4);
}

#[test]
fn gallery_map_draws_eight_badges_and_sixteen_curves() {
    let html = render(gallery);
    assert_eq!(html.matches("class=\"kit-acm__badge\"").count(), 8);
    assert_eq!(html.matches("class=\"kit-acm__curve\"").count(), 16);
}

#[test]
fn each_badge_joins_one_left_chip_and_one_right_chip_and_a_repeated_chip_draws_one_curve_per_clash()
{
    let map = normalise(&gallery_participants(), &gallery_clashes());
    let left = left_count(map.agents.len());
    for clash in 0..map.clashes.len() {
        let agents = map.clash_agents(clash);
        let id = &map.clashes[clash].id;
        assert_eq!(agents.iter().filter(|&&a| a < left).count(), 1, "{id}");
        assert_eq!(agents.iter().filter(|&&a| a >= left).count(), 1, "{id}");
    }
    let rui = map.agent_index("rui", "Fixer").expect("Rui's chip exists");
    assert_eq!(map.agent_clashes(rui).len(), 3);
}

#[test]
fn no_two_badges_sit_level_in_the_centre_column() {
    let map = normalise(&gallery_participants(), &gallery_clashes());
    let chips = map.agents.len();
    let height = stage_height(
        left_count(chips),
        chips - left_count(chips),
        map.clashes.len(),
        0.0,
    );
    let centres = slot_centres(map.clashes.len(), height);
    const BADGE_PX: f64 = 28.0;
    assert!(centres.windows(2).all(|pair| pair[1] - pair[0] >= BADGE_PX));
}

#[test]
fn yuki_appears_only_in_the_in_sync_line() {
    let map = normalise(&gallery_participants(), &gallery_clashes());
    assert_eq!(map.in_sync, vec!["Yuki Tanaka".to_string()]);
    let html = render(gallery);
    assert!(html.contains("In sync: Yuki Tanaka"));
    assert_eq!(html.matches("Yuki").count(), 1);
}

#[test]
fn each_badge_carries_its_weight_word_and_the_legend_lists_all_three() {
    let html = render(gallery);
    assert_eq!(html.matches("class=\"kit-acm__weight\">").count(), 8);
    assert!(html.contains("class=\"kit-acm__weight\">Decide<"));
    assert!(html.contains("class=\"kit-acm__weight\">Notice<"));
    assert!(html.contains("class=\"kit-acm__weight\">Faint<"));
    assert!(html.contains("Decide: needs a person now"));
    assert!(html.contains("Notice: a heads-up"));
    assert!(html.contains("Faint: background"));
}

#[test]
fn narrow_stylesheet_puts_badges_in_the_right_half_beside_forty_six_percent_chips() {
    let css = include_str!("../assets/agent-clash-map.css");
    assert!(css.contains(".kit-acm__stage[data-narrow=\"true\"] .kit-acm__chip { width: 46%;"));
    assert!(css.contains("left: 54%; right: auto;"));
    assert!(css.contains("width: 46%; max-width: 46%;"));
}

#[test]
fn identity_slots_stay_within_the_six_kit_identities() {
    for key in ["cli", "cloud", "Docs writer", "Release bot", "", "🚀"] {
        let slot = accent_slot(key);
        assert!((1..=6).contains(&slot), "slot {slot} for {key:?}");
    }
    assert_eq!(accent_slot("cli"), accent_slot("cli"));
}
