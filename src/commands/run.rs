use super::Command;
use crate::rgl::{runner, Config, Session, UserConfig};
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
    /// Extra arguments to forward to every filter in the profile
    filter_args: Vec<String>,
}

impl Command for Run {
    fn dispatch(&self) -> Result<()> {
        let config = Config::load()?;
        let mut session = Session::lock()?;

        smol::block_on(runner(
            &config,
            &self.profile,
            self.clean,
            self.compat || UserConfig::force_compat(),
            &self.filter_args,
        ))?;

        session.unlock()
    }
    fn error_context(&self) -> String {
        format!("Error running <profile>{}</> profile", self.profile)
    }
}
