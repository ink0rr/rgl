use super::{
    get_cache_dir, FilterDeno, FilterExe, FilterGo, FilterNodejs, FilterPython, RemoteFilter,
};
use anyhow::{anyhow, Result};
use dunce::canonicalize;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    env,
    path::{Path, PathBuf},
};

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

    pub fn get_type(&self) -> FilterType {
        match self {
            FilterDefinition::Local(_) => FilterType::Local,
            FilterDefinition::Remote(_) => FilterType::Remote,
        }
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

pub enum FilterType {
    Local,
    Remote,
    Global,
}

impl FilterType {
    pub fn cache_dir(&self, name: &str) -> Result<PathBuf> {
        let dir = match self {
            FilterType::Local => env::current_dir()?,
            FilterType::Remote => PathBuf::from(".regolith")
                .join("cache")
                .join("filters")
                .join(name),
            FilterType::Global => get_cache_dir()?.join("global-filters").join(name),
        };
        Ok(dir)
    }
}

pub struct FilterContext {
    pub name: String,
    pub filter_dir: PathBuf,
    filter_type: FilterType,
}

impl FilterContext {
    pub fn new(filter_type: FilterType, name: &str) -> Result<Self> {
        Ok(Self {
            name: name.to_owned(),
            filter_dir: canonicalize(filter_type.cache_dir(name)?).map_err(
                |_| match filter_type {
                    FilterType::Local => unreachable!(),
                    FilterType::Remote => {
                        anyhow!("Filter <b>{name}</> is missing, run `rgl get` to retrieve it")
                    }
                    FilterType::Global => anyhow!(
                        "Filter <b>{name}</> is not installed, run `rgl install {name}` to install it"
                    ),
                },
            )?,
            filter_type,
        })
    }

    pub fn filter_dir(&self, path: &str) -> Result<PathBuf> {
        match self.filter_type {
            FilterType::Local => {
                let mut dir = PathBuf::from(path);
                dir.pop();
                Ok(dir)
            }
            _ => Ok(self.filter_dir.to_owned()),
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
