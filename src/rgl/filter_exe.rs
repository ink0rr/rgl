use super::{Filter, FilterContext};
use crate::subprocess::Subprocess;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct FilterExe {
    pub exe: String,
}

impl Filter for FilterExe {
    fn run(&self, context: &FilterContext, temp: &Path, run_args: &[String]) -> Result<()> {
        let exe = context.dir.join(&self.exe);
        Subprocess::new(exe)
            .args(run_args)
            .current_dir(temp)
            .setup_env(&context.dir)?
            .run()?;
        Ok(())
    }
}
