use super::{Filter, FilterInfo, Subprocess, UserConfig};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct FilterExe {
    pub exe: String,
}

impl Filter for FilterExe {
    fn run(&self, info: &FilterInfo, temp: &Path, run_args: &[String]) -> Result<()> {
        let exe = info.filter_dir.join(&self.exe);
        let mut subprocess = Subprocess::new(exe);
        subprocess
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
}
