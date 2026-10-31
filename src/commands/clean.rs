use super::Command;
use crate::fs::rimraf;
use crate::info;
use crate::rgl::{
    get_filter_cache_dir, get_repo_cache_dir, get_resolver_cache_dir, Config, Session,
};
use anyhow::Result;
use clap::Args;

/// Clean the current project's cache and build files
#[derive(Args)]
pub struct Clean {
    /// Clean the global cache instead of the current project
    #[arg(long)]
    global_cache: bool,
}

impl Command for Clean {
    fn dispatch(&self) -> Result<()> {
        if self.global_cache {
            info!("Cleaning filter cache...");
            rimraf(get_filter_cache_dir()?)?;
            info!("Cleaning repo cache...");
            rimraf(get_repo_cache_dir()?)?;
            info!("Cleaning resolver cache...");
            rimraf(get_resolver_cache_dir()?)?;
            info!("Completed!");
            return Ok(());
        }

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
