use super::{Filter, FilterContext, Subprocess};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct FilterShell {
    pub command: String,
}

impl Filter for FilterShell {
    fn run(&self, context: &FilterContext, temp: &Path, run_args: &[String]) -> Result<()> {
        let shell = if cfg!(windows) { "powershell" } else { "sh" };
        Subprocess::new(shell)
            .arg("-c")
            .arg(&self.command)
            .args(run_args)
            .current_dir(temp)
            .setup_env(&context.filter_dir)
            .run()?;
        Ok(())
    }
}
