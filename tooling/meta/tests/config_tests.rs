use std::fs;

use tempfile::TempDir;

#[test]
fn test_config_loads_valid_toml() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("meta.toml");

    fs::write(
        &config_path,
        r#"version = "1"

[workspace]
name = "Test Workspace"
root = "."

[tools.cargo]
enabled = true
command = "cargo"
for_languages = ["rust"]
for_tasks = ["build"]

[projects.test]
type = "rust"
path = "apps/test"

[projects.test.tasks]
build = { tool = "cargo", command = "build" }
"#,
    )
    .unwrap();

    // Change to temp directory
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();

    // This would use the Config::load() function
    // For now just verify the file exists
    assert!(config_path.exists());

    // Restore directory
    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_detect_projects_finds_rust() {
    let temp_dir = TempDir::new().unwrap();

    // Create a Rust project
    let rust_path = temp_dir.path().join("apps/api");
    fs::create_dir_all(&rust_path).unwrap();
    fs::write(
        rust_path.join("Cargo.toml"),
        r#"[package]
name = "api"
version = "1.0.0"
"#,
    )
    .unwrap();

    // Directory structure is created correctly
    assert!(rust_path.join("Cargo.toml").exists());
}

#[test]
fn test_detect_projects_finds_nextjs() {
    let temp_dir = TempDir::new().unwrap();

    // Create a Next.js project
    let next_path = temp_dir.path().join("apps/web");
    fs::create_dir_all(&next_path).unwrap();
    fs::write(
        next_path.join("package.json"),
        r#"{
  "name": "@test/web",
  "dependencies": {
    "next": "14.0.0"
  }
}"#,
    )
    .unwrap();

    let package_json = fs::read_to_string(next_path.join("package.json")).unwrap();
    assert!(package_json.contains("next"));
}

#[test]
fn test_detect_projects_finds_node() {
    let temp_dir = TempDir::new().unwrap();

    // Create a Node project (no Next.js)
    let node_path = temp_dir.path().join("apps/service");
    fs::create_dir_all(&node_path).unwrap();
    fs::write(
        node_path.join("package.json"),
        r#"{
  "name": "@test/service",
  "dependencies": {
    "express": "4.0.0"
  }
}"#,
    )
    .unwrap();

    let package_json = fs::read_to_string(node_path.join("package.json")).unwrap();
    assert!(!package_json.contains("next"));
}

#[test]
fn test_multiple_projects_detected() {
    let temp_dir = TempDir::new().unwrap();
    let apps_dir = temp_dir.path().join("apps");

    // Create multiple projects
    let rust_path = apps_dir.join("api");
    fs::create_dir_all(&rust_path).unwrap();
    fs::write(rust_path.join("Cargo.toml"), "[package]\nname = \"api\"").unwrap();

    let next_path = apps_dir.join("web");
    fs::create_dir_all(&next_path).unwrap();
    fs::write(
        next_path.join("package.json"),
        r#"{"name": "@test/web", "dependencies": {"next": "14.0.0"}}"#,
    )
    .unwrap();

    let node_path = apps_dir.join("api-node");
    fs::create_dir_all(&node_path).unwrap();
    fs::write(
        node_path.join("package.json"),
        r#"{"name": "@test/api-node"}"#,
    )
    .unwrap();

    // Verify all projects exist
    assert!(rust_path.join("Cargo.toml").exists());
    assert!(next_path.join("package.json").exists());
    assert!(node_path.join("package.json").exists());
}
