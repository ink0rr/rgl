use super::{Filter, FilterContext};
use crate::subprocess::Subprocess;
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
        Subprocess::new("deno")
            .args(vec!["run", "-A"])
            .arg(script)
            .args(run_args)
            .current_dir(temp)
            .setup_env(&context.filter_dir)?
            .run()?;
        Ok(())
    }
}
