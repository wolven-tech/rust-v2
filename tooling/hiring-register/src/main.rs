//! Keeps `crates/rv2-hiring/data/companies.json` current.
//!
//! ```text
//! hiring-register check                 re-check careers links and re-read every job board
//! hiring-register discover              also probe for boards and missing careers pages
//! hiring-register ingest <board.json>   classify a captured contract-board listing file
//! hiring-register classify              fill in any role with no stored verdict
//! ```
//!
//! **This is where a job description is still in hand**, which makes it the
//! only place that can classify a role properly. The page reads the verdicts
//! this writes and never recomputes them, because a title alone cannot say how
//! many days are in the office or whether a role builds a team.
//!
//! **Server-side and deliberately not a background job.** `rv2-jobs` runs every
//! registered job on every instance with no leasing, and nothing survives a
//! restart; a feed poller whose second concurrent execution rewrites the
//! register is exactly the defect that crate's own docs warn about.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

use reqwest::Client;
use rv2_hiring::engagement::PostingSource;
use rv2_hiring::model::{AtsRef, Company, LinkCheck, LinkStatus, Openings, Register, Role};
use serde::Deserialize;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

mod ats;
mod http;
mod links;

const REGISTER: &str = "crates/rv2-hiring/data/companies.json";

type Fallible = Result<(), Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("help");

    let result = match command {
        "check" => check(false).await,
        "discover" => check(true).await,
        "classify" => classify(),
        "ingest" => match args.get(1) {
            Some(path) => ingest(Path::new(path)),
            None => Err("ingest needs a path to a captured board file".into()),
        },
        _ => {
            eprintln!("{}", usage());
            return ExitCode::from(2);
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("failed: {e}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> &'static str {
    "hiring-register check | discover | classify | ingest <board.json>"
}

fn workspace_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut dir = std::env::current_dir()?;
    loop {
        if dir.join(REGISTER).exists() {
            return Ok(dir);
        }
        if !dir.pop() {
            return Err(format!("no {REGISTER} above the working directory").into());
        }
    }
}

fn load() -> Result<(PathBuf, Register), Box<dyn std::error::Error>> {
    let path = workspace_root()?.join(REGISTER);
    let raw = std::fs::read_to_string(&path)?;
    Ok((path, serde_json::from_str(&raw)?))
}

/// Write the register back with a trailing newline, so a re-run does not show
/// up as a one-line diff in every review.
fn save(path: &Path, register: &Register) -> Fallible {
    let mut json = serde_json::to_string_pretty(register)?;
    json.push('\n');
    std::fs::write(path, json)?;
    Ok(())
}

/// Today, as the register spells dates.
///
/// Read from the clock only here. Everything below takes the date as an
/// argument, which is what lets the classifiers stay pure functions of their
/// inputs and be tested without freezing time.
fn today() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = i64::try_from(secs / 86_400).unwrap_or(0);
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Howard Hinnant's `civil_from_days`, so a date costs no dependency.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = u32::try_from(doy - (153 * mp + 2) / 5 + 1).unwrap_or(1);
    let m = u32::try_from(if mp < 10 { mp + 3 } else { mp - 9 }).unwrap_or(1);
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn classify() -> Fallible {
    let (path, register) = load()?;
    let before = unclassified(&register);
    let register = register.classified();
    let after = unclassified(&register);
    save(&path, &register)?;
    println!(
        "classified {}, {after} still without a verdict",
        before - after
    );
    Ok(())
}

fn unclassified(register: &Register) -> usize {
    register
        .companies
        .iter()
        .flat_map(|c| c.openings.iter().flat_map(|o| o.roles.iter()))
        .filter(|r| r.mandate.is_none())
        .count()
}

/// A listing captured from a contract board that publishes no feed.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CapturedListing {
    title: String,
    location: String,
    url: String,
    #[serde(default)]
    description: String,
}

/// Classify a captured contract-board file into the register.
///
/// A board with no API still has descriptions, and a description is the only
/// thing that can decide a mandate. Capturing the listings and classifying them
/// here keeps one classifier rather than growing a second that reads the page
/// at render time and disagrees with this one.
fn ingest(file: &Path) -> Fallible {
    let raw = std::fs::read_to_string(file)?;
    let listings: Vec<CapturedListing> = serde_json::from_str(&raw)?;
    let (path, mut register) = load()?;

    let company = register
        .companies
        .iter_mut()
        .find(|c| c.posting_source() == PostingSource::OutsideIr35Board)
        .ok_or("no company in the register is marked as an outside-IR35 board")?;

    let mut roles: Vec<Role> = Vec::new();
    let mut openings = Openings {
        total: listings.len(),
        uk: listings.len(),
        checked_on: today(),
        ..Openings::default()
    };

    for listing in &listings {
        let rust_in_title = rv2_hiring::text::mentions_rust(&listing.title);
        let rust = rust_in_title || rv2_hiring::text::mentions_rust(&listing.description);
        if rust {
            openings.rust += 1;
            openings.uk_rust += 1;
        }
        if rust_in_title {
            openings.uk_rust_in_title += 1;
        }
        if rv2_hiring::text::is_engineering(&listing.title) {
            openings.uk_engineering += 1;
        }
        let mut role = Role {
            title: listing.title.clone(),
            location: listing.location.clone(),
            url: listing.url.clone(),
            rust,
            rust_in_title,
            ..Role::default()
        };
        role.classify(&listing.description, PostingSource::OutsideIr35Board);
        roles.push(role);
    }

    let remote = roles
        .iter()
        .filter(|r| r.work_pattern.at_most_quarter_office())
        .count();
    roles.sort_by(|a, b| {
        b.clears_essential_filters()
            .cmp(&a.clears_essential_filters())
            .then_with(|| a.title.cmp(&b.title))
    });
    openings.roles = roles;
    company.openings = Some(openings);

    save(&path, &register)?;
    println!(
        "ingested {} listings, {remote} of them inside the office ceiling",
        listings.len()
    );
    Ok(())
}

