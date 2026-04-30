//! `cargo xtask snapshots {review|accept|prune}` — thin wrappers around `cargo insta`.
//!
//! Value is in discoverability via `cargo xtask --help`; the underlying
//! commands already do the right thing (workspace-aware discovery, paired
//! binary-file cleanup, interactive TUI for review).

use std::process::Command;

use anyhow::{Result, bail};

pub fn review() -> Result<()> {
    run_insta(&["review", "--workspace"])
}

pub fn accept() -> Result<()> {
    run_insta(&["accept", "--workspace"])
}

pub fn prune() -> Result<()> {
    run_insta(&["test", "--workspace", "--unreferenced", "delete"])
}

fn run_insta(args: &[&str]) -> Result<()> {
    let status = Command::new("cargo").arg("insta").args(args).status()?;
    if !status.success() {
        bail!("cargo insta {} exited with {}", args[0], status);
    }
    Ok(())
}
