//! Standing questions, answered against whatever the register says today.
//!
//! A panel is a saved [`Filters`] plus a way of showing the answer. A dashboard
//! is a named list of them. Nothing here is a snapshot: a panel re-answers
//! itself every time the page loads, so "how many roles are remote and outside
//! IR35" stops being something somebody wrote down in a report and starts being
//! something the register says.
//!
//! ## Why a value rather than a store
//!
//! A dashboard is defined as a serialisable value first and given a transport
//! second, and that order is the whole design. The same `Dashboard` travels in
//! a URL fragment, sits in a file committed beside the register, and — if a
//! second person ever needs to edit one at runtime — would go into a datastore
//! without the type changing at all.
//!
//! The page has no backend today, and that is worth protecting: no loading
//! state, no fetch failure, no branch where the data is missing. Spending it
//! should take more than wanting to save a filter.
//!
//! ## The encoding stays readable
//!
//! Panels are written into the fragment with a `p0.` prefix over the existing
//! filter codec, so a dashboard link is still something a person can read and
//! edit in the address bar. Base64 or a compressed blob would be shorter and
//! would give that up, and the existing codec already refuses that trade for a
//! single filter.

use serde::{Deserialize, Serialize};

use crate::breakdown::GroupBy;
use crate::view::{Filters, Sort, hash};

/// How a panel shows its answer.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Show {
    /// One number, with the denominator that makes it readable.
    #[default]
    Count,
    /// The first few companies, in a stated order.
    List { top: usize, by: Sort },
    /// Every surviving company, as rows.
    Table,
    /// The pool counted along one axis.
    Breakdown { by: GroupBy },
}

impl Show {
    /// The fragment spelling, chosen so a reader can edit it by hand.
    #[must_use]
    pub fn key(self) -> String {
        match self {
            Self::Count => "count".to_string(),
            Self::List { top, by } => format!("list:{top}:{}", by.key()),
            Self::Table => "table".to_string(),
            Self::Breakdown { by } => format!("by:{}", by.key()),
        }
    }

    /// Parse a spelling, falling back to a count rather than refusing.
    ///
    /// A fragment is user-editable and a link may outlive the version that
    /// wrote it. A panel that renders a number is a worse answer than the one
    /// intended; a page that refuses to load is no answer at all.
    #[must_use]
    pub fn from_key(key: &str) -> Self {
        if key == "table" {
            return Self::Table;
        }
        if let Some(axis) = key.strip_prefix("by:") {
            return GroupBy::from_key(axis).map_or(Self::Count, |by| Self::Breakdown { by });
        }
        if let Some(rest) = key.strip_prefix("list:") {
            let mut parts = rest.splitn(2, ':');
            let top = parts.next().and_then(|n| n.parse().ok()).unwrap_or(5);
            let by = parts.next().map_or(Sort::Group, Sort::from_key);
            return Self::List { top, by };
        }
        Self::Count
    }
}

/// One standing question.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Panel {
    /// What the panel asks, as a reader would say it.
    pub title: String,
    pub filters: Filters,
    pub show: Show,
}

/// A named set of standing questions.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Dashboard {
    pub name: String,
    pub panels: Vec<Panel>,
}

impl Dashboard {
    /// Encode as a URL fragment.
    ///
    /// Each panel's filters go through the existing codec unchanged and are
    /// prefixed, so the two cannot drift: a filter that round-trips alone
    /// round-trips inside a dashboard.
    #[must_use]
    pub fn write(&self) -> String {
        let mut parts: Vec<String> = vec![format!("d={}", encode(&self.name))];
        for (i, panel) in self.panels.iter().enumerate() {
            parts.push(format!("p{i}.t={}", encode(&panel.title)));
            parts.push(format!("p{i}.s={}", encode(&panel.show.key())));
            let filters = hash::write(&panel.filters);
            for pair in filters.split('&').filter(|s| !s.is_empty()) {
                parts.push(format!("p{i}.{pair}"));
            }
        }
        parts.join("&")
    }

