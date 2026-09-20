//! Every route off this page, and the guard on all of them.
//!
//! Records here come from web research, browser bookmark exports and job-board
//! feeds — three sources that are edited by hand or served by someone else. A
//! URL from any of them is untrusted input, so nothing becomes an `href`
//! without passing [`safe_url`] first.

/// The URL if it is one a page may link to, otherwise nothing.
///
/// Only `http` and `https`. The schemes this refuses are the point: `javascript:`
/// executes, `data:` can carry a whole document, and `file:` reads the reader's
/// disk. None of them can reach an `href` here, and a record carrying one shows
/// as no link rather than as a link that does something else.
#[must_use]
pub fn safe_url(url: &str) -> Option<&str> {
    let trimmed = url.trim();
    let lower = trimmed.to_ascii_lowercase();
    (lower.starts_with("http://") || lower.starts_with("https://")).then_some(trimmed)
}

/// Percent-encode a query value.
///
/// Hand-rolled because this crate is WASM-safe and takes no dependency for six
/// lines. Everything outside the unreserved set is escaped, which is stricter
/// than a URL needs and never wrong.
fn encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char);
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// A LinkedIn job search for this company in the UK.
///
/// The company name is quoted so the search does not scatter across every
/// posting containing one of its words. `rust` narrows the keywords, because a
/// reader who has switched the Rust filter on wants the Rust roles when they
/// follow the link rather than the whole engineering list.
#[must_use]
pub fn linkedin_search(company: &str, rust: bool) -> String {
    let what = if rust { "rust" } else { "software engineer" };
    format!(
        "https://www.linkedin.com/jobs/search/?keywords={}&location={}",
        encode(&format!("\"{company}\" {what}")),
        encode("United Kingdom")
    )
}

/// A web search for a company's careers site.
///
/// Offered where the careers link is broken or was never found, so the entry
/// still gives the reader somewhere to go. An entry that can only say "no link"
/// is a dead end.
#[must_use]
pub fn careers_search(company: &str) -> String {
    format!(
        "https://www.google.com/search?q={}",
        encode(&format!("{company} careers software engineer UK"))
    )
}

#[cfg(test)]
mod tests {
    use super::{careers_search, linkedin_search, safe_url};

    #[test]
    fn an_ordinary_web_url_passes() {
        assert_eq!(
            safe_url("https://example.com/jobs"),
            Some("https://example.com/jobs")
        );
        assert_eq!(safe_url("http://example.com"), Some("http://example.com"));
    }

    #[test]
    fn a_scheme_that_executes_or_reads_the_disk_is_refused() {
        for hostile in [
            "javascript:alert(1)",
            "JavaScript:alert(1)",
            "data:text/html,<script>alert(1)</script>",
            "file:///etc/passwd",
            "vbscript:msgbox(1)",
        ] {
            assert_eq!(safe_url(hostile), None, "{hostile} must not become a link");
        }
    }

    #[test]
    fn a_scheme_is_matched_without_regard_to_case_or_padding() {
        assert_eq!(
            safe_url("  HTTPS://example.com  "),
            Some("HTTPS://example.com")
        );
    }

    #[test]
    fn an_empty_or_relative_url_is_refused_rather_than_guessed_at() {
        assert_eq!(safe_url(""), None);
        assert_eq!(safe_url("   "), None);
        assert_eq!(safe_url("/careers"), None);
        assert_eq!(safe_url("example.com"), None);
    }

    #[test]
    fn a_company_name_is_quoted_and_escaped_in_a_linkedin_search() {
        let url = linkedin_search("Auto Trader", false);
        assert!(url.contains("%22Auto+Trader%22"));
        assert!(url.contains("United+Kingdom"));
    }

    #[test]
    fn the_rust_filter_changes_what_the_linkedin_link_searches_for() {
        assert!(linkedin_search("Monzo", true).contains("rust"));
        assert!(linkedin_search("Monzo", false).contains("software+engineer"));
    }

    #[test]
    fn an_ampersand_in_a_company_name_cannot_add_a_query_parameter() {
        let url = careers_search("Marks & Spencer&foo=bar");
        assert!(!url.contains("&foo=bar"), "got {url}");
        assert!(url.contains("%26"));
    }
}