/// What one company's re-read produced.
struct Refreshed {
    index: usize,
    ats: Option<AtsRef>,
    careers_url: Option<String>,
    link: Option<LinkCheck>,
    openings: Option<Openings>,
    read_failed: bool,
}

/// Re-read every board and re-check every careers link.
///
/// Concurrent, at [`http::CONCURRENCY`], because the register holds over a
/// hundred companies and a sequential pass over that many network round-trips
/// does not finish inside the five minutes any step here is allowed.
async fn check(probe_for_missing: bool) -> Fallible {
    let (path, mut register) = load()?;
    let client = http::client();
    let checked_on = today();
    let permits = Arc::new(Semaphore::new(http::CONCURRENCY));

    let mut work = JoinSet::new();
    for (index, company) in register.companies.iter().enumerate() {
        // The contract board has no feed to read and its listings are ingested
        // from a capture, so re-reading here would erase them.
        if company.posting_source() == PostingSource::OutsideIr35Board {
            continue;
        }
        let job = CompanyJob {
            index,
            name: company.name.clone(),
            careers_url: company.careers_url.clone(),
            website: company
                .rest
                .get("website")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            evidence: company.evidence.clone(),
            ats: company.ats.clone(),
            source: company.posting_source(),
            link_status: company.link_status(),
        };
        let client = client.clone();
        let permits = Arc::clone(&permits);
        let checked_on = checked_on.clone();
        work.spawn(async move {
            let _permit = permits.acquire().await;
            refresh(&client, job, probe_for_missing, &checked_on).await
        });
    }

    let mut results = Vec::new();
    while let Some(done) = work.join_next().await {
        match done {
            Ok(refreshed) => results.push(refreshed),
            Err(e) => eprintln!("a company task did not finish: {e}"),
        }
    }

    for r in results {
        apply(&mut register.companies[r.index], r, &checked_on);
    }

    register.openings_checked_on = Some(checked_on.clone());
    register.links_checked_on = Some(checked_on);
    save(&path, &register)?;

    let remote = register
        .companies
        .iter()
        .flat_map(|c| c.openings.iter().flat_map(|o| o.roles.iter()))
        .filter(|r| r.clears_essential_filters())
        .count();
    println!("{remote} roles clear both essential filters");
    Ok(())
}

struct CompanyJob {
    index: usize,
    name: String,
    careers_url: String,
    website: String,
    evidence: String,
    ats: Option<AtsRef>,
    source: PostingSource,
    link_status: LinkStatus,
}

async fn refresh(
    client: &Client,
    job: CompanyJob,
    probe_for_missing: bool,
    checked_on: &str,
) -> Refreshed {
    let mut out = Refreshed {
        index: job.index,
        ats: None,
        careers_url: None,
        link: None,
        openings: None,
        read_failed: false,
    };

    let mut careers_url = job.careers_url.clone();

    if probe_for_missing
        && (careers_url.is_empty()
            || matches!(job.link_status, LinkStatus::Broken | LinkStatus::Missing))
        && !job.website.is_empty()
        && let Some(found) = links::find_careers_page(client, &job.website).await
    {
        println!("{}: found a careers page at {found}", job.name);
        careers_url.clone_from(&found);
        out.careers_url = Some(found);
    }

    let ats = match job.ats.clone() {
        Some(existing) => Some(existing),
        None if probe_for_missing => {
            let found = ats::discover(client, &job.name, &careers_url, &job.evidence).await;
            if let Some(found) = found.clone() {
                println!("{}: found a {:?} board", job.name, found.provider);
                out.ats = Some(found);
            }
            found
        }
        None => None,
    };

    if !careers_url.is_empty() {
        out.link = Some(links::check_url(client, &careers_url, checked_on).await);
    }

    if let Some(ats) = ats {
        match ats::fetch(client, ats.provider, &ats.slug).await {
            Ok(postings) => {
                out.openings = Some(ats::summarise(&postings, job.source, checked_on));
            }
            Err(e) => {
                eprintln!("{}: {e}", job.name);
                out.read_failed = true;
            }
        }
    }

    out
}

fn apply(company: &mut Company, r: Refreshed, checked_on: &str) {
    if let Some(url) = r.careers_url {
        company.careers_url = url;
    }
    if let Some(ats) = r.ats {
        company.ats = Some(ats);
    }
    if let Some(link) = r.link {
        company.link = Some(link);
    }
    if let Some(openings) = r.openings {
        company.openings = Some(openings);
    } else if r.read_failed
        && let Some(existing) = company.openings.as_mut()
    {
        // Keeping the previous counts and saying when the read failed beats
        // zeroing them: a zero reads as "they stopped hiring", which is a
        // different and much more actionable claim than "we could not look".
        existing.last_read_failed = Some(checked_on.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::civil_from_days;
    use rv2_hiring::work_pattern;

    #[test]
    fn the_epoch_converts_to_the_first_of_january_nineteen_seventy() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
    }

    #[test]
    fn a_known_later_date_round_trips() {
        assert_eq!(civil_from_days(20_351), (2025, 9, 20));
    }

    #[test]
    fn a_board_that_states_hybrid_beside_its_offices_still_fails_the_ceiling() {
        let v = work_pattern::classify("London, Manchester (Hybrid)", "");
        assert!(!v.pattern.at_most_quarter_office());
    }
}
