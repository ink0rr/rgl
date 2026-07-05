use super::{Filter, FilterContext, Subprocess, UserConfig};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct FilterBun {
    pub script: String,
}

impl Filter for FilterBun {
    fn run(&self, context: &FilterContext, temp: &Path, run_args: &[String]) -> Result<()> {
        let script = context.filter_dir.join(&self.script);
        let mut subprocess = Subprocess::new(UserConfig::bun_runner());
        subprocess
            .arg("run")
            .arg(script)
            .args(run_args)
            .current_dir(temp)
            .setup_env(&context.filter_dir);
        if UserConfig::subprocess_logging() {
            subprocess.run_with_prefix(&context.name)?;
        } else {
            subprocess.run()?;
        }
        Ok(())
    }

    fn install_dependencies(&self, context: &FilterContext) -> Result<()> {
        let filter_dir = context.filter_dir(&self.script);
        Subprocess::new(UserConfig::bun_runner())
            .arg("i")
            .current_dir(filter_dir)
            .run()?;
        Ok(())
    }
}
