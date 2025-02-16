use super::Command;
use crate::rgl::run;
use anyhow::Result;
use clap::Args;

/// Runs rgl with specified profile
#[derive(Args)]
pub struct Run {
    #[arg(default_value = "default")]
    profile: String,
    /// Use previous run output as cache
    #[arg(long)]
    cached: bool,
}

impl Command for Run {
    fn dispatch(&self) -> Result<()> {
        run(&self.profile, self.cached)
    }
    fn error_context(&self) -> String {
        format!("Error running <b>{}</> profile", self.profile)
    }
}
