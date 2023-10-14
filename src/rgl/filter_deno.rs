use super::{Filter, RglError, RglResult, Subprocess};
use dunce::canonicalize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct FilterDeno {
    pub name: String,
    pub script: String,
}

impl FilterDeno {
    pub fn new(name: &str, script: &str) -> Self {
        Self {
            name: name.to_owned(),
            script: script.to_owned(),
        }
    }
}

impl Filter for FilterDeno {
    fn run(&mut self, temp: &PathBuf, run_args: &Vec<String>) -> RglResult<()> {
        let script = canonicalize(&self.script)
            .map(|script| script.display().to_string())
            .map_err(|_| RglError::InvalidFilterDefinition {
                filter_name: self.name.to_owned(),
                cause: RglError::PathNotExists {
                    path: self.script.to_owned(),
                }
                .into(),
            })?;

        let output = Subprocess::new("deno")
            .args(vec!["run", "-A", &script])
            .args(run_args)
            .current_dir(temp)
            .setup_env()?
            .run()?;

        if output.status.success() {
            Ok(())
        } else {
            Err(RglError::FilterRunFailed {
                filter_name: self.name.to_owned(),
            })
        }
    }
}
