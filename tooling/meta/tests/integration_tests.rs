use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

// Issue #7: Projects sharing a path should have independent dev task detection
#[test]
fn test_doctor_shared_path_independent_dev_task() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("apps/trainee-app");
    fs::create_dir_all(&project_path).unwrap();
    fs::write(
        project_path.join("Cargo.toml"),
        "[package]\nname = \"trainee-app\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::create_dir_all(project_path.join("src")).unwrap();
    fs::write(project_path.join("src/main.rs"), "fn main() {}").unwrap();

    // Create meta.toml where trainee-android shares path but has NO dev task
    fs::write(
        temp_dir.path().join("meta.toml"),
        r#"version = "1"

[workspace]
name = "Test"
root = "."

[tools.trunk]
enabled = true
command = "trunk"

[tools.cargo]
enabled = true
command = "cargo"

[projects.trainee-app]
type = "rust"
path = "apps/trainee-app"

[projects.trainee-app.tasks]
dev = { tool = "trunk", command = "serve --port 3853" }

[projects.trainee-android]
type = "rust"
path = "apps/trainee-app"

[projects.trainee-android.tasks]
build = { tool = "cargo", command = "build" }
"#,
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("meta");
    cmd.current_dir(&temp_dir);
    cmd.arg("doctor");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // trainee-app should show "dev task configured"
    // Find the trainee-app section and verify dev task is reported
    assert!(
        stdout.contains("trainee-app"),
        "Should mention trainee-app in output"
    );

    // trainee-android should NOT have "dev task configured" after its name
    // Split output into per-project blocks and check
    let lines: Vec<&str> = stdout.lines().collect();
    let mut in_android_section = false;
    for line in &lines {
        if line.contains("trainee-android") && line.contains("✓") {
            in_android_section = true;
            continue;
        }
        if in_android_section {
            // The next line after trainee-android's project line
            // should NOT say "dev task configured"
            if line.trim().starts_with("•")
                || line.trim().starts_with("✓")
                || line.trim().starts_with("✗")
            {
                // We've moved past trainee-android's details
                break;
            }
            assert!(
                !line.contains("dev task configured"),
                "trainee-android should NOT report 'dev task configured', but got: {}",
                line
            );
            break;
        }
    }
}

// Issue #2: status should only show projects with dev task
#[test]
fn test_status_hides_projects_without_dev_task() {
    let temp_dir = TempDir::new().unwrap();
    let api_path = temp_dir.path().join("apps/api");
    fs::create_dir_all(api_path.join("src")).unwrap();
    fs::write(api_path.join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(
        api_path.join("Cargo.toml"),
        "[package]\nname = \"api\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    let shared_path = temp_dir.path().join("crates/shared");
    fs::create_dir_all(shared_path.join("src")).unwrap();
    fs::write(shared_path.join("src/lib.rs"), "pub fn hello() {}").unwrap();
    fs::write(
        shared_path.join("Cargo.toml"),
        "[package]\nname = \"shared\"\nversion = \"0.1.0\"\n[lib]\nname = \"shared\"\n",
    )
    .unwrap();

    fs::write(
        temp_dir.path().join("meta.toml"),
        r#"version = "1"

[workspace]
name = "Test"
root = "."

[tools.bacon]
enabled = true
command = "bacon"

[tools.cargo]
enabled = true
command = "cargo"

[projects.api]
type = "rust"
path = "apps/api"

[projects.api.tasks]
dev = { tool = "bacon", command = "run-long" }

[projects.shared]
type = "rust"
path = "crates/shared"

[projects.shared.tasks]
build = { tool = "cargo", command = "build" }
"#,
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("meta");
    cmd.current_dir(&temp_dir);
    cmd.args(["status"]);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // "shared" should NOT appear in the Running Processes table
    // (it has no dev task)
    let processes_section: Vec<&str> = stdout
        .lines()
        .skip_while(|l| !l.contains("## Running Processes"))
        .take_while(|l| !l.contains("## Recent Events"))
        .collect();

    let shared_in_processes = processes_section.iter().any(|l| l.starts_with("shared"));
    assert!(
        !shared_in_processes,
        "Library crate 'shared' should not appear in Running Processes"
    );
}

#[test]
fn test_meta_version() {
    let mut cmd = cargo_bin_cmd!("meta");
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("meta"));
}

#[test]
fn test_meta_help() {
    let mut cmd = cargo_bin_cmd!("meta");
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Meta task orchestrator"));
}

#[test]
fn test_meta_init_creates_config() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = cargo_bin_cmd!("meta");

    cmd.current_dir(&temp_dir);
    cmd.arg("init");
    cmd.assert().success();

    // Verify meta.toml was created
    let config_path = temp_dir.path().join("meta.toml");
    assert!(config_path.exists());

    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("[workspace]"));
    assert!(content.contains("[tools.turborepo]"));
    assert!(content.contains("[tools.bacon]"));
}

#[test]
fn test_meta_init_detects_rust_project() {
    let temp_dir = TempDir::new().unwrap();

    // Create apps directory with a Rust project
    let api_path = temp_dir.path().join("apps/api");
    fs::create_dir_all(&api_path).unwrap();
    fs::write(
        api_path.join("Cargo.toml"),
        r#"[package]
name = "api"
version = "1.0.0"
"#,
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("meta");
    cmd.current_dir(&temp_dir);
    cmd.arg("init");
    cmd.assert().success();

    let config_path = temp_dir.path().join("meta.toml");
    let content = fs::read_to_string(&config_path).unwrap();

    assert!(content.contains("[projects.api]"));
    assert!(content.contains("type = \"rust\""));
    assert!(content.contains("path = \"apps/api\""));
}

#[test]
fn test_meta_init_detects_next_project() {
    let temp_dir = TempDir::new().unwrap();

    // Create apps directory with a Next.js project
    let web_path = temp_dir.path().join("apps/web");
    fs::create_dir_all(&web_path).unwrap();
    fs::write(
        web_path.join("package.json"),
        r#"{
  "name": "@test/web",
  "dependencies": {
    "next": "14.0.0",
    "react": "18.0.0"
  }
}"#,
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("meta");
    cmd.current_dir(&temp_dir);
    cmd.arg("init");
    cmd.assert().success();

    let config_path = temp_dir.path().join("meta.toml");
    let content = fs::read_to_string(&config_path).unwrap();

    assert!(content.contains("[projects.web]"));
    assert!(content.contains("type = \"next\""));
    assert!(content.contains("@test/web"));
}

#[test]
fn test_meta_dev_requires_config() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = cargo_bin_cmd!("meta");

    cmd.current_dir(&temp_dir);
    cmd.arg("dev");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("meta.toml not found"));
}

