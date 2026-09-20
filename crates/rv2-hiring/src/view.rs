//! What the page shows: the filter state, what survives it, and in what order.
//!
//! This lives beside the classifiers rather than in the app because the page
//! and the command line must agree about what a filtered register is. A
//! `Filters` here is the same value the URL hash encodes, so a bookmark
//! reopens the same view, and the same value a CSV export renders.
//!
//! ## Two grains
//!
//! Most filters are about a **company** — which index it sits in, whether its
//! careers link answers. The two essential filters and the Litmus axes are
//! about a **role**: a company survives when at least one of its roles does,
//! and only the surviving roles render beneath it. Mixing the two grains is
//! how a page ends up claiming a company is remote because one of its
//! twenty roles is.

use serde::{Deserialize, Serialize};

use crate::litmus::LitmusFilter;
use crate::mandate::MandateStrength;
use crate::model::{Company, LinkStatus, Register, Role};
use crate::text::words;

/// How many facet options are shown before a "show all" control appears.
pub const FACET_PREVIEW: usize = 8;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Sort {
    #[default]
    Group,
    Roles,
    Rust,
    Depth,
    Mandate,
    Name,
    Link,
}

impl Sort {
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Self::Group => "group",
            Self::Roles => "roles",
            Self::Rust => "rust",
            Self::Depth => "depth",
            Self::Mandate => "mandate",
            Self::Name => "name",
            Self::Link => "link",
        }
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Group => "Index, then open roles",
            Self::Roles => "UK engineering roles",
            Self::Rust => "Rust postings",
            Self::Depth => "How Rust-first the work is",
            Self::Mandate => "Roles that build a department",
            Self::Name => "Company name",
            Self::Link => "Careers link status",
        }
    }

    #[must_use]
    pub fn from_key(key: &str) -> Self {
        Self::ALL
            .iter()
            .copied()
            .find(|s| s.key() == key)
            .unwrap_or_default()
    }

    pub const ALL: [Self; 7] = [
        Self::Group,
        Self::Roles,
        Self::Rust,
        Self::Depth,
        Self::Mandate,
        Self::Name,
        Self::Link,
    ];
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Layout {
    #[default]
    Register,
    Table,
}

/// The index groups, in the order they are shown, each with the colour it
/// carries everywhere.
///
/// One colour per index, used on the ticker badge, the index chip and the
/// filter swatch alike. That repetition is the feature: it is what lets a
/// reader scan a long list and see which market a company sits on without
/// reading a word.
pub const GROUPS: [(&str, &str, &str); 5] = [
    ("ftse100", "FTSE 100", "var(--color-index-ftse100)"),
    ("ftse250", "FTSE 250", "var(--color-index-ftse250)"),
    ("smallcap", "FTSE SmallCap", "var(--color-index-smallcap)"),
    ("aim", "AIM", "var(--color-index-aim)"),
    (
        "targets",
        "Targets outside UK indices",
        "var(--color-index-targets)",
    ),
];

#[must_use]
pub fn group_label(key: &str) -> &str {
    GROUPS
        .iter()
        .find(|(k, _, _)| *k == key)
        .map_or(key, |(_, label, _)| *label)
}

/// The colour for an index, falling back to the target-list colour so an
/// unknown key renders as something rather than as nothing.
#[must_use]
pub fn group_colour(key: &str) -> &'static str {
    GROUPS
        .iter()
        .find(|(k, _, _)| *k == key)
        .map_or("var(--color-index-targets)", |(_, _, colour)| *colour)
}

fn group_rank(key: &str) -> usize {
    GROUPS
        .iter()
        .position(|(k, _, _)| *k == key)
        .unwrap_or(GROUPS.len())
}

