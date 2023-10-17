use super::{Filter, Subprocess};
use anyhow::Result;
use dunce::canonicalize;
use std::path::PathBuf;

pub struct FilterGo {
    pub filter_dir: PathBuf,
    pub script: PathBuf,
}

impl FilterGo {
    pub fn new(filter_dir: PathBuf, script: PathBuf) -> Self {
        Self { filter_dir, script }
    }
}

impl Filter for FilterGo {
    fn run(&self, temp: &PathBuf, run_args: &Vec<String>) -> Result<()> {
        let temp = canonicalize(temp).unwrap();
        let output = match cfg!(target_os = "windows") {
            true => temp.join(".gofilter.exe"),
            false => temp.join(".gofilter"),
        };

        Subprocess::new("go")
            .args(vec!["build", "-o"])
            .arg(&output)
            .arg(&self.script)
            .current_dir(&self.filter_dir)
            .run()?;

        Subprocess::new(output)
            .args(run_args)
            .current_dir(temp)
            .setup_env(&self.filter_dir)?
            .run()?;
        Ok(())
    }

    fn install_dependencies(&self) -> Result<()> {
        Subprocess::new("go")
            .args(vec!["mod", "download"])
            .current_dir(&self.filter_dir)
            .run()?;
        Ok(())
    }
}
