use super::{Filter, FilterContext, Subprocess};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct FilterExe {
    pub exe: String,
    pub arguments: Option<Vec<String>>,
}

impl Filter for FilterExe {
    fn run(&self, context: &FilterContext, temp: &Path, run_args: &[String]) -> Result<()> {
        let exe = context.filter_dir.join(&self.exe);
        let args = self.arguments.as_deref().unwrap_or(&[]);
        Subprocess::new(exe)
            .args(run_args)
            .args(args)
            .current_dir(temp)
            .setup_env(&context.filter_dir)?
            .run()?;
        Ok(())
    }
}