/// Everything the reader has switched on.
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Filters {
    pub q: String,
    pub groups: Vec<String>,
    pub sectors: Vec<String>,
    pub feeds: Vec<String>,

    pub hiring: bool,
    pub rust: bool,
    pub rust_first: bool,
    pub public_equity: bool,
    pub live: bool,

    /// Essential filter one: at most 25% in-office.
    pub remote_only: bool,
    /// Essential filter two: a B2B engagement the reader can take through
    /// their own company.
    pub b2b_only: bool,
    /// Only roles that carry a department-building mandate.
    pub mandate_only: bool,

    pub litmus: LitmusFilter,
    pub sort: Sort,
    pub layout: Layout,
}

impl Filters {
    /// Whether any filter operates on roles rather than on companies.
    #[must_use]
    pub fn role_filters_active(&self) -> bool {
        self.remote_only || self.b2b_only || self.mandate_only || !self.litmus.is_empty()
    }

    /// How many controls are on, for the count beside the rail.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.groups.len()
            + self.sectors.len()
            + self.feeds.len()
            + usize::from(self.hiring)
            + usize::from(self.rust)
            + usize::from(self.rust_first)
            + usize::from(self.public_equity)
            + usize::from(self.live)
            + usize::from(self.remote_only)
            + usize::from(self.b2b_only)
            + usize::from(self.mandate_only)
            + self.litmus.active_axes()
    }

    #[must_use]
    pub fn is_clear(&self) -> bool {
        self.q.is_empty() && self.active_count() == 0
    }

    /// Which of a company's roles survive the role-grain filters.
    ///
    /// With no role filter on, every role survives, which is what keeps the
    /// register readable before the reader has asked anything of it.
    #[must_use]
    pub fn surviving_roles<'a>(&self, company: &'a Company) -> Vec<&'a Role> {
        let Some(openings) = company.openings.as_ref() else {
            return Vec::new();
        };
        openings
            .roles
            .iter()
            .filter(|role| {
                if self.remote_only && !role.work_pattern.at_most_quarter_office() {
                    return false;
                }
                if self.b2b_only && !role.engagement.is_b2b() {
                    return false;
                }
                if self.mandate_only
                    && !role
                        .mandate
                        .as_ref()
                        .is_some_and(|m| matches!(m.strength, MandateStrength::Strong))
                {
                    return false;
                }
                self.litmus.matches_role(company, role)
            })
            .collect()
    }

    /// Whether a company survives, ignoring one facet so that facet can count
    /// its own options against the rest of the filters.
    #[must_use]
    pub fn matches(&self, company: &Company, except: Option<&str>) -> bool {
        let skip = |name: &str| except == Some(name);

        if !self.q.is_empty() {
            let hay = haystack(company);
            if !words(&self.q)
                .split_whitespace()
                .all(|token| hay.contains(token))
            {
                return false;
            }
        }

        if !skip("groups") && !self.groups.is_empty() {
            let indices = company.indices();
            if !indices.iter().any(|g| self.groups.contains(g)) {
                return false;
            }
        }
        if !skip("sectors")
            && !self.sectors.is_empty()
            && !self.sectors.iter().any(|s| s == company.sector())
        {
            return false;
        }
        if !skip("feeds")
            && !self.feeds.is_empty()
            && !self.feeds.iter().any(|f| f == company.feed())
        {
            return false;
        }

        if !skip("hiring") && self.hiring && !company.hiring_now() {
            return false;
        }
        if !skip("rust") && self.rust && !company.rust_signal() {
            return false;
        }
        if !skip("rustFirst") && self.rust_first && company.rust_depth() != "first" {
            return false;
        }
        if !skip("publicEquity") && self.public_equity && !company.public_equity() {
            return false;
        }
        if !skip("live") && self.live && company.link_status() != LinkStatus::Live {
            return false;
        }

        if self.role_filters_active() && self.surviving_roles(company).is_empty() {
            return false;
        }

        true
    }
}

