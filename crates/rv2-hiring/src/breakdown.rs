//! Counting the register by one axis at a time.
//!
//! `view::facet_options` already counts companies by an index, a sector or a
//! job board. This generalises that to any axis, including the role-level ones
//! the filters actually care about — work pattern, engagement, mandate — so the
//! page can show the shape of the register rather than assert it in prose.
//!
//! The finding that motivated it: of the 30 listings on the outside-IR35 board,
//! six are remote enough and one carries a real mandate, and they are not the
//! same listing. Written down, that is a sentence a reader has to trust.
//! Grouped by work pattern beside the same set grouped by mandate, it is
//! something they can see.
//!
//! ## Every count carries its denominator
//!
//! A bare number is the lie this module exists to prevent. "6 remote roles" is
//! useless without knowing that 95 of 103 companies publish no readable board
//! at all, and actively misleading if two boards failed to read this morning.
//!
//! So [`Breakdown`] is not a `Vec` of counts. It carries `counted`, `unread`
//! and `untracked` alongside, and there is no way to construct one without
//! them. A panel that wants to render a number has the context to render it
//! honestly, whether or not its author remembered to.

use serde::{Deserialize, Serialize};

use crate::engagement::Engagement;
use crate::mandate::{LevelBand, MandateStrength};
use crate::model::{LinkStatus, Register};
use crate::view::{Filters, group_colour, group_label};
use crate::work_pattern::WorkPattern;

/// Which axis to count along.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum GroupBy {
    WorkPattern,
    Engagement,
    LevelBand,
    Mandate,
    Index,
    Sector,
    LinkStatus,
}

/// Whether an axis counts roles or the companies that hold them.
///
/// Mixing the two is how a page ends up claiming a company is remote because
/// one of its twenty roles is. The grain is a property of the axis, so a caller
/// cannot get it wrong.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Grain {
    Roles,
    Companies,
}

impl GroupBy {
    #[must_use]
    pub fn grain(self) -> Grain {
        match self {
            Self::WorkPattern | Self::Engagement | Self::LevelBand | Self::Mandate => Grain::Roles,
            Self::Index | Self::Sector | Self::LinkStatus => Grain::Companies,
        }
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::WorkPattern => "Working pattern",
            Self::Engagement => "Engagement",
            Self::LevelBand => "Level",
            Self::Mandate => "Mandate",
            Self::Index => "Index",
            Self::Sector => "Sector",
            Self::LinkStatus => "Careers link",
        }
    }

    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Self::WorkPattern => "work-pattern",
            Self::Engagement => "engagement",
            Self::LevelBand => "level",
            Self::Mandate => "mandate",
            Self::Index => "index",
            Self::Sector => "sector",
            Self::LinkStatus => "link",
        }
    }

    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|g| g.key() == key)
    }

    pub const ALL: [Self; 7] = [
        Self::WorkPattern,
        Self::Engagement,
        Self::LevelBand,
        Self::Mandate,
        Self::Index,
        Self::Sector,
        Self::LinkStatus,
    ];
}

/// One row of a breakdown.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Bucket {
    pub label: String,
    pub count: usize,
    /// A CSS colour where the axis already has one the reader knows — index
    /// colours are the only case, and reusing them keeps the chart and the
    /// entries agreeing.
    pub colour: Option<String>,
    /// Whether this bucket is the one the reader is hunting for. Exactly the
    /// values the two essential filters keep.
    pub wanted: bool,
}

/// A count along one axis, with the context that makes the numbers readable.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Breakdown {
    pub by: GroupBy,
    pub buckets: Vec<Bucket>,
    /// How many roles, or companies, actually went into the buckets.
    pub counted: usize,
    /// Companies whose job board failed to read, so their roles are missing
    /// from the count entirely.
    pub unread: usize,
    /// Companies with no readable board at all. Not a failure — most of the
    /// register is careers pages with no public feed.
    pub untracked: usize,
}

impl Breakdown {
    #[must_use]
    pub fn largest(&self) -> usize {
        self.buckets.iter().map(|b| b.count).max().unwrap_or(0)
    }

