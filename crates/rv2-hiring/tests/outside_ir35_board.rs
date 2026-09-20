//! The classifiers, run over a real capture of the UK's specialist
//! outside-IR35 job board rather than over invented strings.
//!
//! The capture is 30 listings taken on 2026-09-20. It exists because the unit
//! tests all pass against phrasing chosen by the person writing them, and the
//! first run against real postings found the mandate scorer returning zero
//! signals for a Head of AI role that carries an obvious one — its phrasing
//! ("proofs of concept", "future hires", "strategic direction") was simply
//! absent from the lists.
//!
//! The assertion at the bottom is the finding this register was built to
//! produce, written as a test so a phrasing change cannot quietly reverse it.

use rv2_hiring::engagement::{Engagement, PostingSource};
use rv2_hiring::mandate::MandateStrength;
use rv2_hiring::work_pattern::WorkPattern;
use rv2_hiring::{engagement, mandate, work_pattern};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Listing {
    title: String,
    location: String,
    url: String,
    rate: String,
    description: String,
}

fn board() -> Vec<Listing> {
    let raw = include_str!("fixtures/outside-ir35-board-2026-09-20.json");
    serde_json::from_str(raw).expect("the captured board fixture must parse")
}

struct Judged {
    title: String,
    url: String,
    rate: String,
    pattern: WorkPattern,
    engagement: Engagement,
    strength: MandateStrength,
    signals: usize,
}

fn judge_all() -> Vec<Judged> {
    board()
        .into_iter()
        .map(|l| {
            let w = work_pattern::classify(&l.location, &l.description);
            let e = engagement::classify(&l.title, &l.description, PostingSource::OutsideIr35Board);
            let m = mandate::classify(&l.title, &l.description);
            Judged {
                title: l.title,
                url: l.url,
                rate: l.rate,
                pattern: w.pattern,
                engagement: e.engagement,
                strength: m.strength,
                signals: m.signals.len(),
            }
        })
        .collect()
}

#[test]
fn every_listing_on_an_outside_ir35_board_clears_the_engagement_filter() {
    let judged = judge_all();
    assert_eq!(judged.len(), 30);
    assert!(
        judged.iter().all(|j| j.engagement.is_b2b()),
        "the board admits outside-IR35 roles only, so every listing must clear \
         the engagement filter on the board's own evidence"
    );
}

#[test]
fn a_work_type_of_hybrid_with_no_stated_day_count_fails_the_office_ceiling() {
    let judged = judge_all();
    let hybrid: Vec<_> = judged
        .iter()
        .filter(|j| j.pattern == WorkPattern::HybridTwoPlus)
        .collect();
    assert!(
        !hybrid.is_empty(),
        "the capture contains hybrid listings; if this is empty the location \
         format changed and the classifier is reading nothing"
    );
    assert!(hybrid.iter().all(|j| !j.pattern.at_most_quarter_office()));
}

#[test]
fn the_head_of_ai_listing_scores_a_real_mandate() {
    let judged = judge_all();
    let head = judged
        .iter()
        .find(|j| j.title.contains("Head Of Artificial Intelligence"))
        .expect("the capture contains the Head of AI listing");

    assert_eq!(head.engagement, Engagement::OutsideIr35);
    assert_eq!(head.strength, MandateStrength::Strong);
    assert!(head.signals >= 3);
    assert_eq!(head.rate, "£900/day");
    assert_eq!(
        head.pattern,
        WorkPattern::HybridTwoPlus,
        "the board states its work type as Hybrid with no day count, which is \
         the single fact standing between this role and both filters"
    );
}

#[test]
fn remote_and_mandate_do_not_co_occur_anywhere_on_this_board() {
    let judged = judge_all();

    let clears_both: Vec<&Judged> = judged
        .iter()
        .filter(|j| j.pattern.at_most_quarter_office() && j.engagement.is_b2b())
        .collect();
    let real_mandate: Vec<&Judged> = judged
        .iter()
        .filter(|j| matches!(j.strength, MandateStrength::Strong))
        .collect();
    let both: Vec<&&Judged> = clears_both
        .iter()
        .filter(|j| matches!(j.strength, MandateStrength::Strong))
        .collect();

    assert_eq!(
        clears_both.len(),
        6,
        "six listings are remote and outside IR35: {:?}",
        clears_both.iter().map(|j| &j.title).collect::<Vec<_>>()
    );
    assert_eq!(
        real_mandate.len(),
        1,
        "one listing carries a department-building mandate: {:?}",
        real_mandate.iter().map(|j| &j.title).collect::<Vec<_>>()
    );
    assert!(
        both.is_empty(),
        "no listing on this board is both remote and mandate-carrying; if one \
         appears, the market moved and the report needs rewriting: {:?}",
        both.iter().map(|j| &j.url).collect::<Vec<_>>()
    );
}
