use std::{collections::HashMap, fs, path::Path};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub workspace: WorkspaceConfig,
    pub tools: HashMap<String, ToolConfig>,
    pub projects: HashMap<String, ProjectConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub name: String,
    pub root: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolConfig {
    pub enabled: bool,
    pub command: String,
    #[serde(default)]
    pub for_languages: Vec<String>,
    #[serde(default)]
    pub for_tasks: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    #[serde(rename = "type")]
    pub project_type: String,
    pub path: String,
    #[serde(default = "default_true")]
    pub dev_default: bool,
    pub tasks: HashMap<String, TaskConfig>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskConfig {
    pub tool: String,
    pub command: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Path::new("meta.toml");
        if !path.exists() {
            anyhow::bail!("meta.toml not found. Run 'meta init' first.");
        }

        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Returns projects that have a "dev" task configured
    pub fn projects_with_dev_task(&self) -> HashMap<String, &ProjectConfig> {
        self.projects
            .iter()
            .filter(|(_, project)| project.tasks.contains_key("dev"))
            .map(|(name, project)| (name.clone(), project))
            .collect()
    }

    /// Returns projects that should run by default with `meta dev`
    /// (have a dev task AND dev_default is not false)
    pub fn default_dev_projects(&self) -> HashMap<String, &ProjectConfig> {
        self.projects
            .iter()
            .filter(|(_, project)| project.tasks.contains_key("dev") && project.dev_default)
            .map(|(name, project)| (name.clone(), project))
            .collect()
    }
}

pub fn init(with_mcp: bool) -> Result<()> {
    // Auto-detect projects in the monorepo
    let detected_projects = detect_projects()?;
    let docker_detected = detect_docker(Path::new("."));

    // Generate configuration based on detected projects
    let config = generate_config(&detected_projects, docker_detected)?;

    fs::write("meta.toml", config)?;

    if with_mcp {
        write_mcp_log_server_entry(Path::new(".mcp.json"))?;
    }
    Ok(())
}

/// Merge an mcp-log-server entry into `.mcp.json`, creating the file if absent.
///
/// Idempotent: if an `mcp-log-server` entry already exists, leaves it alone.
fn write_mcp_log_server_entry(path: &Path) -> Result<()> {
    let mut root: serde_json::Value = if path.exists() {
        serde_json::from_str(&fs::read_to_string(path)?)?
    } else {
        serde_json::json!({ "mcpServers": {} })
    };

    let root_obj = root
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!(".mcp.json: root must be a JSON object"))?;

    let servers = root_obj
        .entry("mcpServers")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!(".mcp.json: mcpServers must be an object"))?;

    if servers.contains_key("mcp-log-server") {
        return Ok(());
    }

    servers.insert(
        "mcp-log-server".into(),
        serde_json::json!({
            "command": "docker",
            "args": [
                "run", "--rm", "-i",
                "-v", "./.meta/logs:/logs",
                "-e", "LOG_DIR=/logs",
                "ghcr.io/wolven-tech/mcp-log-server:latest"
            ]
        }),
    );

    fs::write(path, serde_json::to_string_pretty(&root)?)?;
    Ok(())
}

const COMPOSE_FILES: &[&str] = &[
    "docker-compose.yml",
    "docker-compose.yaml",
    "compose.yml",
    "compose.yaml",
];

fn detect_docker(root: &Path) -> bool {
    for f in COMPOSE_FILES {
        if root.join(f).exists() {
            return true;
        }
    }

    if let Ok(entries) = fs::read_dir(root.join("apps")) {
        for entry in entries.flatten() {
            let Ok(ft) = entry.file_type() else { continue };
            if !ft.is_dir() {
                continue;
            }
            let path = entry.path();
            for f in COMPOSE_FILES {
                if path.join(f).exists() {
                    return true;
                }
            }
        }
    }

    false
}

