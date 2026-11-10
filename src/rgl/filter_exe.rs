use super::{Filter, FilterInfo, RunContext, Subprocess, UserConfig};
use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FilterExe {
    pub exe: String,
}

impl Filter for FilterExe {
    fn run(&self, ctx: &RunContext, info: &FilterInfo, run_args: &[String]) -> Result<()> {
        let exe = info.filter_dir.join(&self.exe);
        let mut subprocess = Subprocess::new(exe);
        subprocess
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
