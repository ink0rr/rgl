use super::Command;
use crate::rgl::run_or_watch;
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
}

impl Command for Watch {
    fn dispatch(&self) -> Result<()> {
        loop {
            run_or_watch(&self.profile, true, self.clean)?;
        }
    }
    fn error_context(&self) -> String {
        format!("Error running <b>{}</> profile", self.profile)
    }
}
