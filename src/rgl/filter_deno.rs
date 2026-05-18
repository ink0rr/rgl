use super::{Filter, FilterContext, Subprocess, UserConfig};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct FilterDeno {
    pub script: String,
}

impl Filter for FilterDeno {
    fn run(&self, context: &FilterContext, temp: &Path, run_args: &[String]) -> Result<()> {
        let script = context.filter_dir.join(&self.script);
        let mut subprocess = Subprocess::new("deno");
        subprocess
            .args(vec!["run", "-A", "--no-lock"])
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
}
