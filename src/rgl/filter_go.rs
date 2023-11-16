use super::{Filter, FilterArgs, Subprocess};
use anyhow::Result;
use std::{env, path::Path};

pub struct FilterGo(pub FilterArgs);

impl Filter for FilterGo {
    fn run(&self, temp: &Path, run_args: &[String]) -> Result<()> {
        let mut output = env::current_dir()?
            .join(".regolith")
            .join("cache")
            .join("go")
            .join(&self.0.name);
        if cfg!(windows) {
            output.set_extension("exe");
        }

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
