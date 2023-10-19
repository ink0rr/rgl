use super::{Filter, Subprocess};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct FilterNode {
    pub filter_dir: PathBuf,
    pub script: PathBuf,
}

impl FilterNode {
    pub fn new(filter_dir: PathBuf, script: PathBuf) -> Self {
        Self { filter_dir, script }
    }
}

impl Filter for FilterNode {
    fn run(&self, temp: &Path, run_args: &[String]) -> Result<()> {
        Subprocess::new("node")
            .arg(&self.script)
            .args(run_args)
            .current_dir(temp)
            .setup_env(&self.filter_dir)?
            .run()?;
        Ok(())
    }

    fn install_dependencies(&self) -> Result<()> {
        let npm = match cfg!(target_os = "windows") {
            true => "npm.cmd",
            false => "npm",
        };
        Subprocess::new(npm)
            .args(vec!["i", "--no-fund", "--no-audit"])
            .current_dir(&self.filter_dir)
            .run_silent()?;
        Ok(())
    }
}
