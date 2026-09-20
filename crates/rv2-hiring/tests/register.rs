//! The two essential filters, run over the whole register.
//!
//! The register stores what a job feed returns — a title, a location and a URL
//! — and no description. So these classifications are made from exactly what a
//! feed read leaves behind, which is the state the page renders from. Where
//! that is not enough to decide, the answer is the unknown variant and the
//! filter rejects it.

use rv2_hiring::engagement::{Engagement, PostingSource};
use rv2_hiring::mandate::{LevelBand, MandateStrength};
use rv2_hiring::model::Register;
use rv2_hiring::work_pattern::WorkPattern;
use rv2_hiring::{engagement, mandate, work_pattern};

fn register() -> Register {
    let raw = include_str!("../data/companies.json");
    serde_json::from_str(raw).expect("the register must parse into the model")
}

#[test]
fn every_company_record_in_the_register_parses() {
    let r = register();
    assert_eq!(r.companies.len(), 103);
    assert!(r.companies.iter().all(|c| !c.name.is_empty()));
}

#[test]
fn the_thunes_go_role_fails_both_essential_filters() {
    let r = register();
    let thunes = r
        .companies
        .iter()
        .find(|c| c.name == "Thunes")
        .expect("Thunes was added to the register on 2026-09-20");
    let role = &thunes
        .openings
        .as_ref()
        .expect("Thunes carries a job-board read")
        .roles[0];

    assert_eq!(role.title, "GO Senior Software Engineer");

    let pattern = work_pattern::classify(&role.location, "").pattern;
    assert_eq!(
        pattern,
        WorkPattern::Onsite,
        "the posting names a city and says nothing about remote work"
    );
    assert!(!pattern.at_most_quarter_office());

    let engaged = engagement::classify(&role.title, "", PostingSource::PermanentAtsBoard);
    assert_eq!(engaged.engagement, Engagement::Permanent);
    assert!(!engaged.engagement.is_b2b());

    let m = mandate::classify(&role.title, "");
    assert_eq!(m.level, LevelBand::Other);
    assert_eq!(m.strength, MandateStrength::None);
}

#[test]
fn no_role_read_from_an_employer_ats_board_clears_the_engagement_filter() {
    let r = register();
    let from_ats: Vec<_> = r
        .companies
        .iter()
        .filter(|c| c.ats.is_some())
        .flat_map(|c| c.openings.iter().flat_map(|o| o.roles.iter()))
        .collect();

    assert!(
        from_ats.len() >= 100,
        "the register holds {} roles read from employer boards",
        from_ats.len()
    );
    assert!(
        from_ats.iter().all(|role| {
            !engagement::classify(&role.title, "", PostingSource::PermanentAtsBoard)
                .engagement
                .is_b2b()
        }),
        "an employer applicant-tracking board advertises employment; if one of \
         these now reads as B2B, a posting stated a determination in its title"
    );
}

#[test]
fn the_remote_filter_alone_keeps_a_useful_slice_of_the_register() {
    let r = register();
    let roles: Vec<_> = r
        .companies
        .iter()
        .flat_map(|c| c.openings.iter().flat_map(|o| o.roles.iter()))
        .collect();

    let remote = roles
        .iter()
        .filter(|role| {
            work_pattern::classify(&role.location, "")
                .pattern
                .at_most_quarter_office()
        })
        .count();

    assert!(
        remote > 0 && remote < roles.len(),
        "{remote} of {} roles clear the office ceiling on their location string \
         alone; nought would mean the classifier reads nothing, and all of them \
         would mean it rejects nothing",
        roles.len()
    );
}
