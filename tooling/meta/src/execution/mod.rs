use std::collections::HashMap;
use std::io::IsTerminal;

use anyhow::Result;
use tokio::process::Command;

use crate::{adapters::ToolAdapter, config::Config};

/// Generate unique session name from current directory
fn get_session_name() -> String {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .map(|name| format!("meta-{}", sanitize_session_name(&name)))
        .unwrap_or_else(|| "meta-dev".to_string())
}

fn sanitize_session_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    sanitized.trim_matches('-').to_string()
}

/// Pane information from tmux
struct PaneInfo {
    index: usize,
    title: String,
    pid: u32,
    _start_time: String,
}

/// Query tmux for pane information
async fn get_tmux_panes(session_name: &str) -> Result<Vec<PaneInfo>> {
    let output = Command::new("tmux")
        .args([
            "list-panes",
            "-t",
            session_name,
            "-F",
            "#{pane_index}|#{pane_title}|#{pane_pid}|#{pane_start_time}",
        ])
        .output()
        .await?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut panes = vec![];

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 4 {
            panes.push(PaneInfo {
                index: parts[0].parse().unwrap_or(0),
                title: parts[1].to_string(),
                pid: parts[2].parse().unwrap_or(0),
                _start_time: parts[3].to_string(),
            });
        }
    }
    Ok(panes)
}

/// Build a pid→children map from the full process table.
/// This is more reliable than `pgrep -P` because it captures the tree in a
/// single snapshot, avoiding races where processes exit between calls.
fn build_process_tree(ps_output: &str) -> HashMap<u32, Vec<u32>> {
    let mut tree: HashMap<u32, Vec<u32>> = HashMap::new();
    for line in ps_output.lines().skip(1) {
        // format: "  PID  PPID"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let (Ok(pid), Ok(ppid)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                tree.entry(ppid).or_default().push(pid);
            }
        }
    }
    tree
}

/// Recursively collect all descendant PIDs from a process tree map.
fn collect_descendants(tree: &HashMap<u32, Vec<u32>>, pid: u32) -> Vec<u32> {
    let mut result = Vec::new();
    if let Some(children) = tree.get(&pid) {
        for &child in children {
            result.push(child);
            result.extend(collect_descendants(tree, child));
        }
    }
    result
}

/// Find the best PID to report for a project.
///
/// Strategy (in order of reliability):
/// 1. Check if the pane PID itself is alive (shell wrapper still running)
/// 2. Walk the full process tree to find any alive descendant
/// 3. Fall back to tmux's live pane_pid (tmux tracks the foreground process)
///
/// This handles the bacon case where: shell → bacon → cargo → binary
/// If the shell exits, children are reparented to launchd (pid 1) on macOS,
/// making pgrep -P unreliable. The full process tree snapshot avoids this.
async fn find_active_pid(
    pane_pid: u32,
    session_name: &str,
    pane_index: Option<usize>,
) -> Option<u32> {
    // Strategy 1: Check if the pane PID itself is alive
    let direct_check = Command::new("ps")
        .args(["-p", &pane_pid.to_string(), "-o", "pid="])
        .output()
        .await;

    if let Ok(output) = &direct_check {
        let pid_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !pid_str.is_empty() {
            return Some(pane_pid);
        }
    }

    // Strategy 2: Build full process tree and find descendants
    let ps_output = Command::new("ps").args(["-axo", "pid,ppid"]).output().await;

    if let Ok(output) = ps_output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let tree = build_process_tree(&stdout);
        let descendants = collect_descendants(&tree, pane_pid);

        // Return the deepest alive descendant (most likely the actual binary)
        // Check from the end since descendants are added depth-first
        for &desc_pid in descendants.iter().rev() {
            let alive = Command::new("ps")
                .args(["-p", &desc_pid.to_string(), "-o", "pid="])
                .output()
                .await;
            if let Ok(out) = alive {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !s.is_empty() {
                    return Some(desc_pid);
                }
            }
        }
    }

    // Strategy 3: Ask tmux for the current pane PID (may have updated)
    if let Some(idx) = pane_index {
        let pane_target = format!("{}:{}.{}", session_name, 0, idx);
        let tmux_pid = Command::new("tmux")
            .args(["display-message", "-p", "-t", &pane_target, "#{pane_pid}"])
            .output()
            .await;

        if let Ok(output) = tmux_pid {
            let pid_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Ok(pid) = pid_str.parse::<u32>() {
                if pid != pane_pid {
                    // tmux reports a different PID — check if it's alive
                    let alive = Command::new("ps")
                        .args(["-p", &pid.to_string(), "-o", "pid="])
                        .output()
                        .await;
                    if let Ok(out) = alive {
                        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                        if !s.is_empty() {
                            return Some(pid);
                        }
                    }
                }
            }
        }
    }

    None
}