    /// Whether every bucket is empty, which is a different claim from the
    /// breakdown being unreadable.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.counted == 0
    }

    /// Whether any part of the register could not be counted.
    #[must_use]
    pub fn is_partial(&self) -> bool {
        self.unread > 0
    }

    /// The sentence that goes under the numbers, naming the denominator.
    ///
    /// Always rendered. A count without one is the failure mode this module was
    /// written to remove, so it is not an option the caller can decline.
    #[must_use]
    pub fn denominator(&self) -> String {
        let noun = match self.by.grain() {
            Grain::Roles => {
                if self.counted == 1 {
                    "role"
                } else {
                    "roles"
                }
            }
            Grain::Companies => {
                if self.counted == 1 {
                    "company"
                } else {
                    "companies"
                }
            }
        };
        let mut out = format!("{} {noun} counted", self.counted);
        if self.unread > 0 {
            out.push_str(&format!(
                ", {} job {} could not be read",
                self.unread,
                if self.unread == 1 { "board" } else { "boards" }
            ));
        }
        if self.untracked > 0 {
            out.push_str(&format!(
                ", {} {} publish no board",
                self.untracked,
                if self.untracked == 1 {
                    "company"
                } else {
                    "companies"
                }
            ));
        }
        out.push('.');
        out
    }
}

fn work_pattern_buckets() -> Vec<(WorkPattern, bool)> {
    vec![
        (WorkPattern::RemoteAnywhere, true),
        (WorkPattern::RemoteUk, true),
        (WorkPattern::RemoteFirstOneDay, true),
        (WorkPattern::HybridTwoPlus, false),
        (WorkPattern::Onsite, false),
        (WorkPattern::Unknown, false),
    ]
}

fn engagement_buckets() -> Vec<(Engagement, bool)> {
    vec![
        (Engagement::OutsideIr35, true),
        (Engagement::B2bDayRate, true),
        (Engagement::FractionalInterim, true),
        (Engagement::InsideIr35, false),
        (Engagement::Permanent, false),
        (Engagement::Unknown, false),
    ]
}

/// Count the filtered register along one axis.
///
/// The pool is whatever `filters` already keeps, so a breakdown always
/// describes what the reader is looking at rather than the whole register.
///
/// Coverage is counted over a wider pool than the buckets. A role-grain filter
/// can only keep a company that published a board, so counting the gap over the
/// filtered pool would report nought every time the question was about roles —
/// which is the one case where the reader most needs it.
#[must_use]
pub fn breakdown(register: &Register, filters: &Filters, by: GroupBy) -> Breakdown {
    let companies = crate::view::apply(register, filters);

    let mut company_grain = filters.clone();
    company_grain.remote_only = false;
    company_grain.b2b_only = false;
    company_grain.mandate_only = false;
    company_grain.applied_only = false;
    company_grain.litmus = crate::litmus::LitmusFilter::default();
    let visible = crate::view::apply(register, &company_grain);

    let unread = visible
        .iter()
        .filter(|c| {
            c.openings
                .as_ref()
                .is_some_and(|o| o.error.is_some() || o.last_read_failed.is_some())
        })
        .count();
    let untracked = visible.iter().filter(|c| c.openings.is_none()).count();

    let mut buckets: Vec<Bucket> = Vec::new();
    let mut counted = 0;

    match by {
        GroupBy::WorkPattern => {
            for (pattern, wanted) in work_pattern_buckets() {
                let count = companies
                    .iter()
                    .flat_map(|c| filters.surviving_roles(c))
                    .filter(|r| r.work_pattern == pattern)
                    .count();
                counted += count;
                buckets.push(Bucket {
                    label: pattern.label().to_string(),
                    count,
                    colour: None,
                    wanted,
                });
            }
        }
        GroupBy::Engagement => {
            for (engagement, wanted) in engagement_buckets() {
                let count = companies
                    .iter()
                    .flat_map(|c| filters.surviving_roles(c))
                    .filter(|r| r.engagement == engagement)
                    .count();
                counted += count;
                buckets.push(Bucket {
                    label: engagement.label().to_string(),
                    count,
                    colour: None,
                    wanted,
                });
            }
        }
        GroupBy::LevelBand => {
            for band in [
                LevelBand::CLevel,
                LevelBand::VpGm,
                LevelBand::Director,
                LevelBand::HeadLead,
                LevelBand::Other,
            ] {
                let count = companies
                    .iter()
                    .flat_map(|c| filters.surviving_roles(c))
                    .filter(|r| r.level() == band)
                    .count();
                counted += count;
                buckets.push(Bucket {
                    label: band.label().to_string(),
                    count,
                    colour: None,
                    wanted: band.is_litmus_target(),
                });
            }
        }
        GroupBy::Mandate => {
            for strength in [
                MandateStrength::Strong,
                MandateStrength::Partial,
                MandateStrength::Boilerplate,
                MandateStrength::None,
            ] {
                let count = companies
                    .iter()
                    .flat_map(|c| filters.surviving_roles(c))
                    .filter(|r| r.mandate.as_ref().is_some_and(|m| m.strength == strength))
                    .count();
                counted += count;
                buckets.push(Bucket {
                    label: strength.label().to_string(),
                    count,
                    colour: None,
                    wanted: strength == MandateStrength::Strong,
                });
            }
        }
        GroupBy::Index => {
            for (key, label, colour) in crate::view::GROUPS {
                let count = companies
                    .iter()
                    .filter(|c| c.indices().iter().any(|g| g == key))
                    .count();
                counted += count;
                buckets.push(Bucket {
                    label: label.to_string(),
                    count,
                    colour: Some(colour.to_string()),
                    wanted: false,
                });
            }
        }
        GroupBy::Sector => {
            let mut seen: Vec<String> = Vec::new();
            for company in &companies {
                let sector = company.sector().to_string();
                if !seen.contains(&sector) {
                    seen.push(sector);
                }
            }
            for sector in seen {
                let count = companies.iter().filter(|c| c.sector() == sector).count();
                counted += count;
                buckets.push(Bucket {
                    label: sector,
                    count,
                    colour: None,
                    wanted: false,
                });
            }
            buckets.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.label.cmp(&b.label)));
        }
        GroupBy::LinkStatus => {
            for status in [
                LinkStatus::Live,
                LinkStatus::Blocked,
                LinkStatus::Unreachable,
                LinkStatus::Broken,
                LinkStatus::Missing,
                LinkStatus::Unchecked,
            ] {
                let count = companies
                    .iter()
                    .filter(|c| c.link_status() == status)
                    .count();
                counted += count;
                buckets.push(Bucket {
                    label: status.label().to_string(),
                    count,
                    colour: None,
                    wanted: status == LinkStatus::Live,
                });
            }
        }
    }

    Breakdown {
        by,
        buckets,
        counted,
        unread,
        untracked,
    }
}