    /// Decode a fragment, keeping whatever parses.
    #[must_use]
    pub fn read(fragment: &str) -> Option<Self> {
        let fragment = fragment.trim_start_matches('#');
        let pairs: Vec<(&str, &str)> = fragment
            .split('&')
            .filter_map(|p| p.split_once('='))
            .collect();

        let name = pairs
            .iter()
            .find(|(k, _)| *k == "d")
            .map(|(_, v)| decode(v))?;

        let mut panels = Vec::new();
        for i in 0.. {
            let prefix = format!("p{i}.");
            let own: Vec<(&str, &str)> = pairs
                .iter()
                .filter_map(|(k, v)| k.strip_prefix(prefix.as_str()).map(|k| (k, *v)))
                .collect();
            if own.is_empty() {
                break;
            }
            let title = own
                .iter()
                .find(|(k, _)| *k == "t")
                .map_or_else(String::new, |(_, v)| decode(v));
            let show = own
                .iter()
                .find(|(k, _)| *k == "s")
                .map_or(Show::Count, |(_, v)| Show::from_key(&decode(v)));
            let filter_fragment = own
                .iter()
                .filter(|(k, _)| *k != "t" && *k != "s")
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("&");
            panels.push(Panel {
                title,
                filters: hash::read(&filter_fragment),
                show,
            });
        }

        Some(Self { name, panels })
    }
}

impl Dashboard {
    /// The dashboard the page opens on.
    ///
    /// Five panels, and each one had to answer the same question to be here:
    /// what would a reader do differently on seeing this? A panel whose answer
    /// changes nothing is decoration, however interesting the number is.
    ///
    /// The two breakdowns in the middle are the point. The register's finding —
    /// that the roles which are remote enough and the roles that carry a real
    /// mandate are not the same roles — is a sentence somebody has to trust
    /// when it is written in a report. Side by side, grouped from the same
    /// pool, it is something a reader sees in a second and can argue with.
    #[must_use]
    pub fn standing() -> Self {
        let takeable = Filters {
            remote_only: true,
            b2b_only: true,
            ..Filters::default()
        };

        Self {
            name: "This week".to_string(),
            panels: vec![
                Panel {
                    title: "Roles you could take today".to_string(),
                    filters: Filters {
                        sort: Sort::Mandate,
                        ..takeable.clone()
                    },
                    show: Show::List {
                        top: 8,
                        by: Sort::Mandate,
                    },
                },
                Panel {
                    title: "What you gave up by needing remote".to_string(),
                    filters: Filters {
                        b2b_only: true,
                        ..Filters::default()
                    },
                    show: Show::Breakdown {
                        by: GroupBy::WorkPattern,
                    },
                },
                Panel {
                    title: "What you gave up by needing a mandate".to_string(),
                    filters: takeable.clone(),
                    show: Show::Breakdown {
                        by: GroupBy::Mandate,
                    },
                },
                Panel {
                    title: "What you have applied to".to_string(),
                    filters: Filters {
                        applied_only: true,
                        ..Filters::default()
                    },
                    show: Show::List {
                        top: 10,
                        by: Sort::Name,
                    },
                },
                Panel {
                    title: "Can the register still see?".to_string(),
                    filters: Filters::default(),
                    show: Show::Breakdown {
                        by: GroupBy::LinkStatus,
                    },
                },
            ],
        }
    }
}

