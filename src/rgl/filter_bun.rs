use super::{Filter, FilterInfo, Subprocess, UserConfig};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct FilterBun {
    pub script: String,
}

impl Filter for FilterBun {
    fn run(&self, info: &FilterInfo, temp: &Path, run_args: &[String]) -> Result<()> {
        let script = info.filter_dir.join(&self.script);
        let mut subprocess = Subprocess::new("bun");
        subprocess
            .arg("run")
            .arg(script)
            .args(run_args)
            .current_dir(temp)
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