/// Every word a search can match, folded once per company.
#[must_use]
pub fn haystack(company: &Company) -> String {
    let mut parts = vec![
        company.name.clone(),
        company.ticker(),
        company.sector().to_string(),
        company.uk_locations().to_string(),
        company.tech_stack_note().to_string(),
        company.feed().to_string(),
        company.exchange().to_string(),
    ];
    parts.extend(company.indices().iter().map(|g| group_label(g).to_string()));
    if let Some(openings) = company.openings.as_ref() {
        parts.extend(openings.roles.iter().map(|r| r.title.clone()));
    }
    words(&parts.join(" "))
}

/// The companies that survive, in the reader's chosen order.
#[must_use]
pub fn apply<'a>(register: &'a Register, filters: &Filters) -> Vec<&'a Company> {
    let mut list: Vec<&Company> = register
        .companies
        .iter()
        .filter(|c| filters.matches(c, None))
        .collect();

    let uk_engineering = |c: &Company| c.openings.as_ref().map_or(0, |o| o.uk_engineering);
    let rust_titles = |c: &Company| c.openings.as_ref().map_or(0, |o| o.uk_rust_in_title);
    let rust_postings = |c: &Company| c.openings.as_ref().map_or(0, |o| o.uk_rust);
    let best_mandate = |c: &Company| {
        filters
            .surviving_roles(c)
            .iter()
            .filter_map(|r| r.mandate.as_ref())
            .map(|m| (m.strength, usize::MAX - m.signals.len()))
            .min()
            .unwrap_or((MandateStrength::None, usize::MAX))
    };
    let primary_group = |c: &Company| {
        c.indices()
            .iter()
            .map(|g| group_rank(g))
            .min()
            .unwrap_or(GROUPS.len())
    };

    match filters.sort {
        Sort::Name => list.sort_by(|a, b| a.name.cmp(&b.name)),
        Sort::Roles => list.sort_by(|a, b| {
            uk_engineering(b)
                .cmp(&uk_engineering(a))
                .then_with(|| a.name.cmp(&b.name))
        }),
        Sort::Rust => list.sort_by(|a, b| {
            rust_titles(b)
                .cmp(&rust_titles(a))
                .then_with(|| rust_postings(b).cmp(&rust_postings(a)))
                .then_with(|| a.name.cmp(&b.name))
        }),
        Sort::Depth => list.sort_by(|a, b| {
            // A company nobody has graded sorts below every graded one rather
            // than above them, which is what a missing value would otherwise do.
            let depth = |c: &Company| c.depth().map_or(u8::MAX, |d| d.rank());
            depth(a)
                .cmp(&depth(b))
                .then_with(|| rust_titles(b).cmp(&rust_titles(a)))
                .then_with(|| a.name.cmp(&b.name))
        }),
        Sort::Mandate => list.sort_by(|a, b| {
            best_mandate(a)
                .cmp(&best_mandate(b))
                .then_with(|| a.name.cmp(&b.name))
        }),
        Sort::Link => list.sort_by(|a, b| {
            a.link_status()
                .rank()
                .cmp(&b.link_status().rank())
                .then_with(|| a.name.cmp(&b.name))
        }),
        Sort::Group => list.sort_by(|a, b| {
            primary_group(a)
                .cmp(&primary_group(b))
                .then_with(|| uk_engineering(b).cmp(&uk_engineering(a)))
                .then_with(|| a.name.cmp(&b.name))
        }),
    }

    list
}

/// One facet option: its value, its label, and how many companies it would
/// match given every other filter.
pub struct FacetOption {
    pub value: String,
    pub label: String,
    pub count: usize,
}