/// Show status of running dev processes
///
/// # Output Format (designed for Claude Code parsing)
///
/// ```text
/// === META DEV STATUS ===
/// Log file: .meta/logs/dev.log
///
/// ## Running Processes
/// PROJECT    PID      STARTED                    UPTIME
/// api        12345    2025-12-08T12:02:18        1h 23m
/// web        12346    2025-12-08T12:02:19        1h 23m
///
/// ## Recent Events (last 20)
/// [2025-12-08T12:02:18] [api] START: Process started (pid=12345)
/// [2025-12-08T12:05:32] [api] RESTART: bacon rebuild detected
/// ...
///
/// ## Binary Modification Times
/// apps/api/target/debug/api: 2025-12-08T12:05:30 (rebuilt 1h 20m ago)
/// ```
pub async fn status(
    config: &Config,
    project: Option<String>,
    lines: usize,
    json: bool,
) -> Result<()> {
    if json {
        return status_json(config, project).await;
    }

    let session_name = get_session_name();

    println!("=== META DEV STATUS ===");
    println!("Session: {}", session_name);
    println!("Log file: .meta/logs/dev.log\n");

    // Check if tmux session exists
    let session_check = Command::new("tmux")
        .args(["has-session", "-t", &session_name])
        .output()
        .await;

    let session_active = session_check.map(|o| o.status.success()).unwrap_or(false);

    if !session_active {
        println!(
            "⚠️  No active {} session. Run 'meta dev' to start.\n",
            session_name
        );
    }

    // Get tmux panes for process detection
    let panes = get_tmux_panes(&session_name).await.unwrap_or_default();

    // Show running processes
    println!("## Running Processes");
    println!("{:<15} {:<10} {:<28} UPTIME", "PROJECT", "PID", "STARTED");
    println!("{}", "-".repeat(70));

    let dev_projects = config.projects_with_dev_task();
    for name in dev_projects.keys() {
        if let Some(ref filter) = project {
            if name != filter {
                continue;
            }
        }

        // Find the pane for this project by matching title
        let pane = panes.iter().find(|p| &p.title == name);

        if let Some(pane) = pane {
            // Walk process tree to find the actual running process
            // This handles bacon-spawned processes where the shell wrapper may have exited
            let active_pid = find_active_pid(pane.pid, &session_name, Some(pane.index)).await;

            if let Some(pid) = active_pid {
                let detail_output = Command::new("ps")
                    .args(["-p", &pid.to_string(), "-o", "pid,lstart,etime"])
                    .output()
                    .await;

                if let Ok(detail) = detail_output {
                    let detail_str = String::from_utf8_lossy(&detail.stdout);
                    let lines_vec: Vec<&str> = detail_str.lines().collect();
                    if lines_vec.len() >= 2 {
                        let info_parts: Vec<&str> = lines_vec[1].split_whitespace().collect();
                        if info_parts.len() >= 6 {
                            let started = info_parts[1..6].join(" ");
                            let uptime = info_parts.last().unwrap_or(&"?");
                            println!("{:<15} {:<10} {:<28} {}", name, pid, started, uptime);
                        } else {
                            println!("{:<15} {:<10} {:<28} -", name, pid, "(running)");
                        }
                    } else {
                        println!("{:<15} {:<10} {:<28} -", name, "-", "not running");
                    }
                } else {
                    println!("{:<15} {:<10} {:<28} -", name, "-", "not running");
                }
            } else {
                println!("{:<15} {:<10} {:<28} -", name, "-", "not running");
            }
        } else {
            println!("{:<15} {:<10} {:<28} -", name, "-", "not running");
        }
    }

    // Show recent log events
    println!("\n## Recent Events (last {})", lines);
    let log_path = std::path::Path::new(".meta/logs/dev.log");

    if log_path.exists() {
        let log_content = std::fs::read_to_string(log_path)?;
        let log_lines: Vec<&str> = log_content.lines().collect();

        let filtered_lines: Vec<&str> = if let Some(ref filter) = project {
            log_lines
                .iter()
                .filter(|line| line.contains(&format!("[{}]", filter)))
                .copied()
                .collect()
        } else {
            log_lines
        };

        let start = filtered_lines.len().saturating_sub(lines);
        for line in &filtered_lines[start..] {
            println!("{}", line);
        }

        if filtered_lines.is_empty() {
            println!("(no log entries yet)");
        }
    } else {
        println!("(no log file yet - will be created on next 'meta dev')");
    }

    // Show binary modification times for Rust projects and detect stale processes
    println!("\n## Binary Status (Rust projects)");
    println!(
        "# NOTE: If binary is newer than process, bacon rebuilt but process may be stale.\n# This \
         can happen if bacon is running in check mode instead of run-long mode.\n"
    );

    for (name, proj) in &dev_projects {
        if proj.project_type != "rust" {
            continue;
        }
        if let Some(ref filter) = project {
            if name != filter {
                continue;
            }
        }

        // Skip library crates — they don't produce binaries
        if is_library_crate(&proj.path) {
            continue;
        }

        // Get actual binary name from Cargo.toml
        let binary_name = get_rust_binary_name(&proj.path).unwrap_or_else(|| name.clone());
        // Use workspace-aware binary path
        let workspace_root = detect_cargo_workspace(&proj.path);
        let binary_path = get_rust_binary_path(&proj.path, &binary_name, workspace_root.as_deref());
        let path = std::path::Path::new(&binary_path);

        if path.exists() {
            if let Ok(metadata) = path.metadata() {
                if let Ok(binary_modified) = metadata.modified() {
                    let binary_age = std::time::SystemTime::now()
                        .duration_since(binary_modified)
                        .unwrap_or_default();

                    let mut process_status = String::new();

                    // Check if there's a running pane for this project
                    // Walk process tree to find active PID (handles bacon grandchildren)
                    if let Some(pane) = panes.iter().find(|p| p.title == *name) {
                        let active_pid = find_active_pid(pane.pid, &session_name, Some(pane.index))
                            .await
                            .unwrap_or(pane.pid);
                        // Get elapsed time using ps -o etime (format: [[DD-]HH:]MM:SS)
                        let etime_output = Command::new("ps")
                            .args(["-p", &active_pid.to_string(), "-o", "etime="])
                            .output()
                            .await;

                        if let Ok(etime) = etime_output {
                            let etime_str =
                                String::from_utf8_lossy(&etime.stdout).trim().to_string();
                            if let Some(process_age_secs) = parse_etime(&etime_str) {
                                let binary_age_secs = binary_age.as_secs();

                                // Only consider stale if binary is more than 60 seconds
                                // newer than process (to avoid false positives from timing)
                                let stale_threshold_secs = 60;
                                if binary_age_secs + stale_threshold_secs < process_age_secs {
                                    // Binary is significantly newer than process - STALE!
                                    let diff_secs = process_age_secs - binary_age_secs;
                                    let diff_mins = diff_secs / 60;
                                    process_status = format!(
                                        " ⚠️  STALE: binary rebuilt {}m after process started",
                                        diff_mins
                                    );
                                } else {
                                    process_status = " ✓ running latest binary".to_string();
                                }
                            }
                        }
                    }

                    let mins = binary_age.as_secs() / 60;
                    let hours = mins / 60;
                    let age_str = if hours > 0 {
                        format!("{}h {}m ago", hours, mins % 60)
                    } else {
                        format!("{}m ago", mins)
                    };

                    println!("{}: rebuilt {}{}", binary_path, age_str, process_status);
                }
            }
        } else {
            println!("{}: not built yet", binary_path);
        }
    }

    // Show available project logs
    println!("\n## Project Logs");
    println!(
        "Use 'meta logs <project>' to view logs, or 'meta logs <project> --follow' to stream.\n"
    );
    for name in dev_projects.keys() {
        if let Some(ref filter) = project {
            if name != filter {
                continue;
            }
        }
        let log_path = format!(".meta/logs/{}.log", name);
        if std::path::Path::new(&log_path).exists() {
            if let Ok(metadata) = std::fs::metadata(&log_path) {
                let size_kb = metadata.len() / 1024;
                println!("  {} → {} ({}KB)", name, log_path, size_kb);
            }
        } else {
            println!("  {} → (no log file yet)", name);
        }
    }

    println!();
    Ok(())
}

