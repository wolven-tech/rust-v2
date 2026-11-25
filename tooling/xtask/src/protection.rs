//! `main`'s branch protection, kept as data in `.github/branch-protection.json`.
//!
//! Every CI check is advisory until GitHub requires it: a red `supply chain`
//! could merge (finding 8 in `docs/review/003-whole-repo-review.md`). The
//! settings live in the repository so a change to them goes through review, and
//! this command reports where GitHub has drifted from the file.
//!
//! - `cargo xtask protection` — compare GitHub's settings with the file.
//! - `cargo xtask protection --apply` — write the file to GitHub, then compare.
//!   Needs admin on the repository.

use std::collections::BTreeSet;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;

pub const POLICY: &str = ".github/branch-protection.json";
pub const WORKFLOW: &str = ".github/workflows/ci.yml";
const BRANCH: &str = "main";
const GH_DEADLINE: Duration = Duration::from_secs(30);

type Fallible = Result<(), Box<dyn std::error::Error>>;

pub fn run(root: &Path, apply: bool) -> Fallible {
    let policy: Value = serde_json::from_str(&std::fs::read_to_string(root.join(POLICY))?)?;

    let missing = unknown_contexts(&policy, &std::fs::read_to_string(root.join(WORKFLOW))?);
    if !missing.is_empty() {
        return Err(
            format!("{POLICY} requires {missing:?}, which no job in {WORKFLOW} reports").into(),
        );
    }

    let repo = gh(&[
        "repo",
        "view",
        "--json",
        "nameWithOwner",
        "--jq",
        ".nameWithOwner",
    ])?
    .trim()
    .to_string();
    let endpoint = format!("repos/{repo}/branches/{BRANCH}/protection");

    if apply {
        let input = root.join(POLICY);
        gh(&[
            "api",
            "-X",
            "PUT",
            &endpoint,
            "--input",
            &input.to_string_lossy(),
        ])?;
        println!("applied {POLICY} to {repo}:{BRANCH}");
    }

    let live = match gh(&["api", &endpoint]) {
        Ok(body) => Some(serde_json::from_str::<Value>(&body)?),
        Err(error) if error.to_string().contains("Branch not protected") => None,
        Err(error) => return Err(error),
    };

    let problems = drift(&policy, live.as_ref());
    if !problems.is_empty() {
        eprintln!("{}", problems.join("\n"));
        return Err(format!(
            "{repo}:{BRANCH} does not match {POLICY}\n         \
             an admin runs `cargo xtask protection --apply`"
        )
        .into());
    }
    println!("{repo}:{BRANCH} matches {POLICY}");
    Ok(())
}

/// Required contexts that no job in the workflow reports. GitHub matches a
/// required check by the job's `name:`, so renaming a job without this file
/// blocks every pull request on a check that never arrives.
pub fn unknown_contexts(policy: &Value, workflow: &str) -> Vec<String> {
    let jobs = job_names(workflow);
    required_contexts(policy)
        .into_iter()
        .filter(|context| !jobs.contains(context))
        .collect()
}

/// The `name:` of each job: the first key under `jobs:` indented by four.
pub fn job_names(workflow: &str) -> BTreeSet<String> {
    let mut in_jobs = false;
    let mut names = BTreeSet::new();
    for line in workflow.lines() {
        if !line.starts_with(' ') && !line.trim().is_empty() {
            in_jobs = line.trim_end() == "jobs:";
            continue;
        }
        if let Some(name) = line.strip_prefix("    name:").filter(|_| in_jobs) {
            names.insert(name.trim().trim_matches(['"', '\'']).to_string());
        }
    }
    names
}

fn required_contexts(settings: &Value) -> BTreeSet<String> {
    settings["required_status_checks"]["contexts"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|context| context.as_str().map(str::to_string))
        .collect()
}

/// Where GitHub's settings (`None` when the branch is unprotected) differ from
/// the policy. GitHub answers booleans as `{"enabled": bool}`; the policy file
/// is in the request shape, where they are bare.
pub fn drift(policy: &Value, live: Option<&Value>) -> Vec<String> {
    let Some(live) = live else {
        return vec![format!("{BRANCH} is not protected")];
    };

    let mut problems = Vec::new();

    let want = required_contexts(policy);
    let have = required_contexts(live);
    if want != have {
        problems.push(format!("required checks: want {want:?}, have {have:?}"));
    }

    let strict = |settings: &Value| settings["required_status_checks"]["strict"].as_bool();
    if strict(policy) != strict(live) {
        problems.push(format!(
            "strict: want {:?}, have {:?}",
            strict(policy),
            strict(live)
        ));
    }

    for key in ["enforce_admins", "allow_force_pushes", "allow_deletions"] {
        let want = policy[key].as_bool();
        let have = live[key]["enabled"].as_bool();
        if want != have {
            problems.push(format!("{key}: want {want:?}, have {have:?}"));
        }
    }

    problems
}

/// `gh` with a deadline. A wedged network call is killed and reaped rather than
/// waited on.
fn gh(args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let mut child = Command::new("gh")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            format!("could not run gh ({e}); install the GitHub CLI and `gh auth login`")
        })?;

    // Drained on threads: a child blocked on a full pipe never exits.
    let drain = |pipe: Option<Box<dyn Read + Send>>| {
        std::thread::spawn(move || {
            let mut text = String::new();
            if let Some(mut pipe) = pipe {
                let _ = pipe.read_to_string(&mut text);
            }
            text
        })
    };
    let stdout = drain(
        child
            .stdout
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
    );
    let stderr = drain(
        child
            .stderr
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
    );

    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if started.elapsed() > GH_DEADLINE {
            child.kill()?;
            child.wait()?;
            return Err(format!("`gh {}` took over {GH_DEADLINE:?}", args.join(" ")).into());
        }
        std::thread::sleep(Duration::from_millis(100));
    };

    let stdout = stdout.join().unwrap_or_default();
    let stderr = stderr.join().unwrap_or_default();
    if !status.success() {
        return Err(format!(
            "`gh {}` failed: {} {}",
            args.join(" "),
            stdout.trim(),
            stderr.trim()
        )
        .into());
    }
    Ok(stdout)
}