/// Count a facet's options against a pool that excludes that same facet.
///
/// Excluding it is what stops selecting one option zeroing its siblings, which
/// would make the rail look broken the moment anyone used it.
#[must_use]
pub fn facet_options(register: &Register, filters: &Filters, facet: &str) -> Vec<FacetOption> {
    let pool: Vec<&Company> = register
        .companies
        .iter()
        .filter(|c| filters.matches(c, Some(facet)))
        .collect();

    let selected: &[String] = match facet {
        "groups" => &filters.groups,
        "sectors" => &filters.sectors,
        _ => &filters.feeds,
    };

    let mut options: Vec<FacetOption> = match facet {
        "groups" => GROUPS
            .iter()
            .map(|(key, label, _)| FacetOption {
                value: (*key).to_string(),
                label: (*label).to_string(),
                count: pool
                    .iter()
                    .filter(|c| c.indices().iter().any(|g| g == key))
                    .count(),
            })
            .collect(),
        "sectors" => distinct(register, |c| c.sector().to_string())
            .into_iter()
            .map(|value| FacetOption {
                count: pool.iter().filter(|c| c.sector() == value).count(),
                label: value.clone(),
                value,
            })
            .collect(),
        _ => distinct(register, |c| c.feed().to_string())
            .into_iter()
            .map(|value| FacetOption {
                count: pool.iter().filter(|c| c.feed() == value).count(),
                label: value.clone(),
                value,
            })
            .collect(),
    };

    options.retain(|o| o.count > 0 || selected.contains(&o.value));
    if facet != "groups" {
        options.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.label.cmp(&b.label)));
    }
    options
}

fn distinct(register: &Register, of: impl Fn(&Company) -> String) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    for company in &register.companies {
        let value = of(company);
        if !seen.contains(&value) {
            seen.push(value);
        }
    }
    seen
}

#[cfg(test)]
mod tests {
    use super::{Filters, Sort, apply, facet_options};
    use crate::model::Register;

    fn register() -> Register {
        let raw = include_str!("../data/companies.json");
        serde_json::from_str::<Register>(raw)
            .expect("the register must parse")
            .classified()
    }

    #[test]
    fn an_empty_filter_shows_the_whole_register() {
        let r = register();
        let f = Filters::default();
        assert!(f.is_clear());
        assert_eq!(apply(&r, &f).len(), r.companies.len());
    }

    #[test]
    fn both_essential_filters_together_leave_only_the_outside_ir35_board() {
        let r = register();
        let f = Filters {
            remote_only: true,
            b2b_only: true,
            ..Filters::default()
        };
        let list = apply(&r, &f);
        assert_eq!(
            list.len(),
            1,
            "only one source in the register carries roles that are both remote \
             and B2B: {:?}",
            list.iter().map(|c| &c.name).collect::<Vec<_>>()
        );
        assert!(list[0].name.contains("Outside IR35"));
        assert_eq!(f.surviving_roles(list[0]).len(), 6);
    }

    #[test]
    fn adding_the_mandate_filter_to_both_essentials_empties_the_register() {
        let r = register();
        let f = Filters {
            remote_only: true,
            b2b_only: true,
            mandate_only: true,
            ..Filters::default()
        };
        assert!(
            apply(&r, &f).is_empty(),
            "nothing in the register is remote, B2B and department-building at once"
        );
    }

    #[test]
    fn the_mandate_filter_alone_finds_the_head_of_ai_listing() {
        let r = register();
        let f = Filters {
            mandate_only: true,
            ..Filters::default()
        };
        let list = apply(&r, &f);
        let titles: Vec<&str> = list
            .iter()
            .flat_map(|c| f.surviving_roles(c))
            .map(|role| role.title.as_str())
            .collect();
        assert!(
            titles
                .iter()
                .any(|t| t.contains("Head Of Artificial Intelligence")),
            "got {titles:?}"
        );
    }

    #[test]
    fn a_search_matches_every_word_or_nothing() {
        let r = register();
        let hit = Filters {
            q: "thunes payments".to_string(),
            ..Filters::default()
        };
        let miss = Filters {
            q: "thunes shipbuilding".to_string(),
            ..Filters::default()
        };
        assert_eq!(apply(&r, &hit).len(), 1);
        assert!(apply(&r, &miss).is_empty());
    }