#[test]
fn test_meta_build_requires_config() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = cargo_bin_cmd!("meta");

    cmd.current_dir(&temp_dir);
    cmd.arg("build");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("meta.toml not found"));
}

#[test]
fn test_meta_test_requires_config() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = cargo_bin_cmd!("meta");

    cmd.current_dir(&temp_dir);
    cmd.arg("test");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("meta.toml not found"));
}

// Issue #8: meta dev --detach flag exists
#[test]
fn test_meta_dev_detach_flag_accepted() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = cargo_bin_cmd!("meta");
    cmd.current_dir(&temp_dir);
    cmd.args(["dev", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--detach"));
}

// Issue #9: meta status --json outputs valid JSON
#[test]
fn test_meta_status_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let api_path = temp_dir.path().join("apps/api");
    fs::create_dir_all(api_path.join("src")).unwrap();
    fs::write(api_path.join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(
        api_path.join("Cargo.toml"),
        "[package]\nname = \"api\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    fs::write(
        temp_dir.path().join("meta.toml"),
        r#"version = "1"

[workspace]
name = "Test"
root = "."

[tools.bacon]
enabled = true
command = "bacon"

[projects.api]
type = "rust"
path = "apps/api"

[projects.api.tasks]
dev = { tool = "bacon", command = "run-long" }
"#,
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("meta");
    cmd.current_dir(&temp_dir);
    cmd.args(["status", "--json"]);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("Invalid JSON output: {}\nOutput: {}", e, stdout));

    // Should have session and projects fields
    assert!(parsed.get("session").is_some(), "Missing 'session' field");
    assert!(parsed.get("projects").is_some(), "Missing 'projects' field");

    let projects = parsed["projects"].as_array().unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0]["name"], "api");
    assert_eq!(projects[0]["status"], "not running");
    assert_eq!(projects[0]["tool"], "bacon");
}

// Issue #9: meta status --json excludes projects without dev task
#[test]
fn test_meta_status_json_excludes_libraries() {
    let temp_dir = TempDir::new().unwrap();
    let api_path = temp_dir.path().join("apps/api");
    fs::create_dir_all(api_path.join("src")).unwrap();
    fs::write(api_path.join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(
        api_path.join("Cargo.toml"),
        "[package]\nname = \"api\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    let shared_path = temp_dir.path().join("crates/shared");
    fs::create_dir_all(shared_path.join("src")).unwrap();
    fs::write(shared_path.join("src/lib.rs"), "").unwrap();
    fs::write(
        shared_path.join("Cargo.toml"),
        "[package]\nname = \"shared\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    fs::write(
        temp_dir.path().join("meta.toml"),
        r#"version = "1"

[workspace]
name = "Test"
root = "."

[tools.bacon]
enabled = true
command = "bacon"

[tools.cargo]
enabled = true
command = "cargo"

[projects.api]
type = "rust"
path = "apps/api"

[projects.api.tasks]
dev = { tool = "bacon", command = "run-long" }

[projects.shared]
type = "rust"
path = "crates/shared"

[projects.shared.tasks]
build = { tool = "cargo", command = "build" }
"#,
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("meta");
    cmd.current_dir(&temp_dir);
    cmd.args(["status", "--json"]);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    let projects = parsed["projects"].as_array().unwrap();
    let names: Vec<&str> = projects
        .iter()
        .map(|p| p["name"].as_str().unwrap())
        .collect();

    assert!(names.contains(&"api"));
    assert!(
        !names.contains(&"shared"),
        "Library crate should not appear in JSON output"
    );
}

// `meta init` writes an mcp-log-server entry to .mcp.json by default
#[test]
fn test_init_writes_mcp_json_by_default() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = cargo_bin_cmd!("meta");
    cmd.current_dir(&temp_dir);
    cmd.arg("init");

    cmd.assert().success();

    let mcp_path = temp_dir.path().join(".mcp.json");
    assert!(mcp_path.exists(), ".mcp.json should be created by default");

    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&mcp_path).unwrap()).unwrap();
    let entry = &value["mcpServers"]["mcp-log-server"];
    assert_eq!(entry["command"], "docker");
    let args = entry["args"].as_array().unwrap();
    assert!(args.iter().any(|v| v == "LOG_DIR=/logs"));
}

// `meta init --no-mcp` skips writing .mcp.json
#[test]
fn test_init_no_mcp_skips_mcp_json() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = cargo_bin_cmd!("meta");
    cmd.current_dir(&temp_dir);
    cmd.args(["init", "--no-mcp"]);

    cmd.assert().success();

    assert!(
        !temp_dir.path().join(".mcp.json").exists(),
        ".mcp.json should not be created with --no-mcp"
    );
}