/// Label an index by its key, for callers rendering a bucket back to a filter.
#[must_use]
pub fn index_label_and_colour(key: &str) -> (&str, &'static str) {
    (group_label(key), group_colour(key))
}

#[cfg(test)]
mod tests {
    use super::{Grain, GroupBy, breakdown};
    use crate::model::Register;
    use crate::view::Filters;

    fn register() -> Register {
        serde_json::from_str::<Register>(include_str!("../data/companies.json"))
            .expect("the register must parse")
            .classified()
    }

    #[test]
    fn a_role_axis_counts_roles_and_a_company_axis_counts_companies() {
        assert_eq!(GroupBy::WorkPattern.grain(), Grain::Roles);
        assert_eq!(GroupBy::Index.grain(), Grain::Companies);
    }

    #[test]
    fn grouping_by_index_counts_every_company_in_the_register() {
        let r = register();
        let b = breakdown(&r, &Filters::default(), GroupBy::Index);
        assert_eq!(
            b.counted,
            r.companies.len(),
            "every company sits in exactly one index bucket"
        );
    }

    #[test]
    fn a_role_filter_still_reports_the_companies_it_could_not_see() {
        let r = register();
        let blind = r.companies.iter().filter(|c| c.openings.is_none()).count();
        assert!(
            blind > 0,
            "the fixture must contain companies with no board"
        );

        let b = breakdown(
            &r,
            &Filters {
                b2b_only: true,
                ..Filters::default()
            },
            GroupBy::WorkPattern,
        );

        assert_eq!(
            b.untracked, blind,
            "a role filter keeps only companies that published a board, so counting \
             the gap over the filtered pool would always report nought"
        );
        assert!(
            b.denominator().contains("publish no board"),
            "the denominator must name the gap: {}",
            b.denominator()
        );
    }

    #[test]
    fn a_company_filter_narrows_the_coverage_gap_with_the_view() {
        let r = register();
        let whole = breakdown(&r, &Filters::default(), GroupBy::WorkPattern);
        let narrowed = breakdown(
            &r,
            &Filters {
                groups: vec!["ftse100".to_string()],
                ..Filters::default()
            },
            GroupBy::WorkPattern,
        );
        assert!(
            narrowed.untracked < whole.untracked,
            "coverage follows the company-grain filters, so FTSE 100 alone cannot \
             carry the whole register's gap: {} vs {}",
            narrowed.untracked,
            whole.untracked
        );
    }