    #[test]
    fn a_facet_counts_its_options_against_a_pool_that_excludes_itself() {
        let r = register();
        let none = Filters::default();
        let baseline = facet_options(&r, &none, "groups");

        let mut picked = Filters::default();
        picked.groups.push("ftse100".to_string());
        let after = facet_options(&r, &picked, "groups");

        for (a, b) in baseline.iter().zip(after.iter()) {
            assert_eq!(
                a.count, b.count,
                "selecting one index must not zero its siblings"
            );
        }
    }

    #[test]
    fn sorting_by_mandate_puts_a_department_building_role_first() {
        let r = register();
        let f = Filters {
            mandate_only: true,
            sort: Sort::Mandate,
            ..Filters::default()
        };
        let list = apply(&r, &f);
        assert!(!list.is_empty());
        assert!(
            f.surviving_roles(list[0])
                .iter()
                .any(|role| role.mandate.as_ref().is_some_and(|m| !m.signals.is_empty()))
        );
    }

    #[test]
    fn the_active_count_adds_up_across_every_kind_of_control() {
        let f = Filters {
            groups: vec!["ftse100".to_string()],
            hiring: true,
            remote_only: true,
            b2b_only: true,
            ..Filters::default()
        };
        assert_eq!(f.active_count(), 4);
        assert!(!f.is_clear());
    }
}

/// The filter state as a URL fragment, and back.
///
/// Kept here rather than in the page because a bookmark is a contract: the
/// string a reader saves last week has to mean the same thing today, and that
/// is easier to hold true when encoding and decoding sit next to each other
/// with tests between them.
///
/// Values are percent-encoded for the handful of characters that would
/// otherwise end a field, and nothing else. A register search is company names
/// and city names; escaping the whole of RFC 3986 would make the common case
/// unreadable in the address bar for no gain.
pub mod hash {
    use super::{Filters, Layout, Sort};
    use crate::litmus::{Challenge, LitmusFilter, Situation, Timing};
    use crate::mandate::LevelBand;
    use crate::model::{MarketReach, OrgScale};

    fn encode(value: &str) -> String {
        value
            .chars()
            .map(|c| match c {
                '%' => "%25".to_string(),
                '&' => "%26".to_string(),
                '=' => "%3D".to_string(),
                ',' => "%2C".to_string(),
                '#' => "%23".to_string(),
                ' ' => "+".to_string(),
                _ => c.to_string(),
            })
            .collect()
    }

