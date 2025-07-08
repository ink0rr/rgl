use super::Command;
use crate::rgl::{run_or_watch, UserConfig};
use anyhow::Result;
use clap::Args;

/// Runs rgl with specified profile
#[derive(Args)]
pub struct Run {
    #[arg(default_value = "default")]
    profile: String,
    /// Removes previous run output before running
    #[arg(long)]
    clean: bool,
    /// Enable this if filters are not working correctly
    #[arg(long)]
    compat: bool,
}

impl Command for Run {
    fn dispatch(&self) -> Result<()> {
        run_or_watch(
            &self.profile,
            false,
            self.clean,
            self.compat || UserConfig::force_compat(),
        )
    }
    fn error_context(&self) -> String {
        format!("Error running <b>{}</> profile", self.profile)
    }
}
