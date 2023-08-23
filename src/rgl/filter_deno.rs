use super::{Filter, Result, RglError, Subprocess};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct FilterDeno {
    pub name: String,
    pub script: String,
}

impl Filter for FilterDeno {
    fn run(&mut self, temp: &PathBuf, run_args: &Vec<String>) -> Result<()> {
        let script = match Path::new(&self.script).canonicalize() {
            Ok(script) => script.display().to_string(),
            Err(_) => return Err(RglError::PathNotExistsError(self.name.to_owned())),
        };

        let output = Subprocess::new("deno")
            .args(vec!["run", "-A", &script])
            .args(run_args)
            .current_dir(temp)
            .run();

        match output {
            Ok(output) => {
                if output.status.success() {
                    Ok(())
                } else {
                    Err(RglError::FilterRunError(
                        String::from_utf8_lossy(&output.stderr).to_string(),
                    ))
                }
            }
            Err(e) => Err(RglError::FilterRunError(e.to_string())),
        }
    }
}