    fn decode(value: &str) -> String {
        let mut out = String::with_capacity(value.len());
        let mut chars = value.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '+' {
                out.push(' ');
                continue;
            }
            if c != '%' {
                out.push(c);
                continue;
            }
            let hex: String = chars.by_ref().take(2).collect();
            match u8::from_str_radix(&hex, 16) {
                Ok(byte) => out.push(byte as char),
                Err(_) => {
                    out.push('%');
                    out.push_str(&hex);
                }
            }
        }
        out
    }

    fn list(values: &[String]) -> String {
        values
            .iter()
            .map(|v| encode(v))
            .collect::<Vec<_>>()
            .join(",")
    }

    fn unlist(raw: &str) -> Vec<String> {
        raw.split(',')
            .filter(|s| !s.is_empty())
            .map(decode)
            .collect()
    }

    fn codes<T: Copy + PartialEq>(values: &[T], all: &[(T, &str)]) -> String {
        values
            .iter()
            .filter_map(|v| all.iter().find(|(k, _)| k == v).map(|(_, code)| *code))
            .collect::<Vec<_>>()
            .join(",")
    }

    fn uncodes<T: Copy>(raw: &str, all: &[(T, &str)]) -> Vec<T> {
        raw.split(',')
            .filter_map(|part| {
                all.iter()
                    .find(|(_, code)| *code == part)
                    .map(|(value, _)| *value)
            })
            .collect()
    }

    const LEVELS: &[(LevelBand, &str)] = &[
        (LevelBand::CLevel, "c"),
        (LevelBand::VpGm, "v"),
        (LevelBand::Director, "d"),
        (LevelBand::HeadLead, "h"),
        (LevelBand::Other, "o"),
    ];
    const MARKETS: &[(MarketReach, &str)] = &[
        (MarketReach::Local, "l"),
        (MarketReach::Regional, "r"),
        (MarketReach::International, "i"),
        (MarketReach::MultiRegion, "m"),
    ];
    const SCALES: &[(OrgScale, &str)] = &[
        (OrgScale::Under50m, "s"),
        (OrgScale::From50mTo500m, "m"),
        (OrgScale::Over500m, "l"),
        (OrgScale::GlobalCorporate, "g"),
        (OrgScale::Unknown, "u"),
    ];
    const SITUATIONS: &[(Situation, &str)] = &[
        (Situation::AdvancingSameTrack, "a"),
        (Situation::MovingHigher, "h"),
        (Situation::TransitioningAcrossFunctions, "x"),
        (Situation::ShiftingToAdvisory, "b"),
    ];
    const TIMINGS: &[(Timing, &str)] = &[
        (Timing::Exploring, "e"),
        (Timing::ThreeToSixMonths, "6"),
        (Timing::OneToThreeMonths, "3"),
        (Timing::Immediately, "n"),
    ];
    const CHALLENGES: &[(Challenge, &str)] = &[
        (Challenge::LimitedInterviewTraction, "t"),
        (Challenge::ConsideredBelowMyLevel, "u"),
        (Challenge::ProfileNotReflectingScope, "s"),
        (Challenge::UnclearPositioning, "p"),
    ];

    /// Encode, omitting anything at its default so a cleared page has an empty
    /// fragment rather than a paragraph of defaults.
    #[must_use]
    pub fn write(f: &Filters) -> String {
        let mut parts: Vec<String> = Vec::new();
        fn put(parts: &mut Vec<String>, key: &str, value: String) {
            if !value.is_empty() {
                parts.push(format!("{key}={value}"));
            }
        }

        put(&mut parts, "q", encode(&f.q));
        put(&mut parts, "g", list(&f.groups));
        put(&mut parts, "s", list(&f.sectors));
        put(&mut parts, "f", list(&f.feeds));
        for (on, key) in [
            (f.hiring, "hiring"),
            (f.rust, "rust"),
            (f.rust_first, "rustfirst"),
            (f.public_equity, "public"),
            (f.live, "live"),
            (f.remote_only, "remote"),
            (f.b2b_only, "b2b"),
            (f.mandate_only, "mandate"),
        ] {
            if on {
                parts.push(format!("{key}=1"));
            }
        }
        put(&mut parts, "lv", codes(&f.litmus.levels, LEVELS));
        put(&mut parts, "mk", codes(&f.litmus.markets, MARKETS));
        put(&mut parts, "sc", codes(&f.litmus.scales, SCALES));
        put(&mut parts, "si", codes(&f.litmus.situations, SITUATIONS));
        put(&mut parts, "ti", codes(&f.litmus.timings, TIMINGS));
        put(&mut parts, "ch", codes(&f.litmus.challenges, CHALLENGES));
        put(&mut parts, "co", list(&f.litmus.countries));
        put(&mut parts, "in", list(&f.litmus.industries));
        if f.sort != Sort::default() {
            parts.push(format!("sort={}", f.sort.key()));
        }
        if f.layout == Layout::Table {
            parts.push("view=table".to_string());
        }
        parts.join("&")
    }

    /// Decode. An unrecognised key is ignored rather than rejected: a fragment
    /// is user-editable, and a bookmark from an older version of the page
    /// should still open.
    #[must_use]
    pub fn read(fragment: &str) -> Filters {
        let mut f = Filters::default();
        let mut litmus = LitmusFilter::default();

        for pair in fragment.trim_start_matches('#').split('&') {
            let Some((key, value)) = pair.split_once('=') else {
                continue;
            };
            let on = value == "1";
            match key {
                "q" => f.q = decode(value),
                "g" => f.groups = unlist(value),
                "s" => f.sectors = unlist(value),
                "f" => f.feeds = unlist(value),
                "hiring" => f.hiring = on,
                "rust" => f.rust = on,
                "rustfirst" => f.rust_first = on,
                "public" => f.public_equity = on,
                "live" => f.live = on,
                "remote" => f.remote_only = on,
                "b2b" => f.b2b_only = on,
                "mandate" => f.mandate_only = on,
                "lv" => litmus.levels = uncodes(value, LEVELS),
                "mk" => litmus.markets = uncodes(value, MARKETS),
                "sc" => litmus.scales = uncodes(value, SCALES),
                "si" => litmus.situations = uncodes(value, SITUATIONS),
                "ti" => litmus.timings = uncodes(value, TIMINGS),
                "ch" => litmus.challenges = uncodes(value, CHALLENGES),
                "co" => litmus.countries = unlist(value),
                "in" => litmus.industries = unlist(value),
                "sort" => f.sort = Sort::from_key(value),
                "view" => {
                    f.layout = if value == "table" {
                        Layout::Table
                    } else {
                        Layout::Register
                    };
                }
                _ => {}
            }
        }

        f.litmus = litmus;
        f
    }

    #[cfg(test)]
    mod tests {
        use super::{read, write};
        use crate::litmus::{Challenge, Timing};
        use crate::mandate::LevelBand;
        use crate::view::{Filters, Layout, Sort};

        #[test]
        fn a_cleared_page_has_an_empty_fragment() {
            assert_eq!(write(&Filters::default()), "");
        }

        #[test]
        fn every_field_survives_a_round_trip() {
            let mut f = Filters {
                q: "auto trader".to_string(),
                groups: vec!["ftse250".to_string()],
                sectors: vec!["Enterprise software".to_string()],
                remote_only: true,
                b2b_only: true,
                mandate_only: true,
                hiring: true,
                sort: Sort::Mandate,
                layout: Layout::Table,
                ..Filters::default()
            };
            f.litmus.levels = vec![LevelBand::Director, LevelBand::VpGm];
            f.litmus.timings = vec![Timing::Immediately];
            f.litmus.challenges = vec![Challenge::ConsideredBelowMyLevel];
            f.litmus.countries = vec!["United Kingdom".to_string()];

            let back = read(&write(&f));
            assert_eq!(back.q, f.q);
            assert_eq!(back.groups, f.groups);
            assert_eq!(back.sectors, f.sectors);
            assert!(back.remote_only && back.b2b_only && back.mandate_only && back.hiring);
            assert_eq!(back.sort, Sort::Mandate);
            assert_eq!(back.layout, Layout::Table);
            assert_eq!(back.litmus.levels, f.litmus.levels);
            assert_eq!(back.litmus.timings, f.litmus.timings);
            assert_eq!(back.litmus.challenges, f.litmus.challenges);
            assert_eq!(back.litmus.countries, f.litmus.countries);
        }

        #[test]
        fn a_search_containing_a_separator_survives_encoding() {
            let f = Filters {
                q: "a&b=c,d".to_string(),
                ..Filters::default()
            };
            assert_eq!(read(&write(&f)).q, "a&b=c,d");
        }

        #[test]
        fn an_unknown_key_from_an_older_bookmark_is_ignored_rather_than_rejected() {
            let f = read("#remote=1&somethingelse=7&b2b=1");
            assert!(f.remote_only && f.b2b_only);
        }

        #[test]
        fn a_leading_hash_is_optional() {
            assert!(read("#remote=1").remote_only);
            assert!(read("remote=1").remote_only);
        }
    }
}
