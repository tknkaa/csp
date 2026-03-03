mod app;
mod loader;
mod session;
mod tui;
mod ui;

use anyhow::{Context, Result};
use std::{path::PathBuf, process::Command};

fn main() -> Result<()> {
    let home = std::env::var("HOME").context("HOME not set")?;
    let session_dir = PathBuf::from(&home).join(".copilot").join("session-state");

    if !session_dir.exists() {
        eprintln!("Session directory not found: {}", session_dir.display());
        std::process::exit(1);
    }

    let sessions = loader::load_sessions(&session_dir)?;
    if sessions.is_empty() {
        eprintln!("No sessions found.");
        std::process::exit(1);
    }

    let Some(sel) = tui::run_tui(sessions)? else {
        return Ok(());
    };

    // cd してから exec
    let work_dir = sel
        .cwd
        .as_deref()
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .unwrap_or_else(|| PathBuf::from(&home));

    eprintln!("cd {}", work_dir.display());
    eprintln!("Resuming {}…", &sel.id[..8.min(sel.id.len())]);

    std::env::set_current_dir(&work_dir)?;

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = Command::new("copilot")
            .arg(format!("--resume={}", sel.id))
            .exec();
        eprintln!("exec failed: {}", err);
        std::process::exit(1);
    }

    #[cfg(not(unix))]
    {
        Command::new("copilot")
            .arg(format!("--resume={}", sel.id))
            .status()?;
        Ok(())
    }
}
