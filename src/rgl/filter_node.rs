use super::{Filter, RglError, RglResult, Subprocess};
use serde::{Deserialize, Serialize};
use simplelog::info;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct FilterNode {
    pub name: String,
    pub script: String,
}

impl FilterNode {
    pub fn new(name: &str, script: &str) -> Self {
        Self {
            name: name.to_owned(),
            script: script.to_owned(),
        }
    }
}

impl Filter for FilterNode {
    fn install_dependencies(&self, filter_dir: PathBuf) -> RglResult<()> {
        info!("Installing npm dependencies for <b>{}</>...", self.name);
        let npm = match cfg!(target_os = "windows") {
            true => "npm.cmd",
            false => "npm",
        };
        Subprocess::new(npm)
            .args(vec!["i", "--no-fund", "--no-audit"])
            .current_dir(filter_dir)
            .run_silent()?;
        Ok(())
    }
    fn run(&mut self, temp: &PathBuf, run_args: &Vec<String>) -> RglResult<()> {
        let script = match Path::new(&self.script).canonicalize() {
            Ok(script) => script.display().to_string(),
            Err(_) => {
                return Err(RglError::InvalidFilterDefinition {
                    filter_name: self.name.to_owned(),
                    cause: RglError::PathNotExists {
                        path: self.script.to_owned(),
                    }
                    .into(),
                })
            }
        };

        let output = Subprocess::new("node")
            .arg(&script)
            .args(run_args)
            .current_dir(temp)
            .run()?;

        match output.status.success() {
            true => Ok(()),
            false => Err(RglError::FilterRunFailed {
                filter_name: self.name.to_owned(),
            }),
        }
    }
}
