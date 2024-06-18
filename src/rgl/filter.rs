use super::{
    get_current_dir, get_filter_cache_dir, FilterDeno, FilterExe, FilterGo, FilterNodejs,
    FilterPython, RemoteFilter,
};
use crate::info;
use anyhow::Result;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "runWith")]
#[enum_dispatch]
pub enum LocalFilter {
    Deno(FilterDeno),
    Exe(FilterExe),
    Go(FilterGo),
    Nodejs(FilterNodejs),
    Python(FilterPython),
}

pub struct FilterContext {
    pub name: String,
    pub filter_dir: PathBuf,
    pub is_remote: bool,
}

impl FilterContext {
    pub fn new(name: &str, filter: &FilterDefinition) -> Result<Self> {
        let filter_dir = match &filter {
            FilterDefinition::Remote(remote) => {
                let dir = get_filter_cache_dir(name, remote)?;
                if !dir.exists() {
                    info!("Filter {name} is not installed, installing...");
                    remote.install(name, None, false)?;
                }
                dir
            }
            _ => get_current_dir()?,
        };
        Ok(Self {
            name: name.to_owned(),
            filter_dir,
            is_remote: matches!(filter, FilterDefinition::Remote(_)),
        })
    }

    pub fn filter_dir(&self, path: &str) -> PathBuf {
        if self.is_remote {
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
