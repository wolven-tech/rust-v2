//! Reading the public feeds of Greenhouse, Lever, Ashby and `SmartRecruiters`.
//!
//! This is the only place with a job description in hand, which makes it the
//! only place that can classify a role properly. The page reads what this
//! writes; see [`crate::ats::summarise`].

use reqwest::Client;
use rv2_hiring::engagement::PostingSource;
use rv2_hiring::model::{AtsRef, Openings, Provider, Role};
use rv2_hiring::text::{company_key, html_to_text, is_engineering, is_uk, mentions_rust, words};
use serde_json::Value;

use crate::http;

/// How many roles a company's record keeps. The page lists them under the
/// entry, and a hundred of them is not a list anybody reads.
const MAX_ROLES: usize = 8;
const SMARTRECRUITERS_PAGE: usize = 100;
const SMARTRECRUITERS_MAX: usize = 1_000;

pub struct Posting {
    pub title: String,
    pub location: String,
    pub url: String,
    pub uk: bool,
    pub body: String,
}

/// Ordered so that embed URLs, whose slug sits in a query string, win over the
/// path forms they also contain.
const PATTERNS: &[(&str, Provider)] = &[
    (
        "boards.greenhouse.io/embed/job_board/js?for=",
        Provider::Greenhouse,
    ),
    (
        "boards.greenhouse.io/embed/job_board?for=",
        Provider::Greenhouse,
    ),
    (
        "job-boards.greenhouse.io/embed/job_board?for=",
        Provider::Greenhouse,
    ),
    (
        "boards.eu.greenhouse.io/embed/job_board?for=",
        Provider::GreenhouseEu,
    ),
    ("job-boards.eu.greenhouse.io/", Provider::GreenhouseEu),
    ("boards.eu.greenhouse.io/", Provider::GreenhouseEu),
    ("job-boards.greenhouse.io/", Provider::Greenhouse),
    ("boards.greenhouse.io/", Provider::Greenhouse),
    ("boards-api.greenhouse.io/v1/boards/", Provider::Greenhouse),
    ("jobs.eu.lever.co/", Provider::LeverEu),
    ("jobs.lever.co/", Provider::Lever),
    ("jobs.ashbyhq.com/", Provider::Ashby),
    ("api.ashbyhq.com/posting-api/job-board/", Provider::Ashby),
    ("careers.smartrecruiters.com/", Provider::Smartrecruiters),
    ("jobs.smartrecruiters.com/", Provider::Smartrecruiters),
];

const NOT_SLUGS: &[&str] = &["embed", "js", "v1", "api", "oneclick-ui", "job_board"];

#[must_use]
pub fn refs_in(text: &str) -> Vec<(Provider, String)> {
    let mut found: Vec<(Provider, String)> = Vec::new();
    let lower = text.to_ascii_lowercase();
    for (pattern, provider) in PATTERNS {
        let mut from = 0;
        while let Some(pos) = lower[from..].find(pattern) {
            let start = from + pos + pattern.len();
            let slug: String = text[start..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
                .collect();
            from = start;
            if slug.is_empty() || NOT_SLUGS.contains(&slug.to_ascii_lowercase().as_str()) {
                continue;
            }
            let entry = (*provider, slug);
            let already = found
                .iter()
                .any(|(p, s)| s.eq_ignore_ascii_case(&entry.1) && same_family(*p, entry.0));
            if !already {
                found.push(entry);
            }
        }
    }
    found
}

fn same_family(a: Provider, b: Provider) -> bool {
    matches!(
        (a, b),
        (
            Provider::Greenhouse | Provider::GreenhouseEu,
            Provider::Greenhouse | Provider::GreenhouseEu
        ) | (
            Provider::Lever | Provider::LeverEu,
            Provider::Lever | Provider::LeverEu
        )
    ) || a == b
}

fn str_at<'a>(v: &'a Value, path: &[&str]) -> &'a str {
    path.iter()
        .fold(v, |acc, key| &acc[*key])
        .as_str()
        .unwrap_or("")
}

