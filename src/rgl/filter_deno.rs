use super::{Filter, RglError, RglResult, Subprocess};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct FilterDeno {
    pub name: String,
    pub script: String,
}

impl Filter for FilterDeno {
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

        let output = Subprocess::new("deno")
            .args(vec!["run", "-A", &script])
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
