use super::{Filter, FilterArgs, Subprocess};
use anyhow::Result;
use std::path::Path;

pub struct FilterNode(pub FilterArgs);

impl Filter for FilterNode {
    fn run(&self, temp: &Path, run_args: &[String]) -> Result<()> {
        Subprocess::new("node")
            .arg(&self.0.script)
            .args(run_args)
            .current_dir(temp)
            .setup_env(&self.0.filter_dir)?
            .run()?;
        Ok(())
    }

    fn install_dependencies(&self) -> Result<()> {
        let npm = match cfg!(windows) {
            true => "npm.cmd",
            false => "npm",
        };
        Subprocess::new(npm)
            .args(vec!["i", "--no-fund", "--no-audit"])
            .current_dir(&self.0.filter_dir)
            .run_silent()?;
        Ok(())
    }
}
