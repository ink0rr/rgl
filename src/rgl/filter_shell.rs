use super::{Filter, FilterContext, Subprocess};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct FilterShell {
    pub command: String,
    pub arguments: Option<Vec<String>>,
}

impl Filter for FilterShell {
    fn run(&self, context: &FilterContext, temp: &Path, run_args: &[String]) -> Result<()> {
        let args = self.arguments.as_deref().unwrap_or(&[]);
        let cmd = if cfg!(windows) { "cmd" } else { "sh" };
        Subprocess::new(cmd)
            .args(&["-c", &self.command])
            .args(args)
            .args(run_args)
            .current_dir(temp)
            .setup_env(&context.filter_dir)?
            .run()?;
        Ok(())
    }
}
