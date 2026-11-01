use super::{Filter, FilterInfo, Subprocess, UserConfig};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct FilterJava {
    pub path: String,
}

impl Filter for FilterJava {
    fn run(&self, info: &FilterInfo, temp: &Path, run_args: &[String]) -> Result<()> {
        let path = info.filter_dir.join(&self.path);
        let mut subprocess = Subprocess::new("java");
        subprocess
            .arg("-jar")
            .arg(path)
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
