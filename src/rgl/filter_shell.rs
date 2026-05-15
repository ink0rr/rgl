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
        let mut subprocess = Subprocess::new(shell);
        subprocess
            .arg("-c")
            .arg(&self.command)
            .args(run_args)
            .current_dir(temp)
            .setup_env(&context.filter_dir);
        if context.sub_process_logging {
            subprocess.run_with_prefix(&context.name)?;
        } else {
            subprocess.run()?;
        }
        Ok(())
    }
}