/// Percent-encode the characters that would otherwise end a field.
///
/// Deliberately minimal, matching the filter codec: a dashboard's titles are
/// English sentences, and escaping the whole of RFC 3986 would make the common
/// case unreadable in the address bar for no gain.
fn encode(value: &str) -> String {
    value
        .chars()
        .map(|c| match c {
            '%' => "%25".to_string(),
            '&' => "%26".to_string(),
            '=' => "%3D".to_string(),
            '#' => "%23".to_string(),
            '.' => "%2E".to_string(),
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

#[cfg(test)]
mod tests {
    use super::{Dashboard, Panel, Show};
    use crate::breakdown::GroupBy;
    use crate::view::{Filters, Sort};

    fn sample() -> Dashboard {
        Dashboard {
            name: "This week".to_string(),
            panels: vec![
                Panel {
                    title: "Takeable right now".to_string(),
                    filters: Filters {
                        remote_only: true,
                        b2b_only: true,
                        ..Filters::default()
                    },
                    show: Show::Count,
                },
                Panel {
                    title: "Where the register sits on working pattern".to_string(),
                    filters: Filters::default(),
                    show: Show::Breakdown {
                        by: GroupBy::WorkPattern,
                    },
                },
                Panel {
                    title: "Roles that build a department".to_string(),
                    filters: Filters {
                        mandate_only: true,
                        ..Filters::default()
                    },
                    show: Show::List {
                        top: 5,
                        by: Sort::Mandate,
                    },
                },
            ],
        }
    }

    #[test]
    fn a_dashboard_survives_a_round_trip_through_a_fragment() {
        let d = sample();
        let back = Dashboard::read(&d.write()).expect("a written dashboard must read back");
        assert_eq!(back, d);
    }

    #[test]
    fn a_panels_filters_round_trip_exactly_as_they_do_alone() {
        let d = sample();
        let back = Dashboard::read(&d.write()).unwrap();
        assert!(back.panels[0].filters.remote_only);
        assert!(back.panels[0].filters.b2b_only);
        assert!(back.panels[2].filters.mandate_only);
        assert!(!back.panels[1].filters.remote_only);
    }

    #[test]
    fn a_title_containing_a_separator_survives() {
        let d = Dashboard {
            name: "a&b=c".to_string(),
            panels: vec![Panel {
                title: "remote & outside IR35 = rare".to_string(),
                filters: Filters::default(),
                show: Show::Count,
            }],
        };
        let back = Dashboard::read(&d.write()).unwrap();
        assert_eq!(back.name, "a&b=c");
        assert_eq!(back.panels[0].title, "remote & outside IR35 = rare");
    }

    #[test]
    fn a_fragment_with_no_dashboard_in_it_is_not_one() {
        assert!(Dashboard::read("remote=1&b2b=1").is_none());
        assert!(Dashboard::read("").is_none());
    }

    #[test]
    fn an_unreadable_show_falls_back_to_a_count_rather_than_refusing_the_link() {
        assert_eq!(Show::from_key("nonsense"), Show::Count);
        assert_eq!(Show::from_key("by:not-an-axis"), Show::Count);
        assert_eq!(
            Show::from_key("by:work-pattern"),
            Show::Breakdown {
                by: GroupBy::WorkPattern
            }
        );
    }

    #[test]
    fn a_list_remembers_how_many_and_in_what_order() {
        assert_eq!(
            Show::from_key("list:8:mandate"),
            Show::List {
                top: 8,
                by: Sort::Mandate
            }
        );
        assert_eq!(Show::from_key("list:8:mandate").key(), "list:8:mandate");
    }

    #[test]
    fn a_dashboard_link_stays_short_enough_to_paste() {
        let written = sample().write();
        assert!(
            written.len() < 400,
            "three panels came to {} characters: {written}",
            written.len()
        );
    }

    #[test]
    fn the_standing_dashboard_survives_a_link() {
        let d = Dashboard::standing();
        let back = Dashboard::read(&d.write()).expect("the default must round-trip");
        assert_eq!(back, d);
    }

    #[test]
    fn the_standing_dashboard_fits_in_a_pasteable_link() {
        let written = Dashboard::standing().write();
        assert!(
            written.len() < 600,
            "five panels came to {} characters",
            written.len()
        );
    }

    #[test]
    fn the_two_counterfactual_panels_relax_different_filters() {
        let d = Dashboard::standing();
        let remote = d
            .panels
            .iter()
            .find(|p| p.title.contains("needing remote"))
            .expect("the remote counterfactual is present");
        let mandate = d
            .panels
            .iter()
            .find(|p| p.title.contains("needing a mandate"))
            .expect("the mandate counterfactual is present");

        assert!(
            !remote.filters.remote_only && remote.filters.b2b_only,
            "the remote counterfactual drops the remote filter and keeps B2B"
        );
        assert!(
            mandate.filters.remote_only
                && mandate.filters.b2b_only
                && !mandate.filters.mandate_only,
            "the mandate counterfactual keeps both essentials and drops the mandate"
        );
    }

    #[test]
    fn every_standing_panel_has_a_title_a_reader_would_recognise() {
        for panel in Dashboard::standing().panels {
            assert!(!panel.title.is_empty());
            assert!(
                !panel.title.contains("Filters") && !panel.title.contains("Show"),
                "{:?} names an implementation detail",
                panel.title
            );
        }
    }

    #[test]
    fn panels_stop_at_the_first_gap_rather_than_scanning_forever() {
        let d = Dashboard::read("d=x&p0.t=One&p0.s=count&p2.t=Orphan").unwrap();
        assert_eq!(d.panels.len(), 1, "p2 without p1 is not a third panel");
    }
}
