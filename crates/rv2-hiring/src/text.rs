//! Word-level text handling shared by every classifier.
//!
//! The whole module rests on [`words`]: lowercase, collapse every run of
//! non-alphanumerics to one space, pad both ends. A whole-word check is then
//! `contains(" word ")` and a whole-phrase check is the same, which is what
//! stops "architecture" matching a search for "architect" and
//! "trustworthy" matching a search for "rust".

/// Lowercases and collapses every run of non-alphanumerics to one space, padded
/// at both ends, so whole-word and whole-phrase checks become
/// `contains(" word ")`.
#[must_use]
pub fn words(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push(' ');
    let mut gap = false;
    for ch in text.chars().flat_map(char::to_lowercase) {
        if ch.is_alphanumeric() {
            if gap {
                out.push(' ');
                gap = false;
            }
            out.push(ch);
        } else if !out.ends_with(' ') {
            gap = true;
        }
    }
    out.push(' ');
    out
}

/// True when any needle appears as a whole word or whole phrase in a string
/// already passed through [`words`].
#[must_use]
pub fn has_any(folded: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| folded.contains(&format!(" {n} ")))
}

/// The first needle that appears as a whole word or phrase, for a classifier
/// that has to report which words decided it.
#[must_use]
pub fn first_match<'a>(folded: &str, needles: &[&'a str]) -> Option<&'a str> {
    needles
        .iter()
        .copied()
        .find(|n| folded.contains(&format!(" {n} ")))
}

#[must_use]
pub fn mentions_rust(text: &str) -> bool {
    words(text).contains(" rust ")
}

const UK_PLACES: &[&str] = &[
    "united kingdom",
    "uk",
    "gb",
    "gbr",
    "great britain",
    "england",
    "scotland",
    "wales",
    "northern ireland",
    "london",
    "manchester",
    "edinburgh",
    "glasgow",
    "cambridge",
    "oxford",
    "bristol",
    "leeds",
    "belfast",
    "birmingham",
    "reading",
    "newcastle",
    "nottingham",
    "sheffield",
    "cardiff",
    "brighton",
    "milton keynes",
    "guildford",
    "bath",
    "dundee",
    "aberdeen",
    "southampton",
    "liverpool",
    "leatherhead",
    "hatfield",
    "cheltenham",
    "derry",
];

/// Whether a job board's location string names somewhere in the UK.
///
/// Cambridge is the reason this is not a plain word check: Cambridge,
/// Massachusetts appears on the same boards as Cambridge, England, and a
/// register of UK roles that silently absorbs the American one is wrong in the
/// direction nobody notices.
#[must_use]
pub fn is_uk(location: &str) -> bool {
    let w = words(location);
    if !has_any(&w, UK_PLACES) {
        return false;
    }
    let explicit_uk = has_any(&w, &["united kingdom", "uk", "gb", "england"]);
    let cambridge_ma =
        w.contains(" cambridge ") && has_any(&w, &["ma", "massachusetts", "usa", "us"]);
    explicit_uk || !cambridge_ma
}

const ENGINEERING: &[&str] = &[
    "engineer",
    "engineering",
    "developer",
    "programmer",
    "sre",
    "devops",
    "architect",
    "site reliability",
    "technical staff",
    "data scientist",
    "research scientist",
    "security researcher",
    "firmware",
    "compiler",
];

/// Titles that read as engineering but sell, support or recruit for it.
const NOT_ENGINEERING: &[&str] = &[
    "sales",
    "presales",
    "pre sales",
    "account",
    "recruiter",
    "recruiting",
    "talent",
    "solutions engineer",
    "solutions architect",
    "services architect",
    "support engineer",
    "customer",
    "field engineer",
    "marketing",
    "partner",
    "partnerships",
    "brand",
];

#[must_use]
pub fn is_engineering(title: &str) -> bool {
    let w = words(title);
    has_any(&w, ENGINEERING) && !has_any(&w, NOT_ENGINEERING)
}

/// Strips legal suffixes so a job board's display name can be compared with a
/// company name.
#[must_use]
pub fn company_key(name: &str) -> String {
    const SUFFIXES: &[&str] = &[
        "plc",
        "group",
        "holdings",
        "ltd",
        "limited",
        "inc",
        "labs",
        "technologies",
        "ag",
    ];
    words(name)
        .split_whitespace()
        .filter(|w| !SUFFIXES.contains(w))
        .collect::<Vec<_>>()
        .join(" ")
}

fn decode_entities(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
}

/// Greenhouse returns job content as entity-escaped HTML, so entities are
/// decoded both before stripping tags and after.
#[must_use]
pub fn html_to_text(html: &str) -> String {
    let decoded = decode_entities(html);
    let mut out = String::with_capacity(decoded.len());
    let mut in_tag = false;
    for ch in decoded.chars() {
        match ch {
            '<' => in_tag = true,
            '>' if in_tag => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    decode_entities(&out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folding_pads_both_ends_so_a_single_word_is_a_whole_word_match() {
        assert_eq!(words("Rust"), " rust ");
    }

    #[test]
    fn folding_collapses_punctuation_runs_to_one_space() {
        assert_eq!(words("C#.NET, Java -- AWS"), " c net java aws ");
    }

    #[test]
    fn a_whole_word_check_does_not_match_inside_a_longer_word() {
        assert!(!mentions_rust("trustworthy systems"));
        assert!(mentions_rust("written in Rust"));
    }

    #[test]
    fn a_uk_location_is_recognised_from_a_city_alone() {
        assert!(is_uk("Nottingham"));
        assert!(is_uk("Cardiff, London or Remote (UK)"));
    }

    #[test]
    fn cambridge_massachusetts_is_not_a_uk_location() {
        assert!(is_uk("Cambridge"));
        assert!(!is_uk("Cambridge, MA"));
        assert!(is_uk("Cambridge, UK"));
    }

    #[test]
    fn a_solutions_architect_is_not_counted_as_engineering() {
        assert!(is_engineering("Staff Software Engineer"));
        assert!(!is_engineering("Solutions Architect"));
        assert!(!is_engineering("Technical Recruiter"));
    }

    #[test]
    fn a_company_key_ignores_legal_suffixes_so_board_names_compare() {
        assert_eq!(company_key("Autotrader Group"), "autotrader");
        assert_eq!(company_key("Sage Group plc"), "sage");
    }

    #[test]
    fn html_entities_are_decoded_on_both_sides_of_tag_stripping() {
        assert_eq!(
            html_to_text("&lt;p&gt;Rust &amp;amp; Go&lt;/p&gt;").trim(),
            "Rust & Go"
        );
    }

    #[test]
    fn first_match_reports_which_phrase_decided_a_classification() {
        let folded = words("This is a remote-first team with monthly onsite days");
        assert_eq!(
            first_match(&folded, &["remote first", "onsite"]),
            Some("remote first")
        );
    }
}
