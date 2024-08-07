use super::{Filter, FilterContext, Subprocess};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{env, path::Path};
use walkdir::{DirEntry, WalkDir};

#[derive(Serialize, Deserialize)]
pub struct FilterGo {
    pub script: String,
}

impl Filter for FilterGo {
    fn run(&self, context: &FilterContext, temp: &Path, run_args: &[String]) -> Result<()> {
        let script = context.filter_dir.join(&self.script);
        let mut output = env::current_dir()?
            .join(".regolith")
            .join("cache")
            .join("go")
            .join(&context.name);
        if cfg!(windows) {
            output.set_extension("exe");
        }

        if should_rebuild(&context.filter_dir, &output)? {
            Subprocess::new("go")
                .args(vec!["build", "-o"])
                .arg(&output)
                .arg(script)
                .current_dir(&context.filter_dir)
                .run()?;
        }

        Subprocess::new(output)
            .args(run_args)
            .current_dir(temp)
            .setup_env(&context.filter_dir)?
            .run()?;
        Ok(())
    }

    fn install_dependencies(&self, context: &FilterContext) -> Result<()> {
        Subprocess::new("go")
            .args(vec!["mod", "download"])
            .current_dir(&context.filter_dir)
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
