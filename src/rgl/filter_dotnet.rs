use super::{Filter, FilterContext, Subprocess, UserConfig};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct FilterDotnet {
    pub path: String,
}

impl Filter for FilterDotnet {
    fn run(&self, context: &FilterContext, temp: &Path, run_args: &[String]) -> Result<()> {
        let path = context.filter_dir.join(&self.path);
        let mut subprocess = Subprocess::new(UserConfig::dotnet_runner());
        subprocess
            .arg(path)
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
}
