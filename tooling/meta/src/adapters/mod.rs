use std::{path::Path, process::Stdio};

use anyhow::Result;
use tokio::process::Command;

pub struct ToolAdapter {
    pub name: String,
    pub command: String,
}

impl ToolAdapter {
    pub fn new(name: String, command: String) -> Self {
        Self { name, command }
    }

    pub async fn execute_in(&self, args: &[&str], working_dir: &Path) -> Result<()> {
        let mut cmd = Command::new(&self.command);
        cmd.args(args);
        cmd.current_dir(working_dir);
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());

        // Enable colored output
        cmd.env("CARGO_TERM_COLOR", "always");
        cmd.env("FORCE_COLOR", "1");

        let status = cmd.status().await?;

        if !status.success() {
            anyhow::bail!(
                "{} command failed: {} {} (in {})",
                self.name,
                self.command,
                args.join(" "),
                working_dir.display()
            );
        }

        Ok(())
    }
}