fn detect_projects() -> Result<Vec<DetectedProject>> {
    let mut projects = Vec::new();

    // Check apps directory
    if let Ok(entries) = fs::read_dir("apps") {
        for entry in entries.flatten() {
            if entry.file_type()?.is_dir() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();

                // Check for Cargo.toml (Rust project)
                if path.join("Cargo.toml").exists() {
                    projects.push(DetectedProject {
                        name: name.clone(),
                        path: format!("apps/{}", name),
                        project_type: ProjectType::Rust,
                        package_name: None,
                    });
                }
                // Check for package.json (Node project)
                else if path.join("package.json").exists() {
                    let package_json = fs::read_to_string(path.join("package.json"))?;
                    let package: serde_json::Value = serde_json::from_str(&package_json)?;
                    let package_name = package["name"].as_str().map(|s| s.to_string());

                    // Detect Next.js project
                    let is_next = package["dependencies"]
                        .as_object()
                        .and_then(|deps| deps.get("next"))
                        .is_some();

                    projects.push(DetectedProject {
                        name: name.clone(),
                        path: format!("apps/{}", name),
                        project_type: if is_next {
                            ProjectType::Next
                        } else {
                            ProjectType::Node
                        },
                        package_name,
                    });
                }
            }
        }
    }

    Ok(projects)
}

#[derive(Debug)]
struct DetectedProject {
    name: String,
    path: String,
    project_type: ProjectType,
    package_name: Option<String>,
}

#[derive(Debug)]
enum ProjectType {
    Rust,
    Next,
    Node,
}

/// Parse a TOML string into a Config
#[cfg(test)]
pub fn parse(contents: &str) -> Result<Config> {
    let config: Config = toml::from_str(contents)?;
    Ok(config)
}

