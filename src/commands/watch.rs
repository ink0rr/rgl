use super::Command;
use crate::rgl::{runner, Config, Session, UserConfig};
use crate::{info, warn};
use anyhow::Result;
use clap::Args;

/// Watch for file changes and restart automatically
#[derive(Args)]
pub struct Watch {
    #[arg(default_value = "default")]
    profile: String,
    /// Removes previous run output before running
    #[arg(long)]
    clean: bool,
    /// Enable this if filters are not working correctly
    #[arg(long)]
    compat: bool,
}

impl Command for Watch {
    fn dispatch(&self) -> Result<()> {
        loop {
            let config = Config::load()?;
            let mut session = Session::lock()?;

            runner(
                &config,
                &self.profile,
                self.clean,
                self.compat || UserConfig::force_compat(),
            )?;

            info!("Watching for changes...");
            info!("Press Ctrl+C to stop watching");
            config.watch_project_files()?;
            warn!("Changes detected, restarting...");
            session.unlock()?;
        }
    }
    fn error_context(&self) -> String {
        format!("Error running <b>{}</> profile", self.profile)
    }
}
