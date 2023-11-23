use super::{Filter, FilterContext, Subprocess};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct FilterNode {
    pub script: String,
}

impl Filter for FilterNode {
    fn run(&self, context: &FilterContext, temp: &Path, run_args: &[String]) -> Result<()> {
        Subprocess::new("node")
            .arg(&self.script)
            .args(run_args)
            .current_dir(temp)
            .setup_env(&context.dir)?
            .run()?;
        Ok(())
    }

    fn install_dependencies(&self, context: &FilterContext) -> Result<()> {
        let npm = match cfg!(windows) {
            true => "npm.cmd",
            false => "npm",
        };
        Subprocess::new(npm)
            .args(vec!["i", "--no-fund", "--no-audit"])
            .current_dir(&context.dir)
            .run_silent()?;
        Ok(())
    }
}
