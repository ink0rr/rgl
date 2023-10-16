use super::{read_json, Filter, FilterDefinition};
use anyhow::{bail, Context, Result};
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
    pub fn new(name: &str) -> Result<Self> {
        let filter_dir = PathBuf::from(".regolith")
            .join("cache")
            .join("filters")
            .join(name);
        if !filter_dir.is_dir() {
            bail!("Filter <b>{name}</> not installed, run \"rgl install\" to install it")
        }

        let mut filter_config =
            read_json::<FilterRemote>(filter_dir.join("filter.json")).context("FilterConfig")?;
        for entry in filter_config.filters.iter_mut() {
            match entry {
                FilterDefinition::Local(def) => {
                    def.script = filter_dir.join(&def.script).display().to_string();
                }
                FilterDefinition::Remote(_) => bail!(
                    "Found nested remote filter definition in filter <b>{name}</>\n\
                     <yellow> >></> This feature is not supported"
                ),
            }
        }
        filter_config.name = name.to_owned();
        filter_config.filter_dir = filter_dir;
        Ok(filter_config)
    }
}

impl Filter for FilterRemote {
    fn run(&self, temp: &PathBuf, run_args: &Vec<String>) -> Result<()> {
        for entry in self.filters.iter() {
            match entry {
                FilterDefinition::Local(_) => {
                    entry
                        .to_filter(&self.name, Some(self.filter_dir.to_owned()))?
                        .run(temp, run_args)?;
                }
                _ => unreachable!(),
            }
        }
        Ok(())
    }
}
