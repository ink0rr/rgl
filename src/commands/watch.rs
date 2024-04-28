use super::Command;
use crate::rgl::run_or_watch;
use anyhow::Result;
use clap::Args;

/// Watch for file changes and restart automatically
#[derive(Args)]
pub struct Watch {
    #[arg(default_value = "default")]
    profile: String,
    /// Do not use previous run output as cache
    #[arg(long)]
    no_cache: bool,
}

impl Command for Watch {
    fn dispatch(&self) -> Result<()> {
        run_or_watch(&self.profile, true, !self.no_cache)
    }
    fn error_context(&self) -> String {
        format!("Error running <b>{}</> profile", self.profile)
    }
}
