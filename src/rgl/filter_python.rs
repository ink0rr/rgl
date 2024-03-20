use super::{Filter, FilterContext};
use crate::subprocess::Subprocess;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use which::which;

#[derive(Serialize, Deserialize)]
pub struct FilterPython {
    pub script: String,
}

impl Filter for FilterPython {
    fn run(&self, context: &FilterContext, temp: &Path, run_args: &[String]) -> Result<()> {
        let script = context.filter_dir.join(&self.script);
        let venv_dir = context.filter_dir.join(".venv");
        if venv_dir.exists() {
            let py = match cfg!(windows) {
                true => venv_dir.join("Scripts").join("python.exe"),
                false => venv_dir.join("bin").join("python"),
            };
            Subprocess::new(py)
                .arg(script)
                .args(run_args)
                .current_dir(temp)
                .run()?;
        } else {
            let py = get_python();
            Subprocess::new(py)
                .arg(script)
                .args(run_args)
                .current_dir(temp)
                .run()?;
        }
        Ok(())
    }

    fn install_dependencies(&self, context: &FilterContext) -> Result<()> {
        let filter_dir = context.filter_dir(&self.script)?;
        let requirements = filter_dir.join("requirements.txt");
        if requirements.exists() {
            let py = get_python();
            Subprocess::new(py)
                .args(vec!["-m", "venv", ".venv"])
                .current_dir(&filter_dir)
                .run_silent()?;

            let venv_dir = filter_dir.join(".venv");
            let pip = match cfg!(windows) {
                true => venv_dir.join("Scripts").join("pip.exe"),
                false => venv_dir.join("bin").join("pip"),
            };
            Subprocess::new(pip)
                .args(vec!["install", "-r", "requirements.txt"])
                .current_dir(filter_dir)
                .run_silent()?;
        }
        Ok(())
    }
}

fn get_python() -> String {
    which("python")
        .map(|_| "python".to_owned())
        .unwrap_or("python3".to_owned())
}
