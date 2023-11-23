use super::{read_json, ref_to_version, write_json, Filter, FilterContext, FilterDefinition};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct RemoteFilter {
    pub url: String,
    pub version: String,
}

impl RemoteFilter {
    pub fn cache_dir(name: &str) -> PathBuf {
        PathBuf::from(".regolith")
            .join("cache")
            .join("filters")
            .join(name)
    }
}

impl Filter for RemoteFilter {
    fn run(&self, context: &FilterContext, temp: &Path, run_args: &[String]) -> Result<()> {
        let config = RemoteFilterConfig::load(&context.name)?;
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
    pub filters: Vec<FilterDefinition>,
    pub version: String,
}

impl RemoteFilterConfig {
    pub fn new(name: &str, git_ref: &str) -> Result<Self> {
        let config_path = RemoteFilter::cache_dir(name).join("filter.json");
        let mut config = read_json::<Value>(&config_path)?;
        config["version"] = json!(ref_to_version(git_ref));
        write_json(config_path, &config)?;

        let config = serde_json::from_value(config)?;
        Ok(config)
    }

    pub fn load(name: &str) -> Result<Self> {
        let filter_dir = RemoteFilter::cache_dir(name);
        let config = read_json::<RemoteFilterConfig>(filter_dir.join("filter.json"))
            .context(format!("Failed to load config for filter <b>{name}</>"))?;
        Ok(config)
    }
}
