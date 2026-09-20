//! Does a company's careers link still answer, and if not, why.

use reqwest::Client;
use rv2_hiring::model::{LinkCheck, LinkStatus};

use crate::http;

const CAREER_PATHS: &[&str] = &[
    "/careers",
    "/careers/",
    "/jobs",
    "/about/careers",
    "/about-us/careers",
    "/company/careers",
    "/join-us",
    "/work-with-us",
    "/vacancies",
];
const CAREER_WORDS: &[&str] = &["career", "job", "vacanc", "join"];
const NOT_FOUND_WORDS: &[&str] = &[
    "page not found",
    "page cannot be found",
    "404 error",
    "error 404",
];

/// Many sites answer 200 for any path, so a candidate counts only if it lands
/// on a careers-looking URL and its page does not read as a not-found page.
pub async fn find_careers_page(client: &Client, website: &str) -> Option<String> {
    let site = website.trim().trim_end_matches('/');
    let host = site
        .strip_prefix("https://")
        .or_else(|| site.strip_prefix("http://"))?;
    let host = host.split('/').next()?.trim_start_matches("www.");

    let mut candidates: Vec<String> = CAREER_PATHS.iter().map(|p| format!("{site}{p}")).collect();
    candidates.push(format!("https://careers.{host}"));
    candidates.push(format!("https://jobs.{host}"));

    for url in candidates {
        let Ok((status, landed)) = http::head_or_get(client, &url).await else {
            continue;
        };
        if !status.is_success() {
            continue;
        }
        let landed_lower = landed.to_ascii_lowercase();
        if !CAREER_WORDS.iter().any(|w| landed_lower.contains(w)) {
            continue;
        }
        let Ok(body) = http::get_page(client, &landed).await else {
            continue;
        };
        let body = body.to_ascii_lowercase();
        let looks_missing = NOT_FOUND_WORDS.iter().any(|w| body.contains(w));
        let mentions_roles = CAREER_WORDS.iter().any(|w| body.contains(w));
        if mentions_roles && !looks_missing {
            return Some(landed);
        }
    }
    None
}

#[must_use]
pub fn classify_http(code: u16) -> LinkStatus {
    match code {
        200..=399 => LinkStatus::Live,
        // Careers sites behind bot protection answer these to scripted clients
        // while serving browsers normally, so they are not evidence the page
        // is gone.
        401 | 403 | 405 | 406 | 429 | 999 => LinkStatus::Blocked,
        500..=599 => LinkStatus::Unreachable,
        _ => LinkStatus::Broken,
    }
}

pub async fn check_url(client: &Client, url: &str, checked_on: &str) -> LinkCheck {
    let url = url.trim();
    let mut check = LinkCheck {
        status: LinkStatus::Missing,
        http_code: None,
        final_url: None,
        checked_on: checked_on.to_owned(),
    };
    if url.is_empty() {
        return check;
    }
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        check.status = LinkStatus::Broken;
        return check;
    }

    match http::head_or_get(client, url).await {
        Ok((status, landed)) => {
            check.status = classify_http(status.as_u16());
            check.http_code = Some(status.as_u16());
            check.final_url =
                (landed.trim_end_matches('/') != url.trim_end_matches('/')).then_some(landed);
        }
        Err(_) => check.status = LinkStatus::Unreachable,
    }
    check
}

#[cfg(test)]
mod tests {
    use super::classify_http;
    use rv2_hiring::model::LinkStatus;

    #[test]
    fn a_bot_protection_refusal_is_not_read_as_a_broken_link() {
        for code in [401, 403, 405, 406, 429, 999] {
            assert_eq!(classify_http(code), LinkStatus::Blocked, "code {code}");
        }
    }

    #[test]
    fn a_redirect_counts_as_live_because_it_answered() {
        assert_eq!(classify_http(301), LinkStatus::Live);
    }

    #[test]
    fn a_server_error_is_unreachable_and_a_client_error_is_broken() {
        assert_eq!(classify_http(503), LinkStatus::Unreachable);
        assert_eq!(classify_http(404), LinkStatus::Broken);
    }
}
