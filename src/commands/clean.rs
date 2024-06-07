use super::Command;
use crate::fs::rimraf;
use crate::info;
use crate::rgl::{Config, Session};
use anyhow::Result;
use clap::Args;

/// Clean the current project's cache and build files
#[derive(Args)]
pub struct Clean;

impl Command for Clean {
    fn dispatch(&self) -> Result<()> {
        // Make sure it's a valid project
        let _ = Config::load()?;
        let mut session = Session::lock()?;
        info!("Cleaning .regolith folder...");
        rimraf(".regolith")?;
        info!("Cleaning build files...");
        rimraf("build")?;
        info!("Completed!");
        session.unlock()
    }
    fn error_context(&self) -> String {
        "Error cleaning files".to_owned()
    }
}
