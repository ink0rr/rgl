use super::{Filter, FilterContext, Subprocess};
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
        Subprocess::new("bun")
            .arg("run")
            .arg(script)
            .args(run_args)
            .current_dir(temp)
            .setup_env(&context.filter_dir)?
            .run()?;
        Ok(())
    }

    fn install_dependencies(&self, context: &FilterContext) -> Result<()> {
        let filter_dir = context.filter_dir(&self.script);
        Subprocess::new("bun")
            .arg("i")
            .current_dir(filter_dir)
            .run_silent()?;
        Ok(())
    }
}