fn generate_config(projects: &[DetectedProject], docker: bool) -> Result<String> {
    let mut config = String::from(
        r#"version = "1"

[workspace]
name = "Meta Monorepo"
root = "."

# Tool declarations
[tools.turborepo]
enabled = true
command = "turbo"
for_languages = ["typescript", "javascript"]
for_tasks = ["dev", "build", "lint", "typecheck"]

[tools.bacon]
enabled = true
command = "bacon"
for_languages = ["rust"]
for_tasks = ["dev"]

[tools.cargo]
enabled = true
command = "cargo"
for_languages = ["rust"]
for_tasks = ["build", "test"]

"#,
    );

    if docker {
        config.push_str(
            r#"[tools.docker]
enabled = true
command = "docker"
for_tasks = ["dev"]

"#,
        );
    }

    // Add detected projects
    config.push_str("# Project definitions\n");

    for project in projects {
        config.push_str(&format!("[projects.{}]\n", project.name));

        match project.project_type {
            ProjectType::Rust => {
                config.push_str(&format!("type = \"rust\"\npath = \"{}\"\n\n", project.path));
                config.push_str(&format!("[projects.{}.tasks]\n", project.name));
                config.push_str("dev = { tool = \"bacon\", command = \"run-long\" }\n");
                config.push_str("build = { tool = \"cargo\", command = \"build --release\" }\n");
                config.push_str("test = { tool = \"cargo\", command = \"test\" }\n\n");
            }
            ProjectType::Next | ProjectType::Node => {
                let project_type = if matches!(project.project_type, ProjectType::Next) {
                    "next"
                } else {
                    "node"
                };

                config.push_str(&format!(
                    "type = \"{}\"\npath = \"{}\"\n\n",
                    project_type, project.path
                ));
                config.push_str(&format!("[projects.{}.tasks]\n", project.name));

                if let Some(ref pkg_name) = project.package_name {
                    config.push_str(&format!(
                        "dev = {{ tool = \"turborepo\", command = \"dev --filter={}\" }}\n",
                        pkg_name
                    ));
                    config.push_str(&format!(
                        "build = {{ tool = \"turborepo\", command = \"build --filter={}\" }}\n",
                        pkg_name
                    ));
                    config.push_str(&format!(
                        "test = {{ tool = \"turborepo\", command = \"test --filter={}\" }}\n\n",
                        pkg_name
                    ));
                } else {
                    config.push_str("dev = { tool = \"turborepo\", command = \"dev\" }\n");
                    config.push_str("build = { tool = \"turborepo\", command = \"build\" }\n");
                    config.push_str("test = { tool = \"turborepo\", command = \"test\" }\n\n");
                }
            }
        }
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config_toml() -> &'static str {
        r#"
version = "1"

[workspace]
name = "Test"
root = "."

[tools.bacon]
enabled = true
command = "bacon"
for_languages = ["rust"]
for_tasks = ["dev"]

[tools.cargo]
enabled = true
command = "cargo"
for_languages = ["rust"]
for_tasks = ["build", "test"]

[projects.api]
type = "rust"
path = "apps/api"

[projects.api.tasks]
dev = { tool = "bacon", command = "run-long" }
build = { tool = "cargo", command = "build" }

[projects.shared]
type = "rust"
path = "crates/shared"

[projects.shared.tasks]
build = { tool = "cargo", command = "build" }
test = { tool = "cargo", command = "test" }

[projects.trainee-app]
type = "rust"
path = "apps/trainee-app"

[projects.trainee-app.tasks]
dev = { tool = "bacon", command = "run-long" }

[projects.trainee-android]
type = "rust"
path = "apps/trainee-app"
dev_default = false

[projects.trainee-android.tasks]
dev = { tool = "cargo", command = "tauri android dev" }
"#
    }

    // Issue #7: doctor should not falsely report dev task for projects without one
    #[test]
    fn test_project_without_dev_task_not_in_dev_projects() {
        let config = parse(test_config_toml()).unwrap();
        let dev_projects = config.projects_with_dev_task();
        assert!(!dev_projects.contains_key("shared"));
    }

    #[test]
    fn test_project_with_dev_task_in_dev_projects() {
        let config = parse(test_config_toml()).unwrap();
        let dev_projects = config.projects_with_dev_task();
        assert!(dev_projects.contains_key("api"));
        assert!(dev_projects.contains_key("trainee-app"));
    }

    #[test]
    fn test_projects_sharing_path_have_independent_dev_task_detection() {
        let config = parse(test_config_toml()).unwrap();
        let dev_projects = config.projects_with_dev_task();
        // Both have dev tasks, both should appear despite shared path
        assert!(dev_projects.contains_key("trainee-app"));
        assert!(dev_projects.contains_key("trainee-android"));
    }

    #[test]
    fn test_project_sharing_path_without_dev_task_excluded() {
        let toml = r#"
version = "1"

[workspace]
name = "Test"
root = "."

[tools.bacon]
enabled = true
command = "bacon"

[projects.trainee-app]
type = "rust"
path = "apps/trainee-app"

[projects.trainee-app.tasks]
dev = { tool = "bacon", command = "run-long" }

[projects.trainee-android]
type = "rust"
path = "apps/trainee-app"

[projects.trainee-android.tasks]
build = { tool = "bacon", command = "build" }
"#;
        let config = parse(toml).unwrap();
        let dev_projects = config.projects_with_dev_task();
        assert!(dev_projects.contains_key("trainee-app"));
        assert!(!dev_projects.contains_key("trainee-android"));
    }

    // Issue #2: status should only show projects with dev task
    #[test]
    fn test_projects_with_dev_task_excludes_libraries() {
        let config = parse(test_config_toml()).unwrap();
        let dev_projects = config.projects_with_dev_task();
        assert!(!dev_projects.contains_key("shared"));
        assert!(dev_projects.contains_key("api"));
    }

    // Issue #1: dev_default = false excludes from default dev run
    #[test]
    fn test_dev_default_false_excludes_from_default_projects() {
        let config = parse(test_config_toml()).unwrap();
        let default_projects = config.default_dev_projects();
        assert!(!default_projects.contains_key("trainee-android"));
        assert!(default_projects.contains_key("api"));
        assert!(default_projects.contains_key("trainee-app"));
    }

    // Issue #7: exact reproduction of the reported config
    #[test]
    fn test_issue_7_exact_reproduction() {
        // This is the exact config from the GitHub issue
        let toml = r#"
version = "1"

[workspace]
name = "Test"
root = "."

[tools.trunk]
enabled = true
command = "trunk"

[tools.tauri]
enabled = true
command = "cargo-tauri"

[projects.trainee-app]
type = "rust"
path = "apps/trainee-app"

[projects.trainee-app.tasks]
dev = { tool = "trunk", command = "serve --port 3853" }

[projects.trainee-android]
type = "rust"
path = "apps/trainee-app"

[projects.trainee-android.tasks]
build = { tool = "tauri", command = "android build --target aarch64" }
"#;
        let config = parse(toml).unwrap();

        // trainee-app HAS dev task
        assert!(
            config.projects["trainee-app"].tasks.contains_key("dev"),
            "trainee-app should have dev task"
        );

        // trainee-android does NOT have dev task — only has build
        assert!(
            !config.projects["trainee-android"].tasks.contains_key("dev"),
            "trainee-android should NOT have dev task"
        );

        // Verify via the helper methods too
        let dev_projects = config.projects_with_dev_task();
        assert!(dev_projects.contains_key("trainee-app"));
        assert!(
            !dev_projects.contains_key("trainee-android"),
            "trainee-android should not appear in dev projects"
        );
    }

    #[test]
    fn test_dev_default_defaults_to_true() {
        let config = parse(test_config_toml()).unwrap();
        assert!(config.projects["api"].dev_default);
        assert!(!config.projects["trainee-android"].dev_default);
    }

    #[test]
    fn test_detect_docker_finds_compose_at_root() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("docker-compose.yml"), "services: {}\n").unwrap();
        assert!(detect_docker(tmp.path()));
    }

    #[test]
    fn test_detect_docker_finds_compose_in_apps_subdir() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("apps/redis")).unwrap();
        fs::write(tmp.path().join("apps/redis/compose.yaml"), "services: {}\n").unwrap();
        assert!(detect_docker(tmp.path()));
    }

    #[test]
    fn test_detect_docker_returns_false_when_absent() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("apps/api")).unwrap();
        fs::write(tmp.path().join("apps/api/Cargo.toml"), "[package]\n").unwrap();
        assert!(!detect_docker(tmp.path()));
    }

    #[test]
    fn test_generate_config_emits_docker_tool_when_detected() {
        let project = DetectedProject {
            name: "api".into(),
            path: "apps/api".into(),
            project_type: ProjectType::Rust,
            package_name: None,
        };
        let output = generate_config(&[project], true).unwrap();
        assert!(output.contains("[tools.docker]"));
        assert!(output.contains("command = \"docker\""));
        // Generated config must parse back into a valid Config
        let parsed = parse(&output).unwrap();
        assert!(parsed.tools.contains_key("docker"));
        assert_eq!(parsed.tools["docker"].command, "docker");
    }

    #[test]
    fn test_generate_config_omits_docker_tool_when_not_detected() {
        let output = generate_config(&[], false).unwrap();
        assert!(!output.contains("[tools.docker]"));
    }

    #[test]
    fn test_write_mcp_entry_creates_file_when_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".mcp.json");

        write_mcp_log_server_entry(&path).unwrap();

        let contents = fs::read_to_string(&path).unwrap();
        let value: serde_json::Value = serde_json::from_str(&contents).unwrap();
        let entry = &value["mcpServers"]["mcp-log-server"];
        assert_eq!(entry["command"], "docker");
        let args = entry["args"].as_array().unwrap();
        assert!(args.iter().any(|v| v == "LOG_DIR=/logs"));
        assert!(args.iter().any(|v| v == "./.meta/logs:/logs"));
    }

    #[test]
    fn test_write_mcp_entry_preserves_existing_servers() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".mcp.json");
        fs::write(
            &path,
            r#"{"mcpServers":{"other":{"command":"npx","args":["-y","some-pkg"]}}}"#,
        )
        .unwrap();

        write_mcp_log_server_entry(&path).unwrap();

        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(value["mcpServers"]["other"]["command"], "npx");
        assert_eq!(value["mcpServers"]["mcp-log-server"]["command"], "docker");
    }

    #[test]
    fn test_write_mcp_entry_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".mcp.json");
        fs::write(
            &path,
            r#"{"mcpServers":{"mcp-log-server":{"command":"custom","args":[]}}}"#,
        )
        .unwrap();

        write_mcp_log_server_entry(&path).unwrap();

        // User's existing entry should be preserved, not overwritten
        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(value["mcpServers"]["mcp-log-server"]["command"], "custom");
    }

    #[test]
    fn test_write_mcp_entry_adds_mcp_servers_key_when_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".mcp.json");
        fs::write(&path, r#"{}"#).unwrap();

        write_mcp_log_server_entry(&path).unwrap();

        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(value["mcpServers"]["mcp-log-server"]["command"], "docker");
    }
}
