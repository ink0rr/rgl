use super::{Filter, FilterInfo, RunContext, Subprocess, UserConfig};
use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FilterJava {
    pub path: String,
}

impl Filter for FilterJava {
    fn run(&self, ctx: &RunContext, info: &FilterInfo, run_args: &[String]) -> Result<()> {
        let path = info.filter_dir.join(&self.path);
        let mut subprocess = Subprocess::new(UserConfig::java_runner());
        subprocess
            .arg("-jar")
            .arg(path)
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
