use super::Command;
use crate::rgl::watch;
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
        watch(&self.profile, !self.no_cache)
    }
    fn error_context(&self) -> String {
        format!("Error running <b>{}</> profile", self.profile)
    }
}
