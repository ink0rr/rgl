use super::{
    get_current_dir, get_remote_filter_dir, FilterBun, FilterDeno, FilterDotnet, FilterExe,
    FilterGo, FilterJava, FilterNim, FilterNodejs, FilterPython, FilterShell, RemoteFilter,
    RemoteFilterConfig, RunContext,
};
use crate::fs::{is_dir_empty, read_json};
use crate::info;
use anyhow::{Context, Result};
use enum_dispatch::enum_dispatch;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use strum::Display;

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

#[derive(Display, Deserialize)]
#[serde(rename_all = "camelCase", tag = "runWith")]
#[enum_dispatch]
pub enum LocalFilter {
    Bun(FilterBun),
    Deno(FilterDeno),
    Dotnet(FilterDotnet),
    Exe(FilterExe),
    Go(FilterGo),
    Java(FilterJava),
    Nim(FilterNim),
    Nodejs(FilterNodejs),
    Python(FilterPython),
    Shell(FilterShell),
}

pub struct FilterInfo {
    pub name: String,
    pub filter_dir: PathBuf,
    pub remote_config: Option<RemoteFilterConfig>,
}

impl FilterInfo {
    pub fn new(name: &str, filter: &FilterDefinition) -> Result<Self> {
        match filter {
            FilterDefinition::Local(_) => Ok(Self {
                name: name.to_owned(),
                filter_dir: get_current_dir()?,
                remote_config: None,
            }),
            FilterDefinition::Remote(remote) => {
                let filter_dir = get_remote_filter_dir(name, remote)?;
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
            let mut dir = self.filter_dir.join(path);
            dir.pop();
            dir
        }
    }
}

#[enum_dispatch(FilterDefinition, LocalFilter)]
pub trait Filter {
    fn run(&self, ctx: &RunContext, info: &FilterInfo, run_args: &[String]) -> Result<()>;
    #[allow(unused_variables)]
    fn install_dependencies(&self, info: &FilterInfo) -> Result<()> {
        Ok(())
    }
}
