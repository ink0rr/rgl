use super::{Filter, FilterInfo, RunContext, Subprocess, UserConfig};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FilterShell {
    pub command: String,
}

impl Filter for FilterShell {
    fn run(&self, ctx: &RunContext, info: &FilterInfo, run_args: &[String]) -> Result<()> {
        let shell = if cfg!(windows) { "powershell" } else { "sh" };
        let mut subprocess = Subprocess::new(shell);
        subprocess
            .arg("-c")
            .arg(&self.command)
            .args(ctx.settings_arg()?)
            .args(run_args)
            .current_dir(&ctx.temp.root)
            .setup_env(&info.filter_dir);
        if UserConfig::subprocess_logging() {
            subprocess.run_with_prefix(&info.name)?;
        } else {
            subprocess.run()?;
        }
        Ok(())
    }
}
