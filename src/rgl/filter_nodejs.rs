use super::{Filter, FilterContext};
use crate::subprocess::Subprocess;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct FilterNodejs {
    pub script: String,
}

impl Filter for FilterNodejs {
    fn run(&self, context: &FilterContext, temp: &Path, run_args: &[String]) -> Result<()> {
        let script = context.filter_dir.join(&self.script);
        Subprocess::new("node")
            .arg(script)
            .args(run_args)
            .current_dir(temp)
            .setup_env(&context.filter_dir)?
            .run()?;
        Ok(())
    }

    fn install_dependencies(&self, context: &FilterContext) -> Result<()> {
        let npm = match cfg!(windows) {
            true => "npm.cmd",
            false => "npm",
        };
        let filter_dir = context.filter_dir(&self.script)?;
        Subprocess::new(npm)
            .args(vec!["i", "--no-fund", "--no-audit"])
            .current_dir(filter_dir)
            .run_silent()?;
        Ok(())
    }
}
