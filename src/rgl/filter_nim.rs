use super::{Filter, FilterInfo, RunContext, Subprocess, UserConfig};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Serialize, Deserialize)]
pub struct FilterNim {
    pub script: String,
    /// Optional path to the folder with the nimble file. If not specified
    /// the parent of the script path is used instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirements: Option<String>,
}

impl Filter for FilterNim {
    fn run(&self, ctx: &RunContext, info: &FilterInfo, run_args: &[String]) -> Result<()> {
        let script = info.filter_dir.join(&self.script);
        let mut subprocess = Subprocess::new(UserConfig::nim_runner());
        subprocess
            .args(vec!["-r", "c", "--hints:off", "--warnings:off", "--mm:orc"])
            .arg(script)
            .args(ctx.settings_arg()?)
            .args(run_args)
            .current_dir(&ctx.temp.root)
            .setup_env(&info.filter_dir);
        if UserConfig::subprocess_logging() {
            subprocess.run_with_prefix(&info.name)?;
        } else {
            subprocess.run()?;
        }
        Ok(())
    }

    fn install_dependencies(&self, info: &FilterInfo) -> Result<()> {
        let requirements_path: PathBuf = match &self.requirements {
            Some(requirements) => info.filter_dir.join(requirements),
            None => {
                let script = info.filter_dir.join(&self.script);
                script
                    .parent()
                    .map(|parent| parent.to_path_buf())
                    .unwrap_or_else(|| info.filter_dir.clone())
            }
        };
        if has_nimble(&requirements_path) {
            Subprocess::new(UserConfig::nimble_runner())
                .args(vec!["install", "-d", "-y"])
                .current_dir(requirements_path)
                .run()?;
        }
        Ok(())
    }
}

fn has_nimble(path: &Path) -> bool {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .any(|entry| {
            entry.file_type().is_file()
                && entry.path().extension().is_some_and(|ext| ext == "nimble")
        })
}
