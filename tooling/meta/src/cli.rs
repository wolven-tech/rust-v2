use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "meta")]
#[command(about = "Meta task orchestrator for monorepos", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize meta configuration
    Init {
        /// Skip writing an mcp-log-server entry to .mcp.json
        #[arg(long = "no-mcp")]
        no_mcp: bool,
    },

    /// Start development servers for all projects
    Dev {
        /// Specific projects to run (optional)
        #[arg(short, long)]
        projects: Option<Vec<String>>,

        /// Start in background without attaching to tmux (useful for CI/agents)
        #[arg(short, long)]
        detach: bool,
    },

    /// Stop all running tmux development sessions
    #[command(name = "dev:stop")]
    DevStop,

    /// Show status of running dev processes (useful for Claude Code)
    ///
    /// Displays process info, restart history, and binary modification times.
    /// Log file: .meta/logs/dev.log
    Status {
        /// Show only entries for specific project
        #[arg(short, long)]
        project: Option<String>,

        /// Number of recent log entries to show (default: 20)
        #[arg(short, long, default_value = "20")]
        lines: usize,

        /// Output JSON for programmatic consumption
        #[arg(long)]
        json: bool,
    },

    /// Build projects
    Build {
        /// Production build
        #[arg(long)]
        prod: bool,

        /// Specific projects to build (optional)
        #[arg(short, long)]
        projects: Option<Vec<String>>,
    },

    /// Run tests
    Test {
        /// Watch mode
        #[arg(short, long)]
        watch: bool,
    },

    /// Run a specific task (e.g., meta run fmt, meta run clippy)
    Run {
        /// Task name to run
        task: String,

        /// Specific projects to run task for (optional)
        #[arg(short, long)]
        projects: Option<Vec<String>>,
    },

    /// Validate meta.toml configuration and check tool availability
    Doctor,

    /// View project logs
    ///
    /// Shows stdout/stderr captured from dev processes.
    /// Without a project name, lists available log files.
    Logs {
        /// Project name to view logs for (optional - lists available if
        /// omitted)
        project: Option<String>,

        /// Follow log output (like tail -f)
        #[arg(short, long)]
        follow: bool,

        /// Number of lines to show (default: 50)
        #[arg(short, long, default_value = "50")]
        lines: usize,
    },

    /// List all active meta tmux sessions
    ///
    /// Shows all meta-* tmux sessions across different workspaces.
    /// Helpful for managing multiple development environments.
    Sessions,
}
