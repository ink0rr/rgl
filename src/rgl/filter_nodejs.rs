use super::{Filter, FilterContext, Subprocess, UserConfig};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct FilterNodejs {
    pub script: String,
}

impl Filter for FilterNodejs {
    fn run(&self, context: &FilterContext, temp: &Path, run_args: &[String]) -> Result<()> {
        let runtime = UserConfig::nodejs_runtime();
        let script = context.filter_dir.join(&self.script);
        Subprocess::new(runtime)
            .arg(script)
            .args(run_args)
            .current_dir(temp)
            .setup_env(&context.filter_dir)?
            .run()?;
        Ok(())
    }

    fn install_dependencies(&self, context: &FilterContext) -> Result<()> {
        let filter_dir = context.filter_dir(&self.script);
        if filter_dir.join("package.json").exists() {
            let package_manager = UserConfig::nodejs_package_manager();
            Subprocess::new(package_manager)
                .arg("i")
                .current_dir(filter_dir)
                .run()?;
        }
        Ok(())
    }
}
