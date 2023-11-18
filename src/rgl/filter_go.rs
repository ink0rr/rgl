use super::{Filter, FilterArgs, Subprocess};
use anyhow::Result;
use std::{env, path::Path};
use walkdir::{DirEntry, WalkDir};

pub struct FilterGo(pub FilterArgs);

impl Filter for FilterGo {
    fn run(&self, temp: &Path, run_args: &[String]) -> Result<()> {
        let mut output = env::current_dir()?
            .join(".regolith")
            .join("cache")
            .join("go")
            .join(&self.0.name);
        if cfg!(windows) {
            output.set_extension("exe");
        }

        if should_rebuild(&self.0.filter_dir, &output)? {
            Subprocess::new("go")
                .args(vec!["build", "-o"])
                .arg(&output)
                .arg(&self.0.script)
                .current_dir(&self.0.filter_dir)
                .run()?;
        }

        Subprocess::new(output)
            .args(run_args)
            .current_dir(temp)
            .setup_env(&self.0.filter_dir)?
            .run()?;
        Ok(())
    }

    fn install_dependencies(&self) -> Result<()> {
        Subprocess::new("go")
            .args(vec!["mod", "download"])
            .current_dir(&self.0.filter_dir)
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
