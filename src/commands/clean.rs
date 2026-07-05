use super::Command;
use crate::fs::rimraf;
use crate::info;
use crate::rgl::{
    get_app_data_dot_regolith_dir, get_current_dir, get_filters_cache_dir, get_project_cache_dir,
    get_repo_cache_dir, Config, Session,
};
use anyhow::Result;
use clap::Args;

/// Clean the current project's cache and build files
#[derive(Args)]
pub struct Clean {
    /// Clear all caches stored in the user's app data directory, instead of the cache of the
    /// current project
    #[arg(short, long)]
    user_cache: bool,
    /// Clear the shared filter repository cache stored in the user's app data directory,
    /// instead of the cache of the current project
    #[arg(long)]
    filter_cache: bool,
}

impl Command for Clean {
    fn dispatch(&self) -> Result<()> {
        if self.user_cache {
            info!("Cleaning all rgl cache files from user app data...");
            let project_cache_dir = get_project_cache_dir()?;
            rimraf(&project_cache_dir)?;
            info!(
                "Cleared project caches in <b>{}</>",
                project_cache_dir.display()
            );
            return Ok(());
        }
        if self.filter_cache {
            info!("Cleaning the filter repository cache from user app data...");
            let repo_cache_dir = get_repo_cache_dir()?;
            rimraf(&repo_cache_dir)?;
            info!(
                "Cleared filter repository cache in <b>{}</>",
                repo_cache_dir.display()
            );
            // Also clear the cache of already-installed filters, mirroring Go's
            // CleanFilterCache which wipes its whole filter cache tree.
            let filters_cache_dir = get_filters_cache_dir()?;
            rimraf(&filters_cache_dir)?;
            info!(
                "Cleared installed filter cache in <b>{}</>",
                filters_cache_dir.display()
            );
            return Ok(());
        }
        // Make sure it's a valid project
        let _ = Config::load()?;
        let mut session = Session::lock()?;
        info!("Cleaning .regolith folder...");
        // Clean both possible locations of the `.regolith` dir, regardless of the
        // `use_project_app_data_storage` setting, so toggling it doesn't leave stale
        // caches behind. Mirrors Regolith's `CleanCurrentProject`.
        rimraf(get_current_dir()?.join(".regolith"))?;
        rimraf(get_app_data_dot_regolith_dir()?)?;
        info!("Cleaning build files...");
        rimraf("build")?;
        info!("Completed!");
        session.unlock()
    }
    fn error_context(&self) -> String {
        "Error cleaning files".to_owned()
    }
}
