use super::{Filter, Subprocess};
use anyhow::Result;
use std::path::PathBuf;

pub struct FilterDeno {
    pub filter_dir: PathBuf,
    pub script: PathBuf,
}

impl FilterDeno {
    pub fn new(filter_dir: PathBuf, script: PathBuf) -> Self {
        Self { filter_dir, script }
    }
}

impl Filter for FilterDeno {
    fn run(&self, temp: &PathBuf, run_args: &Vec<String>) -> Result<()> {
        Subprocess::new("deno")
            .args(vec!["run", "-A"])
            .arg(&self.script)
            .args(run_args)
            .current_dir(temp)
            .setup_env(&self.filter_dir)?
            .run()?;
        Ok(())
    }
}
