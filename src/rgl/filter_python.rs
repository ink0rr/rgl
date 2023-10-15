use super::{Filter, RglResult, Subprocess};
use std::path::PathBuf;
use which::which;

pub struct FilterPython {
    pub filter_dir: PathBuf,
    pub script: PathBuf,
}

impl FilterPython {
    pub fn new(filter_dir: PathBuf, script: PathBuf) -> Self {
        Self { filter_dir, script }
    }
}

impl Filter for FilterPython {
    fn run(&self, temp: &PathBuf, run_args: &Vec<String>) -> RglResult<()> {
        let venv_dir = self.filter_dir.join(".venv");
        if venv_dir.exists() {
            let py = match cfg!(target_os = "windows") {
                true => venv_dir.join("Scripts").join("python.exe"),
                false => venv_dir.join("bin").join("python"),
            };
            Subprocess::new(py)
                .arg(&self.script)
                .args(run_args)
                .current_dir(temp)
                .run()?;
        } else {
            let py = get_python();
            Subprocess::new(py)
                .arg(&self.script)
                .args(run_args)
                .current_dir(temp)
                .run()?;
        }
        Ok(())
    }

    fn install_dependencies(&self) -> RglResult<()> {
        let requirements = self.filter_dir.join("requirements.txt");
        if requirements.exists() {
            let py = get_python();
            Subprocess::new(py)
                .args(vec!["-m", "venv", ".venv"])
                .current_dir(&self.filter_dir)
                .run_silent()?;

            let venv_dir = self.filter_dir.join(".venv");
            let pip = match cfg!(target_os = "windows") {
                true => venv_dir.join("Scripts").join("pip.exe"),
                false => venv_dir.join("bin").join("pip"),
            };
            Subprocess::new(pip)
                .args(vec!["install", "-r", "requirements.txt"])
                .current_dir(&self.filter_dir)
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
