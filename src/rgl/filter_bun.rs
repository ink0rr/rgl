use super::{Filter, FilterInfo, RunContext, Subprocess, UserConfig};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FilterBun {
    pub script: String,
}

impl Filter for FilterBun {
    fn run(&self, ctx: &RunContext, info: &FilterInfo, run_args: &[String]) -> Result<()> {
        let script = info.filter_dir.join(&self.script);
        let mut subprocess = Subprocess::new("bun");
        subprocess
            .arg("run")
            .arg(script)
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

    fn install_dependencies(&self, info: &FilterInfo) -> Result<()> {
        let filter_dir = info.filter_dir(&self.script);
        Subprocess::new("bun")
            .arg("i")
            .current_dir(filter_dir)
            .run()?;
        Ok(())
    }
}
