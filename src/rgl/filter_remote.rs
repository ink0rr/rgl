use super::{ref_to_version, Filter, FilterContext, LocalFilter};
use crate::fs::{read_json, write_json};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct RemoteFilter {
    pub url: String,
    pub version: String,
}

impl Filter for RemoteFilter {
    fn run(&self, context: &FilterContext, temp: &Path, run_args: &[String]) -> Result<()> {
        let config_path = context.filter_dir.join("filter.json");
        let config = read_json::<RemoteFilterConfig>(config_path).context(format!(
            "Failed to load config for filter <b>{}</>",
            context.name
        ))?;
        let is_latest = matches!(self.version.as_str(), "HEAD" | "latest");
        if !is_latest && self.version != config.version {
            bail!(
                "Filter version mismatch\n\
                 <yellow> >></> Installed version: {}\n\
                 <yellow> >></> Required version: {}",
                config.version,
                self.version
            );
        }
        for filter in config.filters {
            filter.run(context, temp, run_args)?;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct RemoteFilterConfig {
    pub filters: Vec<LocalFilter>,
    pub version: String,
}

impl RemoteFilterConfig {
    pub fn new(filter_dir: PathBuf, git_ref: &str) -> Result<Self> {
        let config_path = filter_dir.join("filter.json");
        let mut config = read_json::<Value>(&config_path)?;
        config["version"] = json!(ref_to_version(git_ref));
        write_json(config_path, &config)?;

        let config = serde_json::from_value(config)?;
        Ok(config)
    }
}
