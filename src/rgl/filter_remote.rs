use super::{read_json, Filter, FilterDefinition, RglError, RglResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct FilterRemote {
    #[serde(skip_serializing, skip_deserializing)]
    name: String,
    #[serde(skip_serializing, skip_deserializing)]
    filter_dir: PathBuf,
    pub filters: Vec<FilterDefinition>,
    pub version: String,
}

impl FilterRemote {
    pub fn new(name: &str) -> RglResult<Self> {
        let filter_dir = PathBuf::from(".regolith")
            .join("cache")
            .join("filters")
            .join(name);
        if !filter_dir.is_dir() {
            return Err(RglError::FilterNotInstalled {
                filter_name: name.to_owned(),
            });
        }

        let mut filter_config =
            read_json::<FilterRemote>(filter_dir.join("filter.json")).map_err(|e| {
                RglError::FilterConfig {
                    filter_name: name.to_owned(),
                    cause: e.into(),
                }
            })?;
        for entry in filter_config.filters.iter_mut() {
            match entry {
                FilterDefinition::Local(def) => {
                    def.script = filter_dir.join(&def.script).display().to_string();
                }
                FilterDefinition::Remote(_) => {
                    return Err(RglError::NestedRemoteFilter {
                        filter_name: name.to_owned(),
                    })
                }
            }
        }
        filter_config.name = name.to_owned();
        filter_config.filter_dir = filter_dir;
        Ok(filter_config)
    }
}

impl Filter for FilterRemote {
    fn run(&self, temp: &PathBuf, run_args: &Vec<String>) -> RglResult<()> {
        for entry in self.filters.iter() {
            match entry {
                FilterDefinition::Local(_) => {
                    entry
                        .to_filter(&self.name, Some(self.filter_dir.to_owned()))?
                        .run(temp, run_args)?;
                }
                FilterDefinition::Remote(_) => {
                    return Err(RglError::NestedRemoteFilter {
                        filter_name: self.name.to_owned(),
                    })
                }
            }
        }
        Ok(())
    }
}
