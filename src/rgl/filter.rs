use super::{FilterDeno, FilterGo, FilterNode, FilterPython, RemoteFilter};
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

    pub fn is_remote(&self) -> bool {
        matches!(self, FilterDefinition::Remote(_))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "runWith")]
#[enum_dispatch]
pub enum LocalFilter {
    Deno(FilterDeno),
    Go(FilterGo),
    Nodejs(FilterNode),
    Python(FilterPython),
}

pub struct FilterContext {
    pub name: String,
    pub dir: PathBuf,
}

impl FilterContext {
    pub fn new(name: &str, is_remote: bool) -> Result<Self> {
        Ok(Self {
            name: name.to_owned(),
            dir: match is_remote {
                true => canonicalize(RemoteFilter::cache_dir(name)).map_err(|_| {
                    anyhow!("Filter <b>{name}</> not installed, run \"rgl install\" to install it")
                })?,
                false => env::current_dir()?,
            },
        })
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
