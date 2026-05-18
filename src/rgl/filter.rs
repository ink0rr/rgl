use super::{
    get_current_dir, get_filter_cache_dir, FilterBun, FilterDeno, FilterExe, FilterGo,
    FilterNodejs, FilterPython, FilterShell, RemoteFilter, RemoteFilterConfig,
};
use crate::fs::{is_dir_empty, read_json};
use crate::info;
use anyhow::{Context, Result};
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use strum::Display;

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
#[enum_dispatch]
pub enum FilterDefinition {
    Local(LocalFilter),
    Remote(RemoteFilter),
}

impl FilterDefinition {
    pub fn from_value(value: Value) -> Result<Self> {
        let filter = match value["runWith"] {
            Value::String(_) => {
                let filter = serde_json::from_value::<LocalFilter>(value)?;
                FilterDefinition::Local(filter)
            }
            _ => {
                let filter = serde_json::from_value::<RemoteFilter>(value)?;
                FilterDefinition::Remote(filter)
            }
        };
        Ok(filter)
    }
}

#[derive(Display, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "runWith")]
#[enum_dispatch]
pub enum LocalFilter {
    Bun(FilterBun),
    Deno(FilterDeno),
    Exe(FilterExe),
    Go(FilterGo),
    Nodejs(FilterNodejs),
    Python(FilterPython),
    Shell(FilterShell),
}

pub struct FilterContext {
    pub name: String,
    pub filter_dir: PathBuf,
    pub remote_config: Option<RemoteFilterConfig>,
}

impl FilterContext {
    pub fn new(name: &str, filter: &FilterDefinition) -> Result<Self> {
        match filter {
            FilterDefinition::Local(_) => Ok(Self {
                name: name.to_owned(),
                filter_dir: get_current_dir()?,
                remote_config: None,
            }),
            FilterDefinition::Remote(remote) => {
                let filter_dir = get_filter_cache_dir(name, remote)?;
                if is_dir_empty(&filter_dir)? {
                    info!("Filter <filter>{name}</> is not installed, installing...");
                    remote.install(name, None, false)?;
                }
                let remote_config =
                    read_json(filter_dir.join("filter.json")).with_context(|| {
                        format!("Failed to load config for filter <filter>{name}</>")
                    })?;
                Ok(Self {
                    name: name.to_owned(),
                    filter_dir,
                    remote_config,
                })
            }
        }
    }

    pub fn filter_dir(&self, path: &str) -> PathBuf {
        if self.remote_config.is_some() {
            self.filter_dir.to_owned()
        } else {
            let mut dir = PathBuf::from(path);
            dir.pop();
            dir
        }
    }
}

#[enum_dispatch(FilterDefinition, LocalFilter)]
pub trait Filter {
    fn run(&self, context: &FilterContext, temp: &Path, run_args: &[String]) -> Result<()>;
    #[allow(unused_variables)]
    fn install_dependencies(&self, context: &FilterContext) -> Result<()> {
        Ok(())
    }
}
