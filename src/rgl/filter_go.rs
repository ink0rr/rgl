use super::{Filter, FilterInfo, Subprocess, UserConfig};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{env, path::Path};
use walkdir::{DirEntry, WalkDir};

#[derive(Serialize, Deserialize)]
pub struct FilterGo {
    pub script: String,
}

impl Filter for FilterGo {
    fn run(&self, info: &FilterInfo, temp: &Path, run_args: &[String]) -> Result<()> {
        let script = info.filter_dir.join(&self.script);
        let mut output = env::current_dir()?
            .join(".regolith")
            .join("cache")
            .join("go")
            .join(&info.name);
        if cfg!(windows) {
            output.set_extension("exe");
        }

        if should_rebuild(&info.filter_dir, &output)? {
            Subprocess::new("go")
                .args(vec!["build", "-o"])
                .arg(&output)
                .arg(script)
                .current_dir(&info.filter_dir)
                .run()?;
        }

        let mut subprocess = Subprocess::new(output);
        subprocess
            .args(run_args)
            .current_dir(temp)
            .setup_env(&info.filter_dir);
        if UserConfig::subprocess_logging() {
            subprocess.run_with_prefix(&info.name)?;
        } else {
            subprocess.run()?;
        }
        Ok(())
    }

    fn install_dependencies(&self, info: &FilterInfo) -> Result<()> {
        Subprocess::new("go")
            .args(vec!["mod", "download"])
            .current_dir(&info.filter_dir)
            .run()?;
        Ok(())
    }
}

fn should_rebuild(path: &Path, output: &Path) -> Result<bool> {
    let output_time = if let Ok(metadata) = output.metadata() {
        metadata.modified()?
    } else {
        return Ok(true);
    };
    let walker = WalkDir::new(path)
        .into_iter()
        .filter_entry(|entry| !is_hidden(entry));
    for entry in walker {
        if entry?.metadata()?.modified()? > output_time {
            return Ok(true);
        }
    }
    Ok(false)
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}
