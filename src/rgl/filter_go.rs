use super::{Filter, FilterArgs, Subprocess};
use anyhow::Result;
use dunce::canonicalize;
use std::path::Path;

pub struct FilterGo(pub FilterArgs);

impl Filter for FilterGo {
    fn run(&self, temp: &Path, run_args: &[String]) -> Result<()> {
        let temp = canonicalize(temp)?;
        let output = match cfg!(windows) {
            true => temp.join(".gofilter.exe"),
            false => temp.join(".gofilter"),
        };

        Subprocess::new("go")
            .args(vec!["build", "-o"])
            .arg(&output)
            .arg(&self.0.script)
            .current_dir(&self.0.filter_dir)
            .run()?;

        Subprocess::new(output)
            .args(run_args)
            .current_dir(temp)
            .setup_env(&self.0.filter_dir)?
            .run()?;
        Ok(())
    }

    fn install_dependencies(&self) -> Result<()> {
        Subprocess::new("go")
            .args(vec!["mod", "download"])
            .current_dir(&self.0.filter_dir)
            .run()?;
        Ok(())
    }
}
