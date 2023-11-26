use super::{rimraf, Config, Session};
use crate::info;
use anyhow::{Context, Result};

pub fn clean() -> Result<()> {
    // Make sure it's a regolith project
    let _ = Config::load().context("Not a Regolith project")?;
    let mut session = Session::lock()?;
    info!("Cleaning .regolith folder...");
    rimraf(".regolith")?;
    info!("Cleaning build files...");
    rimraf("build")?;
    info!("Completed!");
    session.unlock()
}
