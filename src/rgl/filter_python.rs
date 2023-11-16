use super::{Filter, FilterArgs, Subprocess};
use anyhow::Result;
use std::path::Path;
use which::which;

pub struct FilterPython(pub FilterArgs);

impl Filter for FilterPython {
    fn run(&self, temp: &Path, run_args: &[String]) -> Result<()> {
        let venv_dir = self.0.filter_dir.join(".venv");
        if venv_dir.exists() {
            let py = match cfg!(windows) {
                true => venv_dir.join("Scripts").join("python.exe"),
                false => venv_dir.join("bin").join("python"),
            };
            Subprocess::new(py)
                .arg(&self.0.script)
                .args(run_args)
                .current_dir(temp)
                .run()?;
        } else {
            let py = get_python();
            Subprocess::new(py)
                .arg(&self.0.script)
                .args(run_args)
                .current_dir(temp)
                .run()?;
        }
        Ok(())
    }

    fn install_dependencies(&self) -> Result<()> {
        let requirements = self.0.filter_dir.join("requirements.txt");
        if requirements.exists() {
            let py = get_python();
            Subprocess::new(py)
                .args(vec!["-m", "venv", ".venv"])
                .current_dir(&self.0.filter_dir)
                .run_silent()?;

            let venv_dir = self.0.filter_dir.join(".venv");
            let pip = match cfg!(windows) {
                true => venv_dir.join("Scripts").join("pip.exe"),
                false => venv_dir.join("bin").join("pip"),
            };
            Subprocess::new(pip)
                .args(vec!["install", "-r", "requirements.txt"])
                .current_dir(&self.0.filter_dir)
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