async fn greenhouse(client: &Client, slug: &str, eu: bool) -> Result<Vec<Posting>, String> {
    let host = if eu {
        "boards-api.eu.greenhouse.io"
    } else {
        "boards-api.greenhouse.io"
    };
    let json = http::get_json(
        client,
        &format!("https://{host}/v1/boards/{slug}/jobs?content=true"),
    )
    .await?;
    let jobs = json["jobs"]
        .as_array()
        .ok_or("no jobs array in Greenhouse response")?;
    Ok(jobs
        .iter()
        .map(|j| {
            let stated = str_at(j, &["location", "name"]).trim().to_owned();
            let offices: Vec<(String, String)> = j["offices"]
                .as_array()
                .map(|os| {
                    os.iter()
                        .map(|o| {
                            (
                                str_at(o, &["name"]).trim().to_owned(),
                                str_at(o, &["location"]).trim().to_owned(),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            // Some boards set location to "Hybrid" or "Distributed" and name
            // the city only in the office list, so offices count towards UK
            // detection but are shown only when the stated location says
            // nothing about place.
            let uk = is_uk(&stated)
                || offices
                    .iter()
                    .any(|(name, place)| is_uk(name) || is_uk(place));
            let generic = stated.is_empty()
                || matches!(
                    words(&stated).trim(),
                    "hybrid" | "remote" | "distributed" | "flexible"
                );
            let location = if generic {
                let mut names: Vec<&str> = offices
                    .iter()
                    .map(|(name, _)| name.as_str())
                    .filter(|n| !n.is_empty())
                    .collect();
                names.sort_unstable();
                names.dedup();
                // The stated word is kept alongside the offices: "Hybrid"
                // is what the work-pattern classifier needs and the office
                // list is what tells it where.
                if stated.is_empty() {
                    names.join(", ")
                } else {
                    format!("{} ({stated})", names.join(", "))
                }
            } else {
                stated
            };
            Posting {
                title: str_at(j, &["title"]).to_owned(),
                uk,
                location,
                url: str_at(j, &["absolute_url"]).to_owned(),
                body: html_to_text(str_at(j, &["content"])),
            }
        })
        .collect())
}

async fn lever(client: &Client, slug: &str, eu: bool) -> Result<Vec<Posting>, String> {
    let host = if eu {
        "api.eu.lever.co"
    } else {
        "api.lever.co"
    };
    let json = http::get_json(
        client,
        &format!("https://{host}/v0/postings/{slug}?mode=json"),
    )
    .await?;
    let jobs = json
        .as_array()
        .ok_or("no postings array in Lever response")?;
    Ok(jobs
        .iter()
        .map(|j| {
            let mut places = vec![str_at(j, &["categories", "location"]).to_owned()];
            if let Some(all) = j["categories"]["allLocations"].as_array() {
                places.extend(all.iter().filter_map(Value::as_str).map(str::to_owned));
            }
            places.retain(|p| !p.is_empty());
            places.dedup();
            let lists: String = j["lists"]
                .as_array()
                .map(|ls| {
                    ls.iter()
                        .map(|l| html_to_text(str_at(l, &["content"])))
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .unwrap_or_default();
            Posting {
                title: str_at(j, &["text"]).to_owned(),
                uk: places.iter().any(|p| is_uk(p))
                    || str_at(j, &["country"]).eq_ignore_ascii_case("GB"),
                location: places.join("; "),
                url: str_at(j, &["hostedUrl"]).to_owned(),
                body: format!(
                    "{} {} {lists}",
                    str_at(j, &["descriptionPlain"]),
                    str_at(j, &["additionalPlain"])
                ),
            }
        })
        .collect())
}

async fn ashby(client: &Client, slug: &str) -> Result<Vec<Posting>, String> {
    let json = http::get_json(
        client,
        &format!("https://api.ashbyhq.com/posting-api/job-board/{slug}?includeCompensation=false"),
    )
    .await?;
    let jobs = json["jobs"]
        .as_array()
        .ok_or("no jobs array in Ashby response")?;
    Ok(jobs
        .iter()
        .map(|j| {
            let mut places = vec![str_at(j, &["location"]).to_owned()];
            if let Some(extra) = j["secondaryLocations"].as_array() {
                places.extend(extra.iter().map(|l| str_at(l, &["location"]).to_owned()));
            }
            places.retain(|p| !p.is_empty());
            let country = str_at(j, &["address", "postalAddress", "addressCountry"]);
            Posting {
                title: str_at(j, &["title"]).to_owned(),
                uk: places.iter().any(|p| is_uk(p)) || is_uk(country),
                location: places.join("; "),
                url: str_at(j, &["jobUrl"]).to_owned(),
                body: str_at(j, &["descriptionPlain"]).to_owned(),
            }
        })
        .collect())
}

/// The postings list carries no descriptions, so both Rust detection and the
/// three classifiers see titles and locations only here.
async fn smartrecruiters(client: &Client, slug: &str) -> Result<Vec<Posting>, String> {
    let mut postings = Vec::new();
    let mut offset = 0;
    loop {
        let url = format!(
            "https://api.smartrecruiters.com/v1/companies/{slug}/postings?limit={SMARTRECRUITERS_PAGE}&offset={offset}"
        );
        let json = http::get_json(client, &url).await?;
        let page = json["content"]
            .as_array()
            .ok_or("no content array in SmartRecruiters response")?;
        for j in page {
            let city = str_at(j, &["location", "city"]);
            let country = str_at(j, &["location", "country"]);
            let id = str_at(j, &["id"]);
            let remote = j["location"]["remote"].as_bool().unwrap_or(false);
            let mut place = [city, &country.to_ascii_uppercase()]
                .iter()
                .filter(|s| !s.is_empty())
                .copied()
                .collect::<Vec<_>>()
                .join(", ");
            if remote {
                place.push_str(" (Remote)");
            }
            postings.push(Posting {
                title: str_at(j, &["name"]).to_owned(),
                location: place,
                uk: country.eq_ignore_ascii_case("gb") || is_uk(city),
                url: format!("https://jobs.smartrecruiters.com/{slug}/{id}"),
                body: String::new(),
            });
        }
        let total = usize::try_from(json["totalFound"].as_u64().unwrap_or(0)).unwrap_or(usize::MAX);
        offset += page.len();
        if page.is_empty() || offset >= total || offset >= SMARTRECRUITERS_MAX {
            break;
        }
    }
    Ok(postings)
}

pub async fn fetch(
    client: &Client,
    provider: Provider,
    slug: &str,
) -> Result<Vec<Posting>, String> {
    match provider {
        Provider::Greenhouse => greenhouse(client, slug, false).await,
        Provider::GreenhouseEu => greenhouse(client, slug, true).await,
        Provider::Lever => lever(client, slug, false).await,
        Provider::LeverEu => lever(client, slug, true).await,
        Provider::Ashby => ashby(client, slug).await,
        Provider::Smartrecruiters => smartrecruiters(client, slug).await,
        Provider::Workable => Err("no reader for Workable boards".to_string()),
    }
}

/// Turn a board read into a company's record, classifying every role kept.
///
/// This is where the description is still in hand, and it is the reason the
/// page can filter on work pattern, engagement and mandate at all.
#[must_use]
pub fn summarise(postings: &[Posting], source: PostingSource, checked_on: &str) -> Openings {
    let mut roles: Vec<Role> = Vec::new();
    let mut openings = Openings {
        total: postings.len(),
        checked_on: checked_on.to_owned(),
        ..Openings::default()
    };

    for p in postings {
        let rust_in_title = mentions_rust(&p.title);
        let rust = rust_in_title || mentions_rust(&p.body);
        if rust {
            openings.rust += 1;
        }
        if !p.uk {
            continue;
        }
        openings.uk += 1;
        if rust {
            openings.uk_rust += 1;
        }
        if rust_in_title {
            openings.uk_rust_in_title += 1;
        }
        if is_engineering(&p.title) {
            openings.uk_engineering += 1;
            let mut role = Role {
                title: p.title.trim().to_owned(),
                location: p.location.clone(),
                url: p.url.clone(),
                rust,
                rust_in_title,
                ..Role::default()
            };
            role.classify(&p.body, source);
            roles.push(role);
        }
    }

    // A role that clears both essential filters is the rarest thing in the
    // register, so it is kept in preference to a Rust mention when the list is
    // truncated.
    roles.sort_by(|a, b| {
        b.clears_essential_filters()
            .cmp(&a.clears_essential_filters())
            .then_with(|| (b.rust_in_title, b.rust).cmp(&(a.rust_in_title, a.rust)))
            .then_with(|| a.title.cmp(&b.title))
    });
    roles.truncate(MAX_ROLES);
    openings.roles = roles;
    openings
}

async fn greenhouse_board_name(client: &Client, slug: &str) -> Option<String> {
    let json = http::get_json(
        client,
        &format!("https://boards-api.greenhouse.io/v1/boards/{slug}"),
    )
    .await
    .ok()?;
    json["name"].as_str().map(str::to_owned)
}

fn slug_candidates(name: &str) -> Vec<String> {
    let key = company_key(name);
    let parts: Vec<&str> = key.split_whitespace().collect();
    let full = words(name).split_whitespace().collect::<Vec<_>>().concat();
    let mut slugs = vec![parts.concat(), parts.join("-"), full];
    slugs.retain(|s| s.len() >= 3);
    slugs.dedup();
    slugs
}

/// Attach a board to a company, in a fixed order of evidence.
///
/// A slug guessed from the company name is accepted only when Greenhouse
/// reports a board whose name is the same company name once legal suffixes are
/// removed; a looser rule would take "Wise Worksite Field Sales" for Wise.
/// Lever and Ashby expose no board name to check a guess against, so they are
/// only taken from URLs that name them.
pub async fn discover(
    client: &Client,
    name: &str,
    careers_url: &str,
    evidence: &str,
) -> Option<AtsRef> {
    for (provider, slug) in refs_in(careers_url) {
        if fetch(client, provider, &slug).await.is_ok() {
            return Some(AtsRef {
                provider,
                slug,
                source: "careers-url".to_string(),
            });
        }
    }
    if !careers_url.is_empty()
        && let Ok(page) = http::get_page(client, careers_url).await
    {
        for (provider, slug) in refs_in(&page) {
            if fetch(client, provider, &slug).await.is_ok() {
                return Some(AtsRef {
                    provider,
                    slug,
                    source: "careers-page".to_string(),
                });
            }
        }
    }
    for (provider, slug) in refs_in(evidence) {
        if fetch(client, provider, &slug).await.is_ok() {
            return Some(AtsRef {
                provider,
                slug,
                source: "research-evidence".to_string(),
            });
        }
    }

    let key = company_key(name);
    for slug in slug_candidates(name) {
        let Some(board) = greenhouse_board_name(client, &slug).await else {
            continue;
        };
        if !key.is_empty()
            && company_key(&board) == key
            && fetch(client, Provider::Greenhouse, &slug).await.is_ok()
        {
            return Some(AtsRef {
                provider: Provider::Greenhouse,
                slug,
                source: "greenhouse-name-match".to_string(),
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{refs_in, slug_candidates};
    use rv2_hiring::model::Provider;

    #[test]
    fn an_embed_url_yields_the_slug_from_its_query_string() {
        let found = refs_in("https://boards.greenhouse.io/embed/job_board?for=monzo");
        assert_eq!(found[0], (Provider::Greenhouse, "monzo".to_string()));
    }

    #[test]
    fn a_european_board_host_is_read_as_the_european_provider() {
        let found = refs_in("https://jobs.eu.lever.co/example");
        assert_eq!(found[0].0, Provider::LeverEu);
    }

    #[test]
    fn a_path_segment_that_is_only_plumbing_is_not_taken_as_a_slug() {
        assert!(refs_in("https://boards.greenhouse.io/embed/").is_empty());
    }

    #[test]
    fn slug_guesses_drop_legal_suffixes_and_anything_too_short() {
        let slugs = slug_candidates("Autotrader Group plc");
        assert!(slugs.contains(&"autotrader".to_string()));
        assert!(slugs.iter().all(|s| s.len() >= 3));
    }
}