    #[test]
    fn the_remote_and_mandate_breakdowns_disagree_about_which_roles_matter() {
        let r = register();
        let f = Filters::default();

        let pattern = breakdown(&r, &f, GroupBy::WorkPattern);
        let mandate = breakdown(&r, &f, GroupBy::Mandate);

        let remote: usize = pattern
            .buckets
            .iter()
            .filter(|b| b.wanted)
            .map(|b| b.count)
            .sum();
        let strong: usize = mandate
            .buckets
            .iter()
            .filter(|b| b.wanted)
            .map(|b| b.count)
            .sum();

        assert!(remote > 0, "some roles clear the office ceiling");
        assert!(
            strong > 0,
            "some role carries a department-building mandate"
        );
        assert_ne!(
            remote, strong,
            "the two axes count different roles; if these ever match, check it is \
             not because one of them stopped counting"
        );
    }

    #[test]
    fn every_breakdown_states_its_denominator() {
        let r = register();
        let f = Filters::default();
        for by in GroupBy::ALL {
            let b = breakdown(&r, &f, by);
            let sentence = b.denominator();
            assert!(sentence.ends_with('.'), "{by:?}: {sentence}");
            assert!(
                sentence.contains(&b.counted.to_string()),
                "{by:?} does not say how many it counted: {sentence}"
            );
        }
    }

    #[test]
    fn the_denominator_names_the_companies_that_publish_no_board() {
        let r = register();
        let b = breakdown(&r, &Filters::default(), GroupBy::WorkPattern);
        assert!(
            b.untracked > 0,
            "most of the register has no readable board"
        );
        assert!(
            b.denominator().contains("publish no board"),
            "got {}",
            b.denominator()
        );
    }

    #[test]
    fn a_breakdown_follows_the_filters_rather_than_the_whole_register() {
        let r = register();
        let all = breakdown(&r, &Filters::default(), GroupBy::Index);
        let narrowed = breakdown(
            &r,
            &Filters {
                remote_only: true,
                b2b_only: true,
                ..Filters::default()
            },
            GroupBy::Index,
        );
        assert!(
            narrowed.counted < all.counted,
            "both filters on leaves one source, not the whole register"
        );
    }

    #[test]
    fn an_empty_breakdown_is_distinguishable_from_an_unreadable_one() {
        let r = register();
        let impossible = Filters {
            remote_only: true,
            b2b_only: true,
            mandate_only: true,
            ..Filters::default()
        };
        let b = breakdown(&r, &impossible, GroupBy::WorkPattern);
        assert!(b.is_empty(), "nothing clears all three");
        assert!(
            !b.is_partial(),
            "nothing failed to read; the answer is authoritative"
        );
    }

    #[test]
    fn every_axis_has_a_key_that_round_trips() {
        for by in GroupBy::ALL {
            assert_eq!(GroupBy::from_key(by.key()), Some(by));
        }
        assert_eq!(GroupBy::from_key("nonsense"), None);
    }
}

/// What a nought in this breakdown actually means.
///
/// Two states would be a lie. "Nobody is hiring" and "we could not look" are
/// obviously different, but so is the third case this register lives in: we
/// looked everywhere we can look, and most of the register has no board to
/// look at. A reader told "0 roles" without that distinction will conclude the
/// wrong thing every time.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Coverage {
    /// Every company in the pool answered. A nought here is the real answer.
    Authoritative,
    /// Some boards failed this run, so the count is a floor rather than a total.
    Partial,
    /// Nothing in the pool could be read at all.
    Blind,
}

impl Breakdown {
    /// How much of the pool this count actually saw.
    #[must_use]
    pub fn coverage(&self) -> Coverage {
        if self.unread == 0 && self.untracked == 0 {
            Coverage::Authoritative
        } else if self.counted == 0 && self.unread + self.untracked > 0 {
            Coverage::Blind
        } else {
            Coverage::Partial
        }
    }

    /// What to say when every bucket is nought.
    ///
    /// The page cannot retry — the register is compiled in — so no state here
    /// offers one. What it offers instead is the command that would change the
    /// answer, which is the only honest affordance available.
    #[must_use]
    pub fn empty_copy(&self) -> &'static str {
        match self.coverage() {
            Coverage::Authoritative => {
                "Nothing here matches. Every company in this view was checked, so that is the answer rather than a gap."
            }
            Coverage::Partial => {
                "Nothing found in the companies that could be read. Others publish no board, or their board failed on the last check, so treat this as a floor."
            }
            Coverage::Blind => {
                "No company in this view publishes a readable board, so the register cannot answer this. Run hiring-register check after adding one."
            }
        }
    }
}
