use super::{Filter, FilterArgs, Subprocess};
use anyhow::Result;
use std::path::Path;

pub struct FilterDeno(pub FilterArgs);

impl Filter for FilterDeno {
    fn run(&self, temp: &Path, run_args: &[String]) -> Result<()> {
        Subprocess::new("deno")
            .args(vec!["run", "-A"])
            .arg(&self.0.script)
            .args(run_args)
            .current_dir(temp)
            .setup_env(&self.0.filter_dir)?
            .run()?;
        Ok(())
    }
}