/// JSON output for `meta status --json`
async fn status_json(config: &Config, project: Option<String>) -> Result<()> {
    let session_name = get_session_name();

    // Check session
    let session_check = Command::new("tmux")
        .args(["has-session", "-t", &session_name])
        .output()
        .await;
    let session_active = session_check.map(|o| o.status.success()).unwrap_or(false);

    let panes = if session_active {
        get_tmux_panes(&session_name).await.unwrap_or_default()
    } else {
        vec![]
    };

    let dev_projects = config.projects_with_dev_task();
    let mut project_statuses = Vec::new();

    for (name, proj) in &dev_projects {
        if let Some(ref filter) = project {
            if name != filter {
                continue;
            }
        }

        let pane = panes.iter().find(|p| &p.title == name);
        let mut status = "not running".to_string();
        let mut pid: Option<u32> = None;
        let mut uptime_seconds: Option<u64> = None;
        let mut started_at: Option<String> = None;

        if let Some(pane) = pane {
            let active = find_active_pid(pane.pid, &session_name, Some(pane.index)).await;
            if let Some(active_pid) = active {
                pid = Some(active_pid);
                status = "running".to_string();

                let detail = Command::new("ps")
                    .args(["-p", &active_pid.to_string(), "-o", "lstart,etime"])
                    .output()
                    .await;

                if let Ok(detail) = detail {
                    let detail_str = String::from_utf8_lossy(&detail.stdout);
                    let lines: Vec<&str> = detail_str.lines().collect();
                    if lines.len() >= 2 {
                        let parts: Vec<&str> = lines[1].split_whitespace().collect();
                        if parts.len() >= 6 {
                            started_at = Some(parts[..5].join(" "));
                            if let Some(secs) = parse_etime(parts.last().unwrap_or(&"")) {
                                uptime_seconds = Some(secs);
                            }
                        }
                    }
                }
            }
        }

        let tool = proj.tasks.get("dev").map(|t| t.tool.clone());

        let entry = serde_json::json!({
            "name": name,
            "status": status,
            "pid": pid,
            "tool": tool,
            "started_at": started_at,
            "uptime_seconds": uptime_seconds,
        });
        project_statuses.push(entry);
    }

    let output = serde_json::json!({
        "session": session_name,
        "session_active": session_active,
        "projects": project_statuses,
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// View logs for a specific project
pub async fn logs(
    config: &Config,
    project: Option<String>,
    follow: bool,
    lines: usize,
) -> Result<()> {
    // If no project specified, list available logs
    let Some(project) = project else {
        return list_available_logs(config).await;
    };

    // Validate project exists in config
    if !config.projects.contains_key(&project) {
        let available: Vec<&String> = config.projects.keys().collect();
        anyhow::bail!(
            "Unknown project '{}'. Available projects: {}",
            project,
            available
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    let log_path = format!(".meta/logs/{}.log", project);
    let path = std::path::Path::new(&log_path);

    if !path.exists() {
        println!("No log file for '{}' yet.", project);
        println!(
            "\nLogs are created when you run 'meta dev'. The project must output to stdout/stderr."
        );
        return Ok(());
    }

    if follow {
        // Use tail -f for live streaming
        let status = Command::new("tail")
            .args(["-f", "-n", &lines.to_string(), &log_path])
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .await?;

        if !status.success() {
            anyhow::bail!("Failed to tail log file");
        }
    } else {
        // Use tail for efficient reading of last N lines (avoids loading entire file)
        let output = Command::new("tail")
            .args(["-n", &lines.to_string(), &log_path])
            .output()
            .await?;

        if output.status.success() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        } else {
            anyhow::bail!("Failed to read log file");
        }
    }

    Ok(())
}

/// List available log files
async fn list_available_logs(config: &Config) -> Result<()> {
    println!("## Available Project Logs\n");

    let mut has_logs = false;
    for name in config.projects.keys() {
        let log_path = format!(".meta/logs/{}.log", name);
        if std::path::Path::new(&log_path).exists() {
            if let Ok(metadata) = std::fs::metadata(&log_path) {
                has_logs = true;
                let size = metadata.len();
                let size_str = if size >= 1024 * 1024 {
                    format!("{:.1}MB", size as f64 / (1024.0 * 1024.0))
                } else if size >= 1024 {
                    format!("{}KB", size / 1024)
                } else {
                    format!("{}B", size)
                };
                println!("  {} ({}) → meta logs {}", name, size_str, name);
            }
        }
    }

    if !has_logs {
        println!("  (no log files yet)\n");
        println!("Logs are created when you run 'meta dev'.");
        println!("Project stdout/stderr is captured to .meta/logs/<project>.log");
    } else {
        println!("\nUsage:");
        println!("  meta logs <project>           View last 50 lines");
        println!("  meta logs <project> -l 100    View last 100 lines");
        println!("  meta logs <project> --follow  Stream logs in real-time");
    }

    Ok(())
}

/// Get the actual binary name from a Rust project's Cargo.toml
/// Returns None if Cargo.toml doesn't exist or can't be parsed
fn get_rust_binary_name(project_path: &str) -> Option<String> {
    let cargo_path = std::path::Path::new(project_path).join("Cargo.toml");
    let content = std::fs::read_to_string(cargo_path).ok()?;

    // First check for [[bin]] section with name
    // Then fall back to [package] name
    for line in content.lines() {
        let line = line.trim();
        // Look for name = "..." pattern
        if line.starts_with("name") && line.contains('=') {
            if let Some(name_part) = line.split('=').nth(1) {
                let name = name_part.trim().trim_matches('"').trim_matches('\'');
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
    }
    None
}

/// Parse `ps -o etime` format: [[DD-]HH:]MM:SS
/// Examples: "02:30" (2m30s), "01:02:30" (1h2m30s), "2-01:02:30" (2d1h2m30s)
fn parse_etime(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let mut total_secs: u64 = 0;

    // Check for days (format: DD-HH:MM:SS)
    let (days, rest) = if s.contains('-') {
        let parts: Vec<&str> = s.splitn(2, '-').collect();
        let days: u64 = parts[0].parse().ok()?;
        (days, parts.get(1).copied().unwrap_or(""))
    } else {
        (0, s)
    };

    total_secs += days * 24 * 60 * 60;

    // Parse HH:MM:SS or MM:SS
    let time_parts: Vec<&str> = rest.split(':').collect();
    match time_parts.len() {
        2 => {
            // MM:SS
            let mins: u64 = time_parts[0].parse().ok()?;
            let secs: u64 = time_parts[1].parse().ok()?;
            total_secs += mins * 60 + secs;
        }
        3 => {
            // HH:MM:SS
            let hours: u64 = time_parts[0].parse().ok()?;
            let mins: u64 = time_parts[1].parse().ok()?;
            let secs: u64 = time_parts[2].parse().ok()?;
            total_secs += hours * 60 * 60 + mins * 60 + secs;
        }
        _ => return None,
    }

    Some(total_secs)
}

/// Stop all running meta tmux sessions for this workspace
pub async fn dev_stop() -> Result<()> {
    let session_name = get_session_name();

    println!("🛑 Stopping meta development session...\n");

    // Check if session exists
    let list_output = Command::new("tmux")
        .args(["has-session", "-t", &session_name])
        .output()
        .await;

    match list_output {
        Ok(output) if output.status.success() => {
            // Session exists, kill it
            let kill_result = Command::new("tmux")
                .args(["kill-session", "-t", &session_name])
                .output()
                .await?;

            if kill_result.status.success() {
                println!("✅ Stopped tmux session '{}'", session_name);
                println!("\n💡 All development processes have been terminated.");
            } else {
                let stderr = String::from_utf8_lossy(&kill_result.stderr);
                anyhow::bail!("Failed to kill session: {}", stderr);
            }
        }
        Ok(_) => {
            println!("ℹ️  No active {} session found.", session_name);
            println!("\n💡 Use 'meta dev' to start development servers.");
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                println!("⚠️  tmux is not installed.");
                println!("   Install tmux to use the multi-process dev mode.");
            } else {
                anyhow::bail!("Failed to check tmux session: {}", e);
            }
        }
    }

    Ok(())
}

/// List all active meta-* tmux sessions
pub async fn sessions() -> Result<()> {
    let output = Command::new("tmux")
        .args([
            "list-sessions",
            "-F",
            "#{session_name}|#{session_created}|#{session_windows}",
        ])
        .output()
        .await?;

    if !output.status.success() {
        println!("No tmux sessions found.");
        return Ok(());
    }

    let current_session = get_session_name();
    let stdout = String::from_utf8_lossy(&output.stdout);

    println!("## Active Meta Sessions\n");
    let mut found = false;

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 3 && parts[0].starts_with("meta-") {
            found = true;
            let marker = if parts[0] == current_session {
                " (this workspace)"
            } else {
                ""
            };
            println!("  {}{}", parts[0], marker);
            println!("    Panes: {}", parts[2]);
        }
    }

    if !found {
        println!("  (no active meta sessions)");
    }

    Ok(())
}

pub async fn dev(config: &Config, projects: Option<Vec<String>>, detach: bool) -> Result<()> {
    // When no projects specified, use default_dev_projects (respects dev_default flag)
    // When projects are explicitly specified with -p, use those regardless of dev_default
    let projects_to_run = if projects.is_some() {
        get_projects_to_run(config, projects)?
    } else {
        config.default_dev_projects().into_iter().collect()
    };

    let mut commands = Vec::new();

    println!("🚀 Development Commands:\n");

    for (name, project) in &projects_to_run {
        if let Some(dev_task) = project.tasks.get("dev") {
            let tool = config
                .tools
                .get(&dev_task.tool)
                .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", dev_task.tool))?;

            let command_str = dev_task.command.clone();
            let project_path = &project.path;

            // Turborepo runs from workspace root, other tools run from project directory
            let full_command = if tool.command == "turbo" {
                // Turbo runs from workspace root
                format!("{} {}", tool.command, command_str)
            } else {
                // Other tools (bacon, cargo) run from project directory
                format!("cd {} && {} {}", project_path, tool.command, command_str)
            };

            commands.push((name.clone(), full_command.clone()));

            println!("  {} [{}]: {}", name, dev_task.tool, full_command);
        }
    }

    if commands.is_empty() {
        println!("\n⚠️  No dev tasks configured");
        return Ok(());
    }

    println!("\n💡 Launch Options:");
    println!("  1. Manual: Run each command in a separate terminal");
    println!("  2. Tmux: meta will launch all commands in tmux panes (recommended)\n");

    // Auto-launch with tmux if available
    let tmux_available = tokio::process::Command::new("tmux")
        .arg("-V")
        .output()
        .await
        .is_ok();

    if tmux_available && commands.len() > 1 {
        // Detect non-interactive shell (CI, agent, script)
        let is_interactive = std::io::stdin().is_terminal();
        let should_detach = detach || !is_interactive;

        println!(
            "✨ Launching tmux session with {} panes...\n",
            commands.len()
        );
        launch_tmux_session(&commands, should_detach).await?;
    } else if !tmux_available {
        println!("⚠️  tmux not found. Install tmux to automatically launch all commands.");
        println!("   For now, run these commands manually in separate terminals.");
    } else {
        // Only one command, just run it directly
        println!("Running single command: {}\n", commands[0].1);
        let parts: Vec<&str> = commands[0].1.split("&&").collect();
        if parts.len() == 2 {
            let dir = parts[0].trim().strip_prefix("cd ").unwrap_or(".");
            let cmd_parts: Vec<&str> = parts[1].split_whitespace().collect();
            if !cmd_parts.is_empty() {
                let mut cmd = tokio::process::Command::new(cmd_parts[0]);
                cmd.args(&cmd_parts[1..]);
                cmd.current_dir(dir);
                cmd.stdout(std::process::Stdio::inherit());
                cmd.stderr(std::process::Stdio::inherit());
                cmd.stdin(std::process::Stdio::inherit());
                let status = cmd.status().await?;
                if !status.success() {
                    anyhow::bail!("Command failed");
                }
            }
        }
    }

    Ok(())
}

async fn launch_tmux_session(commands: &[(String, String)], detach: bool) -> Result<()> {
    let session_name = get_session_name();

    // Kill existing session if it exists
    let _ = Command::new("tmux")
        .args(["kill-session", "-t", &session_name])
        .output()
        .await;

    // Ensure log directory exists
    let log_dir = std::path::Path::new(".meta/logs");
    if !log_dir.exists() {
        std::fs::create_dir_all(log_dir)?;
    }

    // Wrap commands in a shell that:
    // 1. Logs process start/restart events to .meta/logs/dev.log
    // 2. Keeps the pane alive after the command exits
    // 3. Allows manual restart with Enter
    //
    // Log format (designed for easy parsing by Claude Code):
    //   [TIMESTAMP] [PROJECT] EVENT: message (pid=PID)
    //
    // Example:
    //   [2025-12-08T12:02:18] [api] START: Process started (pid=12345)
    //   [2025-12-08T12:05:32] [api] RESTART: Process restarted after file change
    // (pid=12346)   [2025-12-08T12:05:32] [api] EXIT: Process exited with code
    // 0
    let wrap_command = |name: &str, cmd: &str| -> String {
        let dev_log = ".meta/logs/dev.log";
        let project_log = format!(".meta/logs/{}.log", name);
        // Bacon handles its own logging via tee in bacon.toml, so skip the outer tee
        // to avoid capturing TUI escape codes. Other tools (turbo, cargo) use the tee
        // wrapper.
        let is_bacon = cmd.contains("bacon");
        let run_cmd = if is_bacon {
            // Bacon handles logging internally - just run the command
            format!("{cmd}; EXIT_CODE=$?")
        } else {
            // Other tools: capture stdout/stderr to project log via tee
            format!("{cmd} 2>&1 | tee -a \"$PROJECT_LOG\"; EXIT_CODE=${{PIPESTATUS[0]}}")
        };
        format!(
            r#"DEV_LOG="{dev_log}"; \
PROJECT_LOG="{project_log}"; \
PROJECT="{name}"; \
log_event() {{ echo "[$(date -u +%Y-%m-%dT%H:%M:%S)] [$PROJECT] $1" >> "$DEV_LOG"; }}; \
rotate_log() {{ \
  if [ -f "$PROJECT_LOG" ]; then \
    SIZE=$(stat -f%z "$PROJECT_LOG" 2>/dev/null || stat -c%s "$PROJECT_LOG" 2>/dev/null || echo 0); \
    if [ "$SIZE" -gt 10485760 ]; then \
      mv "$PROJECT_LOG" "$PROJECT_LOG.1"; \
    fi; \
  fi; \
}}; \
run_with_logging() {{ \
  rotate_log; \
  log_event "START: Process started (pid=$$)"; \
  {run_cmd}; \
  log_event "EXIT: Process exited with code $EXIT_CODE"; \
  return $EXIT_CODE; \
}}; \
run_with_logging; \
while true; do \
  echo '\n✓ Process exited. Press Enter to restart or Ctrl+C to close.'; \
  read -r; \
  log_event "RESTART: Manual restart triggered"; \
  run_with_logging; \
done"#,
            dev_log = dev_log,
            project_log = project_log,
            name = name,
            run_cmd = run_cmd
        )
    };

    // Create new session with first command
    let first_cmd = &commands[0];
    let wrapped_first = wrap_command(&first_cmd.0, &first_cmd.1);
    Command::new("tmux")
        .args(["new-session", "-d", "-s", &session_name, "-n", &first_cmd.0])
        .arg(&wrapped_first)
        .output()
        .await?;

    // Add remaining commands as new panes
    for (name, cmd) in commands.iter().skip(1) {
        let wrapped_cmd = wrap_command(name, cmd);
        Command::new("tmux")
            .args(["split-window", "-t", &session_name, "-h"])
            .arg(&wrapped_cmd)
            .output()
            .await?;

        Command::new("tmux")
            .args(["select-pane", "-t", &session_name, "-T", name])
            .output()
            .await?;
    }

    // Set up tmux pipe-pane for all panes to capture logs.
    // Uses sed to strip ANSI escape codes (colors, cursor movement, erase commands)
    // so logs are readable even from TUI tools like bacon.
    // The sed pattern handles: SGR (m), cursor position (H/G), erase (J/K),
    // and other CSI sequences.
    for (i, (name, _)) in commands.iter().enumerate() {
        let log_path = format!(".meta/logs/{}.log", name);
        let pane_target = format!("{}:{}.{}", session_name, 0, i);
        let pipe_cmd = format!(
            "exec cat - | sed -l 's/\x1b\\[[0-9;]*[mGKHJsu]//g' >> '{}'",
            log_path
        );
        Command::new("tmux")
            .args(["pipe-pane", "-t", &pane_target, &pipe_cmd])
            .output()
            .await?;
    }

    // Tile the panes evenly
    Command::new("tmux")
        .args(["select-layout", "-t", &session_name, "tiled"])
        .output()
        .await?;

    if detach {
        // Detached mode: session is running in background
        println!("✅ Tmux session '{}' started in background.", session_name);
        println!("   Attach:  tmux attach -t {}", session_name);
        println!("   Status:  meta status");
        println!("   Stop:    meta dev:stop");
    } else {
        // Interactive mode: attach to the session
        println!("📺 Attaching to tmux session '{}'...", session_name);
        println!("\n╭─────────────────────────────────────────────────────────╮");
        println!("│ 🎮 Tmux Navigation Guide                                │");
        println!("├─────────────────────────────────────────────────────────┤");
        println!("│ Navigate Panes:  Ctrl+B then Arrow Keys (← → ↑ ↓)      │");
        println!("│ Zoom Pane:       Ctrl+B then Z (toggle full screen)    │");
        println!("│ Show Numbers:    Ctrl+B then Q (then press number)     │");
        println!("│                                                         │");
        println!("│ Detach Session:  Ctrl+B then D (keeps running)         │");
        println!("│ Stop Process:    Ctrl+C (in current pane)              │");
        println!("│ Close Pane:      Ctrl+B then X (confirm with y)        │");
        println!("╰─────────────────────────────────────────────────────────╯\n");

        let status = Command::new("tmux")
            .args(["attach-session", "-t", &session_name])
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .await?;

        if !status.success() {
            // Provide a helpful message instead of a scary error
            println!(
                "ℹ️  Tmux session '{}' is running (could not attach — no terminal).",
                session_name
            );
            println!("   Attach with: tmux attach -t {}", session_name);
            println!("   Check status: meta status");
        }
    }

    Ok(())
}

pub async fn build(config: &Config, _prod: bool, projects: Option<Vec<String>>) -> Result<()> {
    let projects_to_build = get_projects_to_run(config, projects)?;

    println!("🔨 Building projects...\n");

    for (name, project) in projects_to_build {
        if let Some(build_task) = project.tasks.get("build") {
            let tool = config
                .tools
                .get(&build_task.tool)
                .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", build_task.tool))?;

            let adapter = ToolAdapter::new(build_task.tool.clone(), tool.command.clone());

            println!("  → Building {} ({})", name, build_task.tool);

            let parts: Vec<&str> = build_task.command.split_whitespace().collect();

            // Use project path for execution
            let project_path = std::path::Path::new(&project.path);
            adapter.execute_in(&parts, project_path).await?;
        }
    }

    println!("\n✅ Build complete!\n");
    Ok(())
}

pub async fn test(config: &Config, _watch: bool) -> Result<()> {
    println!("🧪 Running tests...\n");

    for (name, project) in &config.projects {
        if let Some(test_task) = project.tasks.get("test") {
            let tool = config
                .tools
                .get(&test_task.tool)
                .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", test_task.tool))?;

            let adapter = ToolAdapter::new(test_task.tool.clone(), tool.command.clone());

            println!("  → Testing {} ({})", name, test_task.tool);

            let parts: Vec<&str> = test_task.command.split_whitespace().collect();

            // Use project path for execution
            let project_path = std::path::Path::new(&project.path);
            adapter.execute_in(&parts, project_path).await?;
        }
    }

    println!("\n✅ Tests complete!\n");
    Ok(())
}

pub async fn run_task(
    config: &Config,
    task_name: &str,
    projects: Option<Vec<String>>,
) -> Result<()> {
    let projects_to_run = get_projects_to_run(config, projects)?;

    println!("🚀 Running task '{}'...\n", task_name);

    for (name, project) in projects_to_run {
        if let Some(task) = project.tasks.get(task_name) {
            let tool = config
                .tools
                .get(&task.tool)
                .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", task.tool))?;

            let adapter = ToolAdapter::new(task.tool.clone(), tool.command.clone());

            println!("  → {} ({})", name, task.tool);

            let parts: Vec<&str> = task.command.split_whitespace().collect();

            // Use project path for execution
            let project_path = std::path::Path::new(&project.path);
            adapter.execute_in(&parts, project_path).await?;
        } else {
            println!("  ⊘ {} (task '{}' not defined, skipping)", name, task_name);
        }
    }

    println!("\n✅ Task '{}' complete!\n", task_name);
    Ok(())
}

fn get_projects_to_run(
    config: &Config,
    projects: Option<Vec<String>>,
) -> Result<HashMap<String, &crate::config::ProjectConfig>> {
    let mut result = HashMap::new();

    match projects {
        Some(names) => {
            for name in names {
                let project = config
                    .projects
                    .get(&name)
                    .ok_or_else(|| anyhow::anyhow!("Project not found: {}", name))?;
                result.insert(name, project);
            }
        }
        None => {
            for (name, project) in &config.projects {
                result.insert(name.clone(), project);
            }
        }
    }

    Ok(result)
}
/// Check if a project has a specific task configured (used by doctor)
pub fn project_has_task(project: &crate::config::ProjectConfig, task_name: &str) -> bool {
    project.tasks.contains_key(task_name)
}

/// Get binary path for a Rust project, respecting Cargo workspace layout.
/// If workspace_root is provided, looks in <workspace_root>/target/debug/<name>.
/// Otherwise falls back to <project_path>/target/debug/<name>.
pub fn get_rust_binary_path(
    project_path: &str,
    binary_name: &str,
    workspace_root: Option<&str>,
) -> String {
    if let Some(root) = workspace_root {
        format!("{}/target/debug/{}", root, binary_name)
    } else {
        format!("{}/target/debug/{}", project_path, binary_name)
    }
}

/// Detect if a project is part of a Cargo workspace by checking for
/// [workspace] in a root Cargo.toml or workspace key in project Cargo.toml
pub fn detect_cargo_workspace(project_path: &str) -> Option<String> {
    // First, check if the project's own Cargo.toml has a `workspace` member key
    // pointing to a parent
    let project_cargo = std::path::Path::new(project_path).join("Cargo.toml");
    if let Ok(content) = std::fs::read_to_string(&project_cargo) {
        // If the project Cargo.toml has [workspace], it IS the workspace root
        if content.contains("[workspace]") {
            return Some(project_path.to_string());
        }
    }

    // Walk up to find a workspace root Cargo.toml
    let mut current = std::path::Path::new(project_path).to_path_buf();
    while let Some(parent) = current.parent() {
        let parent_cargo = parent.join("Cargo.toml");
        if parent_cargo.exists() {
            if let Ok(content) = std::fs::read_to_string(&parent_cargo) {
                if content.contains("[workspace]") {
                    return Some(parent.to_string_lossy().to_string());
                }
            }
        }
        if parent.as_os_str().is_empty() || parent == current {
            break;
        }
        current = parent.to_path_buf();
    }
    None
}

/// Check if a Rust project is a library crate (no binary output)
pub fn is_library_crate(project_path: &str) -> bool {
    let cargo_path = std::path::Path::new(project_path).join("Cargo.toml");
    if let Ok(content) = std::fs::read_to_string(cargo_path) {
        // Has [[bin]] section → not a library
        if content.contains("[[bin]]") {
            return false;
        }
        // Has [lib] but no [[bin]] → library
        if content.contains("[lib]") {
            return true;
        }
        // Check for src/main.rs — if it exists, it's a binary
        let main_rs = std::path::Path::new(project_path).join("src/main.rs");
        if main_rs.exists() {
            return false;
        }
        // No main.rs and no [[bin]] → library
        return true;
    }
    false
}

/// Validate that a bacon-based task has a valid bacon.toml configuration
/// Returns true if `.mcp.json` at `path` contains an mcpServers entry whose
/// `command` field is `"docker"`. Missing/invalid files return false.
pub fn mcp_json_references_docker(path: &std::path::Path) -> bool {
    let Ok(contents) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&contents) else {
        return false;
    };
    value
        .get("mcpServers")
        .and_then(|s| s.as_object())
        .map(|servers| {
            servers
                .values()
                .any(|entry| entry.get("command").and_then(|c| c.as_str()) == Some("docker"))
        })
        .unwrap_or(false)
}

pub fn validate_bacon_config(project_path: &str, command: &str) -> Vec<String> {
    let mut warnings = Vec::new();

    // Check for bacon.toml in the project directory
    let bacon_toml = std::path::Path::new(project_path).join("bacon.toml");
    let bacon_prefs = std::path::Path::new(project_path).join(".bacon.toml");

    if !bacon_toml.exists() && !bacon_prefs.exists() {
        // Also check workspace root
        if let Some(ws_root) = detect_cargo_workspace(project_path) {
            let ws_bacon = std::path::Path::new(&ws_root).join("bacon.toml");
            let ws_bacon_prefs = std::path::Path::new(&ws_root).join(".bacon.toml");
            if !ws_bacon.exists() && !ws_bacon_prefs.exists() {
                warnings.push(format!(
                    "bacon.toml not found in '{}' or workspace root — bacon may fail silently",
                    project_path
                ));
            }
        } else {
            warnings.push(format!(
                "bacon.toml not found in '{}' — bacon may fail silently",
                project_path
            ));
        }
    }

    // Check if the referenced job exists in bacon.toml
    let job_name = command.split_whitespace().next().unwrap_or(command);
    if bacon_toml.exists() {
        if let Ok(content) = std::fs::read_to_string(&bacon_toml) {
            let job_header = format!("[jobs.{}]", job_name);
            if !content.contains(&job_header) && !content.contains(&format!("[jobs.{job_name}]")) {
                // Built-in jobs don't need to be in bacon.toml
                let builtin_jobs = ["check", "clippy", "test", "doc", "run", "run-long"];
                if !builtin_jobs.contains(&job_name) {
                    warnings.push(format!(
                        "job '{}' not found in bacon.toml — bacon will fail",
                        job_name
                    ));
                }
            }
        }
    }

    warnings
}

pub async fn doctor(config: &Config) -> Result<()> {
    println!("🏥 Meta Doctor - Configuration Diagnostics\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut errors = 0;
    let mut warnings = 0;

    // Check meta.toml exists and is valid
    println!("📋 Configuration File:");
    println!("  ✓ meta.toml loaded successfully");
    println!("  ✓ Workspace: {}\n", config.workspace.name);

    // Check tools
    println!("🔧 Tool Availability:");
    for (tool_name, tool_config) in &config.tools {
        if !tool_config.enabled {
            println!("  ⊘ {} (disabled)", tool_name);
            continue;
        }

        match tokio::process::Command::new(&tool_config.command)
            .arg("--version")
            .output()
            .await
        {
            Ok(output) if output.status.success() => {
                let version_str = String::from_utf8_lossy(&output.stdout);
                let version = version_str.lines().next().unwrap_or("unknown").trim();
                println!("  ✓ {} → {} ({})", tool_name, tool_config.command, version);
            }
            Ok(_) => {
                println!(
                    "  ⚠ {} → {} (found but version check failed)",
                    tool_name, tool_config.command
                );
                warnings += 1;
            }
            Err(_) => {
                println!("  ✗ {} → {} (NOT FOUND)", tool_name, tool_config.command);
                errors += 1;
            }
        }
    }

    // Check tmux for multi-process support
    println!("\n🖥️  Terminal Multiplexer:");
    match tokio::process::Command::new("tmux")
        .arg("-V")
        .output()
        .await
    {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("  ✓ tmux → {} (multi-process dev mode available)", version);
        }
        _ => {
            println!("  ⚠ tmux not found (install for 'meta dev' multi-process mode)");
            println!("    Install: brew install tmux (macOS) or apt install tmux (Linux)");
            warnings += 1;
        }
    }

    // Check MCP integration (docker required if .mcp.json uses it)
    let mcp_path = std::path::Path::new(".mcp.json");
    if mcp_path.exists() && mcp_json_references_docker(mcp_path) {
        println!("\n🐳 MCP Integration:");
        match tokio::process::Command::new("docker")
            .arg("--version")
            .output()
            .await
        {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                println!("  ✓ docker → {} (.mcp.json references docker)", version);
            }
            _ => {
                println!("  ⚠ .mcp.json references docker but docker not found on PATH");
                println!("    Install Docker to use mcp-log-server, or re-run 'meta init --no-mcp' to opt out");
                warnings += 1;
            }
        }
    }

    // Check projects
    println!("\n📦 Projects ({}):", config.projects.len());
    for (name, project) in &config.projects {
        let path = std::path::Path::new(&project.path);
        if path.exists() {
            println!("  ✓ {} → {} ({})", name, project.path, project.project_type);

            // Check if project has dev task (per-project, not path-based)
            if project_has_task(project, "dev") {
                if !project.dev_default {
                    println!("    • dev task configured (excluded from default 'meta dev', use -p to include)");
                } else {
                    println!("    • dev task configured");
                }

                // Issue #3: validate bacon config if tool is bacon
                if let Some(dev_task) = project.tasks.get("dev") {
                    if dev_task.tool == "bacon" {
                        let bacon_warnings =
                            validate_bacon_config(&project.path, &dev_task.command);
                        for warning in bacon_warnings {
                            println!("    ⚠ {}", warning);
                            warnings += 1;
                        }
                    }
                }
            }
        } else {
            println!("  ✗ {} → {} (PATH NOT FOUND)", name, project.path);
            errors += 1;
        }
    }

    // Check for common issues
    println!("\n🔍 Configuration Validation:");

    // Validate turborepo commands
    for (name, project) in &config.projects {
        if let Some(dev_task) = project.tasks.get("dev") {
            if dev_task.tool == "turborepo" {
                let cmd = &dev_task.command;
                if !cmd.starts_with("run ") {
                    println!("  ⚠ {} dev task should start with 'run': '{}'", name, cmd);
                    println!("    Suggested: 'run dev --filter=...'");
                    warnings += 1;
                }
                if !cmd.contains("--filter=") {
                    println!("  ⚠ {} turbo task missing --filter flag: '{}'", name, cmd);
                    warnings += 1;
                }
            }
        }
    }

    // Summary
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\n📊 Summary:");

    if errors == 0 && warnings == 0 {
        println!("  🎉 All checks passed! Meta is ready to use.");
        println!("\n💡 Quick Start:");
        println!("  • Run 'meta dev' to start all development servers");
        println!("  • Run 'meta dev --projects api' to start specific project");
        println!("  • Run 'meta run <task>' to execute any task across projects");
    } else {
        if errors > 0 {
            println!("  ✗ {} error(s) found - these must be fixed", errors);
        }
        if warnings > 0 {
            println!(
                "  ⚠ {} warning(s) - meta will work but with reduced functionality",
                warnings
            );
        }
    }

    println!();

    if errors > 0 {
        anyhow::bail!("Configuration has errors");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Issue #7: doctor dev task detection ===

    #[test]
    fn test_project_has_task_returns_true_when_present() {
        let config = crate::config::parse(
            r#"
version = "1"
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
        let project = &config.projects["api"];
        assert!(project_has_task(project, "dev"));
    }

    #[test]
    fn test_project_has_task_returns_false_when_absent() {
        let config = crate::config::parse(
            r#"
version = "1"
[workspace]
name = "Test"
root = "."
[tools.cargo]
enabled = true
command = "cargo"
[projects.shared]
type = "rust"
path = "crates/shared"
[projects.shared.tasks]
build = { tool = "cargo", command = "build" }
"#,
        )
        .unwrap();
        let project = &config.projects["shared"];
        assert!(!project_has_task(project, "dev"));
    }

    #[test]
    fn test_projects_sharing_path_independent_task_check() {
        let config = crate::config::parse(
            r#"
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
"#,
        )
        .unwrap();

        // trainee-app has dev task
        assert!(project_has_task(&config.projects["trainee-app"], "dev"));
        // trainee-android does NOT have dev task (same path, different project)
        assert!(!project_has_task(
            &config.projects["trainee-android"],
            "dev"
        ));
    }

    // === Issue #6: binary path detection ===

    #[test]
    fn test_binary_path_without_workspace() {
        let path = get_rust_binary_path("apps/api", "api", None);
        assert_eq!(path, "apps/api/target/debug/api");
    }

    #[test]
    fn test_binary_path_with_workspace() {
        let path = get_rust_binary_path("apps/api", "api", Some("."));
        assert_eq!(path, "./target/debug/api");
    }

    #[test]
    fn test_binary_path_with_workspace_root() {
        let path = get_rust_binary_path("apps/api", "api", Some("/home/user/project"));
        assert_eq!(path, "/home/user/project/target/debug/api");
    }

    // === Issue #6: library crate detection ===

    #[test]
    fn test_is_library_crate_with_lib_section() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_path = temp_dir.path();
        let src_dir = project_path.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("lib.rs"), "pub fn hello() {}").unwrap();
        std::fs::write(
            project_path.join("Cargo.toml"),
            r#"[package]
name = "shared"
version = "0.1.0"

[lib]
name = "shared"
"#,
        )
        .unwrap();

        assert!(is_library_crate(&project_path.to_string_lossy()));
    }

    #[test]
    fn test_is_not_library_crate_with_main_rs() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_path = temp_dir.path();
        let src_dir = project_path.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();
        std::fs::write(
            project_path.join("Cargo.toml"),
            r#"[package]
name = "api"
version = "0.1.0"
"#,
        )
        .unwrap();

        assert!(!is_library_crate(&project_path.to_string_lossy()));
    }

    #[test]
    fn test_is_not_library_crate_with_bin_section() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_path = temp_dir.path();
        std::fs::write(
            project_path.join("Cargo.toml"),
            r#"[package]
name = "api"
version = "0.1.0"

[[bin]]
name = "api"
path = "src/main.rs"
"#,
        )
        .unwrap();

        assert!(!is_library_crate(&project_path.to_string_lossy()));
    }

    // === Issue #6: workspace detection ===

    #[test]
    fn test_detect_cargo_workspace_finds_workspace_root() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create workspace root Cargo.toml
        std::fs::write(
            root.join("Cargo.toml"),
            r#"[workspace]
members = ["apps/api"]
"#,
        )
        .unwrap();

        // Create project Cargo.toml
        let api_path = root.join("apps/api");
        std::fs::create_dir_all(&api_path).unwrap();
        std::fs::write(
            api_path.join("Cargo.toml"),
            r#"[package]
name = "api"
version = "0.1.0"
"#,
        )
        .unwrap();

        let ws = detect_cargo_workspace(&api_path.to_string_lossy());
        assert!(ws.is_some());
        assert_eq!(ws.unwrap(), root.to_string_lossy().to_string());
    }

    #[test]
    fn test_detect_cargo_workspace_returns_none_without_workspace() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_path = temp_dir.path();
        std::fs::write(
            project_path.join("Cargo.toml"),
            r#"[package]
name = "standalone"
version = "0.1.0"
"#,
        )
        .unwrap();

        let ws = detect_cargo_workspace(&project_path.to_string_lossy());
        assert!(ws.is_none());
    }

    // === Issue #3: bacon config validation ===

    #[test]
    fn test_validate_bacon_config_warns_when_no_bacon_toml() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_path = temp_dir.path().to_string_lossy().to_string();

        let warnings = validate_bacon_config(&project_path, "run-long");
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("bacon.toml not found"));
    }

    #[test]
    fn test_validate_bacon_config_no_warning_when_bacon_toml_exists() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create bacon.toml with the job
        std::fs::write(
            project_path.join("bacon.toml"),
            r#"[jobs.run-long]
command = ["cargo", "run"]
"#,
        )
        .unwrap();

        let warnings = validate_bacon_config(&project_path.to_string_lossy(), "run-long");
        assert!(warnings.is_empty(), "unexpected warnings: {:?}", warnings);
    }

    #[test]
    fn test_validate_bacon_config_builtin_jobs_no_warning() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create empty bacon.toml (builtin jobs don't need config)
        std::fs::write(project_path.join("bacon.toml"), "").unwrap();

        // "run-long" is a builtin job
        let warnings = validate_bacon_config(&project_path.to_string_lossy(), "run-long");
        assert!(warnings.is_empty(), "unexpected warnings: {:?}", warnings);
    }

    // === mcp_json_references_docker ===

    #[test]
    fn test_mcp_references_docker_true_when_command_is_docker() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join(".mcp.json");
        std::fs::write(
            &path,
            r#"{"mcpServers":{"mcp-log-server":{"command":"docker","args":[]}}}"#,
        )
        .unwrap();
        assert!(mcp_json_references_docker(&path));
    }

    #[test]
    fn test_mcp_references_docker_false_when_no_docker_entry() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join(".mcp.json");
        std::fs::write(
            &path,
            r#"{"mcpServers":{"other":{"command":"npx","args":[]}}}"#,
        )
        .unwrap();
        assert!(!mcp_json_references_docker(&path));
    }

    #[test]
    fn test_mcp_references_docker_false_when_file_missing() {
        let tmp = tempfile::TempDir::new().unwrap();
        assert!(!mcp_json_references_docker(&tmp.path().join("absent.json")));
    }

    #[test]
    fn test_mcp_references_docker_false_when_malformed() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join(".mcp.json");
        std::fs::write(&path, "not json").unwrap();
        assert!(!mcp_json_references_docker(&path));
    }

    // === Existing: parse_etime ===

    #[test]
    fn test_parse_etime_minutes_seconds() {
        assert_eq!(parse_etime("02:30"), Some(150));
    }

    #[test]
    fn test_parse_etime_hours_minutes_seconds() {
        assert_eq!(parse_etime("01:02:30"), Some(3750));
    }

    #[test]
    fn test_parse_etime_days() {
        assert_eq!(parse_etime("2-01:02:30"), Some(2 * 86400 + 3750));
    }

    #[test]
    fn test_parse_etime_empty() {
        assert_eq!(parse_etime(""), None);
    }

    // === Issue #4: process tree building ===

    #[test]
    fn test_build_process_tree_basic() {
        let ps_output =
            "  PID  PPID\n    1     0\n  100     1\n  200   100\n  300   100\n  400   200\n";
        let tree = build_process_tree(ps_output);

        assert_eq!(tree.get(&0), Some(&vec![1]));
        assert_eq!(tree.get(&1), Some(&vec![100]));
        assert!(tree.get(&100).unwrap().contains(&200));
        assert!(tree.get(&100).unwrap().contains(&300));
        assert_eq!(tree.get(&200), Some(&vec![400]));
    }

    #[test]
    fn test_build_process_tree_empty() {
        let ps_output = "  PID  PPID\n";
        let tree = build_process_tree(ps_output);
        assert!(tree.is_empty());
    }

    #[test]
    fn test_collect_descendants_full_tree() {
        let ps_output =
            "  PID  PPID\n    1     0\n  100     1\n  200   100\n  300   100\n  400   200\n";
        let tree = build_process_tree(ps_output);
        let mut descendants = collect_descendants(&tree, 100);
        descendants.sort();
        assert_eq!(descendants, vec![200, 300, 400]);
    }

    #[test]
    fn test_collect_descendants_leaf_node() {
        let ps_output = "  PID  PPID\n    1     0\n  100     1\n  200   100\n";
        let tree = build_process_tree(ps_output);
        let descendants = collect_descendants(&tree, 200);
        assert!(descendants.is_empty());
    }

    #[test]
    fn test_collect_descendants_nonexistent_pid() {
        let ps_output = "  PID  PPID\n    1     0\n  100     1\n";
        let tree = build_process_tree(ps_output);
        let descendants = collect_descendants(&tree, 999);
        assert!(descendants.is_empty());
    }

    // Simulates the bacon process tree:
    // shell(100) → bacon(200) → cargo(300) → binary(400)
    #[test]
    fn test_collect_descendants_bacon_tree() {
        let ps_output = "  PID  PPID\n  100     1\n  200   100\n  300   200\n  400   300\n";
        let tree = build_process_tree(ps_output);
        let descendants = collect_descendants(&tree, 100);
        // Should find all: bacon, cargo, binary
        assert_eq!(descendants.len(), 3);
        assert!(descendants.contains(&200)); // bacon
        assert!(descendants.contains(&300)); // cargo
        assert!(descendants.contains(&400)); // binary
    }

    // When shell exits and children are reparented to launchd (pid 1),
    // the old shell PID has no children in the tree
    #[test]
    fn test_collect_descendants_orphaned_to_launchd() {
        // Shell (pid 100) exited. bacon (200) and cargo (300) reparented to pid 1.
        let ps_output = "  PID  PPID\n    1     0\n  200     1\n  300   200\n  400   300\n";
        let tree = build_process_tree(ps_output);
        // Looking for descendants of the dead shell PID 100 — finds nothing
        let descendants = collect_descendants(&tree, 100);
        assert!(descendants.is_empty());
        // But descendants of pid 1 would include the reparented processes
        let launchd_children = collect_descendants(&tree, 1);
        assert!(launchd_children.contains(&200));
    }
}
