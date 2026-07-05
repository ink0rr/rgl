use super::{get_dot_regolith_dir, Filter, FilterContext, Subprocess, UserConfig};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize)]
pub struct FilterPython {
    pub script: String,
    /// Optional path to the file with the requirements (usually
    /// `requirements.txt`), relative to the filter's own directory. If not
    /// specified, `requirements.txt` next to the script is used instead.
    /// Mirrors Go's `PythonFilterDefinition.Requirements`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirements: Option<String>,
    /// Optional venv "slot": filters sharing the same slot share a single
    /// venv, stored at `<dotRegolith>/cache/venvs/<slot>` instead of the
    /// filter's own directory. Mirrors Go's
    /// `PythonFilterDefinition.VenvSlot`. Absent, rgl keeps its own
    /// pre-existing behavior of a per-filter `.venv` directory.
    #[serde(rename = "venvSlot", skip_serializing_if = "Option::is_none")]
    pub venv_slot: Option<u32>,
}

/// The shared venv directory for a given `venvSlot`. Mirrors Go's
/// `resolveVenvPath`.
fn shared_venv_dir(slot: u32) -> Result<PathBuf> {
    Ok(get_dot_regolith_dir()?
        .join("cache")
        .join("venvs")
        .join(slot.to_string()))
}

impl FilterPython {
    /// The requirements *file* path (not just its folder), matching Go's
    /// `Requirements`/default-to-`requirements.txt` resolution.
    fn requirements_file(&self, context: &FilterContext) -> PathBuf {
        match &self.requirements {
            Some(requirements) => context.filter_dir.join(requirements),
            None => context.filter_dir(&self.script).join("requirements.txt"),
        }
    }
}

impl Filter for FilterPython {
    fn run(&self, context: &FilterContext, temp: &Path, run_args: &[String]) -> Result<()> {
        let script = context.filter_dir.join(&self.script);
        // Absent `venvSlot`, this matches rgl's pre-existing (script-path-
        // agnostic) lookup of `<filter_dir>/.venv`, kept unchanged.
        let venv_dir = match self.venv_slot {
            Some(slot) => shared_venv_dir(slot)?,
            None => context.filter_dir.join(".venv"),
        };
        let mut subprocess = Subprocess::new(match venv_dir.exists() {
            true => match cfg!(windows) {
                true => venv_dir.join("Scripts").join("python.exe"),
                false => venv_dir.join("bin").join("python"),
            },
            false => UserConfig::python_runner().into(),
        });
        subprocess
            .arg("-u")
            .arg(script)
            .args(run_args)
            .current_dir(temp)
            .setup_env(&context.filter_dir);
        if UserConfig::subprocess_logging() {
            subprocess.run_with_prefix(&context.name)?;
        } else {
            subprocess.run()?;
        }
        Ok(())
    }

    fn install_dependencies(&self, context: &FilterContext) -> Result<()> {
        let requirements = self.requirements_file(context);
        if requirements.is_file() {
            // Absent `venvSlot`, this matches rgl's pre-existing behavior of
            // creating `.venv` inside the script's own directory (which may
            // differ from `context.filter_dir` for local filters with a
            // script in a subdirectory), kept unchanged.
            let filter_dir = context.filter_dir(&self.script);
            let venv_dir = match self.venv_slot {
                Some(slot) => shared_venv_dir(slot)?,
                None => filter_dir.join(".venv"),
            };
            if let Some(parent) = venv_dir.parent() {
                fs::create_dir_all(parent)?;
            }
            let py = UserConfig::python_runner();
            Subprocess::new(py)
                .args(["-m", "venv"])
                .arg(&venv_dir)
                .run()?;

            let pip = match cfg!(windows) {
                true => venv_dir.join("Scripts").join("pip.exe"),
                false => venv_dir.join("bin").join("pip"),
            };
            let requirements_dir = requirements
                .parent()
                .map(|parent| parent.to_path_buf())
                .unwrap_or_else(|| context.filter_dir.clone());
            let requirements_file_name = requirements
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| "requirements.txt".to_owned());
            Subprocess::new(pip)
                .args(["install", "-r", &requirements_file_name])
                .current_dir(requirements_dir)
                .run()?;
        }
        Ok(())
    }
}
