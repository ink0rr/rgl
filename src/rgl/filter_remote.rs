use super::{read_json, Filter, FilterDefinition, RglError, RglResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct FilterRemoteConfig {
    #[serde(skip_serializing, skip_deserializing)]
    name: String,
    #[serde(skip_serializing, skip_deserializing)]
    filter_dir: PathBuf,
    pub filters: Vec<FilterDefinition>,
    pub version: String,
}

impl FilterRemoteConfig {
    pub fn new(name: &str) -> RglResult<Self> {
        let filter_dir = Path::new(".regolith")
            .join("cache")
            .join("filters")
            .join(name);

        if !filter_dir.is_dir() {
            return Err(RglError::FilterNotInstalled {
                filter_name: name.to_owned(),
            });
        }

        match read_json::<FilterRemoteConfig>(filter_dir.join("filter.json")) {
            Err(e) => Err(RglError::FilterConfig {
                filter_name: name.to_owned(),
                cause: e.into(),
            }),
            Ok(mut filter_config) => {
                filter_config.name = name.to_owned();
                filter_config.filter_dir = filter_dir;
                Ok(filter_config)
            }
        }
    }
}

impl Filter for FilterRemoteConfig {
    fn run(&mut self, temp: &std::path::PathBuf, run_args: &Vec<String>) -> RglResult<()> {
        for entry in self.filters.iter_mut() {
            match entry {
                FilterDefinition::FilterLocal(def) => {
                    def.script = self.filter_dir.join(&def.script).display().to_string();
                    entry.to_filter(&self.name)?.run(temp, run_args)?
                }
                FilterDefinition::FilterRemote(_) => {
                    return Err(RglError::NestedRemoteFilter {
                        filter_name: self.name.to_owned(),
                    })
                }
            }
        }
        Ok(())
    }
}
