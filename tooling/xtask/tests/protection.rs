#[allow(dead_code)]
#[path = "../src/protection.rs"]
mod protection;

use protection::{POLICY, WORKFLOW, drift, job_names, unknown_contexts};
use serde_json::{Value, json};

fn repo_file(path: &str) -> String {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path),
    )
    .unwrap()
}

fn policy() -> Value {
    serde_json::from_str(&repo_file(POLICY)).unwrap()
}

fn live(contexts: &[&str], admins: bool) -> Value {
    json!({
        "required_status_checks": { "strict": false, "contexts": contexts },
        "enforce_admins": { "enabled": admins },
        "allow_force_pushes": { "enabled": false },
        "allow_deletions": { "enabled": false }
    })
}

#[test]
fn every_required_check_is_a_job_the_workflow_reports() {
    assert_eq!(
        unknown_contexts(&policy(), &repo_file(WORKFLOW)),
        Vec::<String>::new()
    );
}

#[test]
fn a_renamed_job_leaves_its_required_check_unreported() {
    let workflow = repo_file(WORKFLOW).replace("name: supply chain", "name: deny");
    assert_eq!(unknown_contexts(&policy(), &workflow), ["supply chain"]);
}

#[test]
fn job_names_come_from_jobs_only_not_from_steps_or_the_workflow() {
    let workflow = "name: CI\njobs:\n  a:\n    name: first\n    steps:\n      - name: a step\n  b:\n    name: \"second\"\n";
    assert_eq!(
        job_names(workflow).into_iter().collect::<Vec<_>>(),
        ["first", "second"]
    );
}

#[test]
fn settings_matching_the_policy_have_no_drift() {
    let live = live(&["live Core", "check", "supply chain"], true);
    assert!(drift(&policy(), Some(&live)).is_empty());
}

#[test]
fn an_unprotected_branch_is_drift() {
    assert_eq!(drift(&policy(), None), ["main is not protected"]);
}

#[test]
fn a_missing_check_and_an_admin_bypass_are_each_reported() {
    let problems = drift(&policy(), Some(&live(&["check"], false)));
    assert_eq!(problems.len(), 2, "{problems:?}");
    assert!(problems[0].starts_with("required checks"));
    assert!(problems[1].starts_with("enforce_admins"));
}
