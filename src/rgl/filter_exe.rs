use super::{Filter, FilterContext, Subprocess};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct FilterExe {
    pub exe: String,
}

impl Filter for FilterExe {
    fn run(&self, context: &FilterContext, temp: &Path, run_args: &[String]) -> Result<()> {
        let exe = context.filter_dir.join(&self.exe);
        let mut subprocess = Subprocess::new(exe);
        subprocess
            .args(run_args)
            .current_dir(temp)
            .setup_env(&context.filter_dir);
        if context.sub_process_logging {
            subprocess.run_with_prefix(&context.name)?;
        } else {
            subprocess.run()?;
        }
        Ok(())
    }
}
